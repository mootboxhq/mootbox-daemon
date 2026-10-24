use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use reqwest::Url;

use super::{Console, Output};
use crate::api::{ApiError, MootboxClient};
use crate::config::{self, CliSession, DEFAULT_MOOTBOX_URL, DaemonConfig};
use crate::contracts::{ExchangeCliLoginRequest, ExchangeCliLoginResponse, StartCliLoginRequest};

const SIGN_IN_AGAIN: &str = "run mootbox login again";

pub struct LoginOptions {
    pub mootbox_url: Option<Url>,
    pub device_name: String,
    pub poll_interval: Duration,
}

pub struct SignIn<'a> {
    pub mootbox_url: &'a Url,
    pub device_name: &'a str,
    pub poll_interval: Duration,
}

impl LoginOptions {
    pub fn server(&self, config: &DaemonConfig) -> Result<Url> {
        if let Some(mootbox_url) = &self.mootbox_url {
            return Ok(mootbox_url.clone());
        }
        if let Some(session) = &config.session {
            return session.server();
        }
        match config.shared_mootbox_url() {
            Some(mootbox_url) => Ok(mootbox_url),
            None => Url::parse(DEFAULT_MOOTBOX_URL).context("the default Mootbox URL is not a URL"),
        }
    }

    pub fn at<'a>(&'a self, mootbox_url: &'a Url) -> SignIn<'a> {
        SignIn {
            mootbox_url,
            device_name: &self.device_name,
            poll_interval: self.poll_interval,
        }
    }
}

pub async fn run(home: &Path, options: &LoginOptions, console: &dyn Console) -> Result<Output> {
    let mootbox_url = options.server(&config::load_or_start_fresh(home)?)?;
    sign_in(home, &options.at(&mootbox_url), console).await?;
    Ok(Output::default())
}

pub async fn current_session(home: &Path, options: &LoginOptions, console: &dyn Console) -> Result<CliSession> {
    let config = config::load_or_start_fresh(home)?;
    let mootbox_url = options.server(&config)?;
    if let Some(session) = config.session.filter(|session| session.is_on(&mootbox_url)) {
        return Ok(session);
    }
    sign_in(home, &options.at(&mootbox_url), console).await
}

pub async fn renewed_session(home: &Path, request: &SignIn<'_>, console: &dyn Console) -> Result<CliSession> {
    forget_session(home)?;
    sign_in(home, request, console).await
}

pub async fn sign_in(home: &Path, request: &SignIn<'_>, console: &dyn Console) -> Result<CliSession> {
    let session = signed_in_session(request, console).await?;
    let mut config = config::load_or_start_fresh(home)?;
    config.session = Some(session.clone());
    config::save(home, &config)?;
    console.show(&format!("Signed in as {}.", session.email));
    Ok(session)
}

fn forget_session(home: &Path) -> Result<()> {
    let mut config = config::load_or_start_fresh(home)?;
    if config.session.take().is_none() {
        return Ok(());
    }
    config::save(home, &config)
}

async fn signed_in_session(request: &SignIn<'_>, console: &dyn Console) -> Result<CliSession> {
    let client = MootboxClient::unauthenticated(request.mootbox_url.clone());
    let start = StartCliLoginRequest {
        name: request.device_name.try_into()?,
    };
    let started = client.start_login(&start).await?;
    let confirm_url = Url::parse(&started.confirm_url).context("Mootbox sent a sign-in link that is not a URL")?;
    console.open_link(
        &format!("Sign in with code {} in your browser", started.user_code),
        &confirm_url,
    );
    let exchange = ExchangeCliLoginRequest {
        device_secret: started.device_secret.as_str().try_into()?,
    };
    loop {
        tokio::time::sleep(request.poll_interval).await;
        let exchanged = client
            .exchange_login(started.login_id, &exchange)
            .await
            .map_err(refused_sign_in)?;
        if let ExchangeCliLoginResponse::SignedIn { session_token, user } = exchanged {
            return Ok(CliSession {
                mootbox_url: request.mootbox_url.to_string(),
                token: session_token,
                email: user.email,
            });
        }
    }
}

pub(crate) fn refused_sign_in(error: ApiError) -> anyhow::Error {
    match error {
        ApiError::Mootbox { message, .. } => anyhow!("{message}; {SIGN_IN_AGAIN}"),
        other => other.into(),
    }
}
