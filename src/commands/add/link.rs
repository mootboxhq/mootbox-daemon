use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::checkout::{LocalCheckout, accepted, without_credentials};
use super::placement::{self, Attachment};
use super::workspace::is_on_mootbox;
use super::{Onboarding, Serving, bring_online, record_connection, server_message};
use crate::api::{ApiError, MootboxClient};
use crate::commands::Console;
use crate::commands::login::{self, LoginOptions};
use crate::commands::start::StartOptions;
use crate::config::{self, CliSession, DaemonConfig, WorkspaceProfile};
use crate::contracts::{
    ErrorErrorCategory, ExchangeRepositoryLinkResponse, StartRepositoryLinkRequest,
    StartRepositoryLinkRequestRemotesItem, StartRepositoryLinkResponse,
};
use crate::device;
use crate::git;

const LINK_COMMAND: &str = "mootbox";
const CHOOSE_PROMPT: &str = "Choose a project for this repository in your browser";

pub struct LinkOptions {
    pub login: LoginOptions,
    pub directory: PathBuf,
}

pub struct LinkRequest {
    pub options: LinkOptions,
    pub start: StartOptions,
    pub serving: Serving,
}

#[derive(Debug)]
pub struct LinkReport {
    pub workspace_name: String,
    pub project_name: String,
    pub chat_url: String,
}

#[derive(Deserialize)]
struct Linked {
    workspace: Named,
    project: Named,
    repository_id: Uuid,
    chat_url: String,
    runner: Option<MintedRunner>,
}

#[derive(Deserialize)]
struct Named {
    id: Uuid,
    name: String,
}

#[derive(Deserialize)]
struct MintedRunner {
    runner_id: Uuid,
    workspace_id: Uuid,
    workspace_name: String,
    credential: String,
}

enum Attached {
    Connected,
    Revoked(WorkspaceProfile),
}

struct Linking<'a> {
    home: &'a Path,
    options: &'a LinkOptions,
    console: &'a dyn Console,
}

impl LinkReport {
    pub fn lines(&self) -> Vec<String> {
        vec![
            format!("Linked to “{}” in {}.", self.project_name, self.workspace_name),
            String::new(),
            format!("Start chatting: {}", self.chat_url),
        ]
    }
}

impl Linked {
    fn from_response(response: &impl Serialize) -> Result<Self> {
        serde_json::to_value(response)
            .and_then(serde_json::from_value)
            .context("Mootbox sent a repository link this mootbox cannot read")
    }

    fn attachment(&self) -> Attachment {
        Attachment {
            repository_id: self.repository_id,
            project_id: self.project.id,
        }
    }
}

pub async fn run(home: &Path, request: LinkRequest, console: &dyn Console) -> Result<()> {
    let report = link(home, &request.options, console).await?;
    let onboarding = Onboarding {
        summary: report.lines(),
        rerun_command: LINK_COMMAND,
        start: request.start,
        serving: request.serving,
    };
    bring_online(home, onboarding, console).await
}

pub async fn link(home: &Path, options: &LinkOptions, console: &dyn Console) -> Result<LinkReport> {
    let checkout = LocalCheckout::of(&options.directory, console).await?;
    let linking = Linking { home, options, console };
    let session = login::current_session(home, &options.login, console).await?;
    let (session, linked) = linking.linked(session, &checkout).await?;
    let linked = match linking.attach(&session, &checkout, &linked).await? {
        Attached::Connected => linked,
        Attached::Revoked(profile) => linking.relinked(session, &checkout, &profile).await?,
    };
    Ok(LinkReport {
        workspace_name: linked.workspace.name,
        project_name: linked.project.name,
        chat_url: linked.chat_url,
    })
}

impl Linking<'_> {
    async fn linked(&self, session: CliSession, checkout: &LocalCheckout) -> Result<(CliSession, Linked)> {
        let request = self.link_request(&session, checkout).await?;
        let (session, started) = self.started(session, &request).await?;
        let linked = match started {
            StartRepositoryLinkResponse::Linked { .. } => Linked::from_response(&started)?,
            StartRepositoryLinkResponse::Choose {
                link_id, choose_url, ..
            } => self.chosen(&session, link_id, &choose_url).await?,
        };
        Ok((session, linked))
    }

    async fn started(
        &self,
        session: CliSession,
        request: &StartRepositoryLinkRequest,
    ) -> Result<(CliSession, StartRepositoryLinkResponse)> {
        let mootbox_url = session.server()?;
        let (session, started) = match MootboxClient::for_session(&session)?.start_link(request).await {
            Err(error) if error.is_unauthorized() => {
                let sign_in = self.options.login.at(&mootbox_url);
                let renewed = login::renewed_session(self.home, &sign_in, self.console).await?;
                let started = MootboxClient::for_session(&renewed)?.start_link(request).await;
                (renewed, started)
            }
            started => (session, started),
        };
        let started = started.map_err(|error| refused_link(error, &mootbox_url))?;
        Ok((session, started))
    }

    async fn chosen(&self, session: &CliSession, link_id: Uuid, choose_url: &str) -> Result<Linked> {
        let choose_url = Url::parse(choose_url).context("Mootbox sent a project link that is not a URL")?;
        self.console.open_link(CHOOSE_PROMPT, &choose_url);
        let client = MootboxClient::for_session(session)?;
        loop {
            tokio::time::sleep(self.options.login.poll_interval).await;
            let exchanged = client.exchange_link(link_id).await.map_err(server_message)?;
            if let ExchangeRepositoryLinkResponse::Linked { .. } = exchanged {
                return Linked::from_response(&exchanged);
            }
        }
    }

    async fn link_request(&self, session: &CliSession, checkout: &LocalCheckout) -> Result<StartRepositoryLinkRequest> {
        let config = config::load_or_start_fresh(self.home)?;
        let selected = config.selected_provider();
        let device = device::report_device(|kind| selected.program_for(kind)).await;
        Ok(StartRepositoryLinkRequest {
            remotes: identifying_first(checkout)
                .into_iter()
                .map(link_remote)
                .collect::<Result<_>>()?,
            name: accepted("repository name", &checkout.name)?,
            default_branch: accepted("default branch", &checkout.default_branch)?,
            device: device.named_contract(&self.options.login.device_name)?,
            paired_workspace_ids: paired_workspace_ids(&config, &session.server()?),
        })
    }

    async fn attach(&self, session: &CliSession, checkout: &LocalCheckout, linked: &Linked) -> Result<Attached> {
        let mut config = config::load_or_start_fresh(self.home)?;
        let profile = linked_profile(&mut config, session, linked)?;
        if linked.runner.is_some() {
            config::save(self.home, &config)?;
        }
        let request = placement::connection_request(checkout, linked.attachment())?;
        let connected = match MootboxClient::for_profile(&profile)?.upsert_connection(&request).await {
            Err(error) if error.is_unauthorized() && linked.runner.is_none() => return Ok(Attached::Revoked(profile)),
            connected => connected.map_err(server_message)?,
        };
        record_connection(
            &mut config,
            connected.connection_id,
            checkout.connection(profile.workspace_id, linked.attachment()),
        );
        config::save(self.home, &config)?;
        Ok(Attached::Connected)
    }

    async fn relinked(
        &self,
        session: CliSession,
        checkout: &LocalCheckout,
        revoked: &WorkspaceProfile,
    ) -> Result<Linked> {
        self.forget(revoked)?;
        let (session, linked) = self.linked(session, checkout).await?;
        match self.attach(&session, checkout, &linked).await? {
            Attached::Connected => Ok(linked),
            Attached::Revoked(profile) => Err(anyhow!(
                "This computer's access to {} was removed. Run mootbox again.",
                profile.display_name()
            )),
        }
    }

    fn forget(&self, revoked: &WorkspaceProfile) -> Result<()> {
        let mut config = config::load_or_start_fresh(self.home)?;
        config.workspaces.retain(|saved| saved != revoked);
        config::save(self.home, &config)?;
        self.console.warn(&format!(
            "This computer's access to {} was removed; reconnecting.",
            revoked.display_name()
        ));
        Ok(())
    }
}

fn linked_profile(config: &mut DaemonConfig, session: &CliSession, linked: &Linked) -> Result<WorkspaceProfile> {
    if let Some(runner) = &linked.runner {
        let profile = WorkspaceProfile {
            mootbox_url: session.mootbox_url.clone(),
            workspace_id: runner.workspace_id,
            runner_id: runner.runner_id,
            credential: runner.credential.clone(),
            workspace_name: Some(runner.workspace_name.clone()),
        };
        config.upsert_workspace(profile.clone());
        return Ok(profile);
    }
    let mootbox_url = session.server()?;
    config
        .workspaces
        .iter()
        .find(|profile| profile.workspace_id == linked.workspace.id && is_on_mootbox(profile, &mootbox_url))
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "This computer is not paired with the {} workspace. Run mootbox again.",
                linked.workspace.name
            )
        })
}

fn identifying_first(checkout: &LocalCheckout) -> Vec<&git::Remote> {
    std::iter::once(&checkout.remote)
        .chain(
            checkout
                .remotes
                .iter()
                .filter(|remote| remote.name != checkout.remote.name),
        )
        .collect()
}

fn link_remote(remote: &git::Remote) -> Result<StartRepositoryLinkRequestRemotesItem> {
    Ok(StartRepositoryLinkRequestRemotesItem {
        name: accepted("remote name", &remote.name)?,
        url: accepted("remote URL", &without_credentials(&remote.url))?,
    })
}

fn paired_workspace_ids(config: &DaemonConfig, mootbox_url: &Url) -> Vec<Uuid> {
    config
        .workspaces
        .iter()
        .filter(|profile| is_on_mootbox(profile, mootbox_url))
        .map(|profile| profile.workspace_id)
        .collect()
}

fn refused_link(error: ApiError, mootbox_url: &Url) -> anyhow::Error {
    if error.is_unauthorized() {
        return login::refused_sign_in(error);
    }
    match error {
        ApiError::Mootbox {
            category: ErrorErrorCategory::PermissionDenied | ErrorErrorCategory::NotFound,
            message,
            ..
        } => anyhow!("{message}\nOpen {mootbox_url} to see the workspaces and projects you can use."),
        other => server_message(other),
    }
}
