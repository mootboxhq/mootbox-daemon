use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::signal::unix::{SignalKind, signal};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::api::mootbox_url_of;
use crate::config::{self, DaemonConfig};
use crate::daemon::{self, DaemonOptions, DaemonTiming};
use crate::journal::Journal;
use crate::lock::{self, DaemonLock};
use crate::logging;
use crate::providers::{ProviderKind, ProviderOptions, SelectedProvider, probe};
use crate::run::{self, RunTiming};

const NOT_SIGNED_IN: &str = "is not ready: authentication_required";

pub struct StartOptions {
    pub provider: Option<ProviderKind>,
    pub slots: Option<u32>,
    pub wall_time_minutes: Option<u32>,
    pub provider_options: ProviderOptions,
}

pub struct OnlineRunner {
    daemon: DaemonOptions,
    lock: DaemonLock,
}

impl OnlineRunner {
    pub async fn serve(self) {
        let Self { daemon, lock } = self;
        daemon::run_daemon(daemon).await;
        info!("runner stopped");
        drop(lock);
    }
}

pub async fn run(home: &Path, options: StartOptions) -> Result<()> {
    go_online(home, options).await?.serve().await;
    Ok(())
}

pub fn save_options(home: &Path, options: &StartOptions) -> Result<()> {
    let mut config = config::load(home)?;
    if let Some(provider) = options.provider {
        if provider != config.provider {
            config.provider_command = None;
        }
        config.provider = provider;
    }
    config.slots = options.slots.unwrap_or(config.slots);
    config.wall_time_minutes = options.wall_time_minutes.unwrap_or(config.wall_time_minutes);
    config.model = options.provider_options.model.clone().or(config.model);
    config.max_budget_usd = options.provider_options.max_budget_usd.or(config.max_budget_usd);
    config::save(home, &config)
}

pub async fn go_online(home: &Path, options: StartOptions) -> Result<OnlineRunner> {
    logging::init(home)?;
    let config = config::load(home)?;
    if config.workspaces.is_empty() {
        bail!(config::NOT_CONNECTED);
    }
    for profile in &config.workspaces {
        mootbox_url_of(profile)?;
    }
    let lock = lock::acquire(home)?;
    let journal = Journal::open(home)?;
    let selected_provider = config.selected_provider();
    require_ready_provider(&selected_provider).await?;
    let provider_options = provider_options(options.provider_options, &config)?;
    let slots = options.slots.unwrap_or(config.slots);
    let wall_time_minutes = options.wall_time_minutes.unwrap_or(config.wall_time_minutes);
    info!(
        workspaces = config.workspaces.len(),
        connections = config.connections.len(),
        slots,
        wall_time_minutes,
        "runner online"
    );
    let shutdown = CancellationToken::new();
    cancel_on_stop_signal(shutdown.clone())?;
    let timing = DaemonTiming {
        shutdown_grace: Duration::from_secs(config.shutdown_grace_seconds),
        ..DaemonTiming::default()
    };
    let run_timing = RunTiming {
        wall_time: run::wall_time_of(wall_time_minutes),
        ..RunTiming::default()
    };
    let daemon = DaemonOptions {
        config,
        home: home.to_path_buf(),
        selected_provider,
        provider_options,
        slots,
        timing,
        run_timing,
        journal,
        shutdown,
    };
    Ok(OnlineRunner { daemon, lock })
}

async fn require_ready_provider(selected: &SelectedProvider) -> Result<()> {
    probe::supported_version(selected.kind, &selected.program).await?;
    if probe::is_signed_in(selected.kind, &selected.program).await == Some(false) {
        bail!(
            "{} {NOT_SIGNED_IN}; run `{}` first",
            selected.kind.label(),
            selected.program.display()
        );
    }
    Ok(())
}

fn provider_options(flags: ProviderOptions, config: &DaemonConfig) -> Result<ProviderOptions> {
    let max_budget_usd = flags.max_budget_usd.or(config.max_budget_usd);
    if max_budget_usd.is_some_and(|budget| !budget.is_finite() || budget <= 0.0) {
        bail!("max_budget_usd in the daemon config must be a positive amount");
    }
    Ok(ProviderOptions {
        model: flags.model.or_else(|| config.model.clone()),
        max_budget_usd,
    })
}

fn cancel_on_stop_signal(shutdown: CancellationToken) -> Result<()> {
    let mut interrupt = signal(SignalKind::interrupt()).context("cannot listen for SIGINT")?;
    let mut terminate = signal(SignalKind::terminate()).context("cannot listen for SIGTERM")?;
    tokio::spawn(async move {
        tokio::select! {
            _ = interrupt.recv() => {}
            _ = terminate.recv() => {}
        }
        info!("shutdown requested; finishing active runs");
        shutdown.cancel();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configured(model: Option<&str>, max_budget_usd: Option<f64>) -> DaemonConfig {
        DaemonConfig {
            model: model.map(str::to_string),
            max_budget_usd,
            ..DaemonConfig::default()
        }
    }

    #[test]
    fn start_flags_override_the_configured_model_and_budget() {
        let flags = ProviderOptions {
            model: Some("sonnet".to_string()),
            max_budget_usd: Some(1.0),
        };

        let options = provider_options(flags, &configured(Some("opus"), Some(5.0))).expect("valid");

        assert_eq!(options.model.as_deref(), Some("sonnet"));
        assert_eq!(options.max_budget_usd, Some(1.0));
    }

    #[test]
    fn the_configured_model_and_budget_apply_without_flags() {
        let options =
            provider_options(ProviderOptions::default(), &configured(Some("opus"), Some(5.0))).expect("valid");

        assert_eq!(options.model.as_deref(), Some("opus"));
        assert_eq!(options.max_budget_usd, Some(5.0));
    }

    #[test]
    fn a_configured_budget_that_is_not_positive_is_refused() {
        for budget in [0.0, -1.0, f64::NAN] {
            assert!(provider_options(ProviderOptions::default(), &configured(None, Some(budget))).is_err());
        }
    }
}
