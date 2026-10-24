use std::io::IsTerminal;
use std::process::ExitCode;
use std::sync::LazyLock;

use anyhow::{Context, Result};
use clap::builder::NonEmptyStringValueParser;
use clap::{Args, Parser, Subcommand};
use reqwest::Url;
use uuid::Uuid;

use crate::commands::add::{AddOptions, AddRequest, Serving};
use crate::commands::pair::PairOptions;
use crate::commands::service::ServiceAction as Action;
use crate::commands::start::StartOptions;
use crate::commands::{self, Console, Output};
use crate::config;
use crate::pairing::PAIRING_POLL_INTERVAL;
use crate::providers::{self, ProviderKind, ProviderOptions};

mod console;

use console::TerminalConsole;

const DEFAULT_DEVICE_NAME: &str = "runner";

static PROVIDER_HELP: LazyLock<String> = LazyLock::new(|| {
    let names = providers::ALL.map(|kind| kind.name()).join(" or ");
    format!(
        "Provider to run: {names} [default: provider in the config, {}]",
        providers::DEFAULT_PROVIDER
    )
});

#[derive(Parser)]
#[command(name = "mootbox", version, about = "Runs your Mootbox assistant on this machine")]
struct Cli {
    #[command(subcommand)]
    command: DaemonCommand,
}

#[derive(Args)]
struct StartArgs {
    #[arg(long, value_parser = provider_kind, help = PROVIDER_HELP.as_str())]
    provider: Option<ProviderKind>,
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    slots: Option<u32>,
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..), help = "Stop runs after this many minutes [default: wall_time_minutes in the config, 30]")]
    wall_time_minutes: Option<u32>,
    #[arg(long, value_parser = NonEmptyStringValueParser::new(), help = "Model for every run, in the provider's own naming [default: model in the config]")]
    model: Option<String>,
    #[arg(long, value_parser = positive_amount, help = "Spending limit per run, in USD; honoured by Claude Code only [default: max_budget_usd in the config]")]
    max_budget_usd: Option<f64>,
}

#[derive(Subcommand)]
enum DaemonCommand {
    #[command(
        alias = "connect",
        about = "Connect this repo folder to a Mootbox project and bring this computer online"
    )]
    Add {
        #[arg(long, help = "Mootbox server URL [default: the one this computer is paired with]")]
        mootbox_url: Option<Url>,
        #[arg(long, value_name = "PROJECT_ID", help = "Connect to this project without asking")]
        project: Option<Uuid>,
        #[arg(long, help = "Pair this computer with another workspace on the same Mootbox server")]
        new_workspace: bool,
        #[arg(long, help = "Serve in this terminal until Ctrl+C instead of in the background")]
        foreground: bool,
        #[command(flatten)]
        start: StartArgs,
    },
    #[command(about = "Pair this machine with a Mootbox server")]
    Pair {
        #[arg(long)]
        mootbox_url: Url,
        #[arg(long, help = "Device name shown in Mootbox [default: this machine's hostname]")]
        name: Option<String>,
        #[arg(
            long,
            value_parser = NonEmptyStringValueParser::new(),
            help = "Pairing code from the Devices page [default: ask, or print a code to enter on the web]"
        )]
        code: Option<String>,
    },
    #[command(
        hide = true,
        about = "Permit an extra shell command for managed runs, e.g. \"Bash(pnpm test:*)\""
    )]
    Allow { rule: String },
    #[command(about = "Show the daemon config without secrets and whether the service runs")]
    Status,
    #[command(about = "Install, remove or show the login service that keeps the daemon running")]
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
    #[command(about = "Check the provider CLI, git, the config, bound repositories, the server and the service")]
    Doctor,
    #[command(about = "Claim and run Mootbox work for every connected repo until stopped")]
    Start(StartArgs),
    #[command(about = "Replace this mootbox with the latest release and restart the background service")]
    Update {
        #[arg(long, help = "Only report whether a newer release exists")]
        check: bool,
    },
}

#[derive(Clone, Copy, Subcommand)]
enum ServiceAction {
    #[command(about = "Run `mootbox start` at login and restart it after a crash")]
    Install,
    #[command(about = "Stop the service and remove its file")]
    Uninstall,
    #[command(about = "Show whether the service is installed, loaded and running")]
    Status,
}

pub async fn run() -> ExitCode {
    match execute(Cli::parse().command).await {
        Ok(output) => {
            print(&output);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("mootbox: {error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn execute(command: DaemonCommand) -> Result<Output> {
    let home = config::daemon_home()?;
    match command {
        DaemonCommand::Add {
            mootbox_url,
            project,
            new_workspace,
            foreground,
            start,
        } => {
            let request = AddRequest {
                options: AddOptions {
                    mootbox_url,
                    project,
                    new_workspace,
                    device_name: default_device_name(),
                    directory: std::env::current_dir().context("cannot read the current folder")?,
                    poll_interval: PAIRING_POLL_INTERVAL,
                },
                start: start.into_options(),
                serving: if foreground {
                    Serving::Foreground
                } else {
                    Serving::Background
                },
            };
            commands::add::run(&home, request, &TerminalConsole).await?;
            Ok(Output::default())
        }
        DaemonCommand::Pair {
            mootbox_url,
            name,
            code,
        } => {
            let options = PairOptions {
                mootbox_url,
                device_name: name.unwrap_or_else(default_device_name),
                code: code.map_or_else(prompted_pairing_code, |code| Ok(Some(code)))?,
                poll_interval: PAIRING_POLL_INTERVAL,
            };
            commands::pair::run(&home, &options, &TerminalConsole).await
        }
        DaemonCommand::Allow { rule } => commands::allow::run(&home, &rule),
        DaemonCommand::Status => commands::status::run(&home),
        DaemonCommand::Service { action } => commands::service::run(&home, action.into()),
        DaemonCommand::Doctor => {
            let report = commands::doctor::run(&home).await;
            print(&report.output());
            report.verdict().map(|()| Output::default())
        }
        DaemonCommand::Start(start) => {
            commands::start::run(&home, start.into_options()).await?;
            Ok(Output::default())
        }
        DaemonCommand::Update { check: true } => commands::update::check().await,
        DaemonCommand::Update { check: false } => commands::update::apply(&home).await,
    }
}

fn print(output: &Output) {
    output.lines.iter().for_each(|line| TerminalConsole.show(line));
    output.warnings.iter().for_each(|line| TerminalConsole.warn(line));
}

impl StartArgs {
    fn into_options(self) -> StartOptions {
        StartOptions {
            provider: self.provider,
            slots: self.slots,
            wall_time_minutes: self.wall_time_minutes,
            provider_options: ProviderOptions {
                model: self.model,
                max_budget_usd: self.max_budget_usd,
            },
        }
    }
}

impl From<ServiceAction> for Action {
    fn from(action: ServiceAction) -> Self {
        match action {
            ServiceAction::Install => Self::Install,
            ServiceAction::Uninstall => Self::Uninstall,
            ServiceAction::Status => Self::Status,
        }
    }
}

fn positive_amount(text: &str) -> Result<f64, String> {
    match text.parse::<f64>() {
        Ok(amount) if amount.is_finite() && amount > 0.0 => Ok(amount),
        _ => Err(format!("expected a positive amount, got {text:?}")),
    }
}

fn provider_kind(text: &str) -> Result<ProviderKind, String> {
    text.parse()
        .map_err(|error: providers::UnknownProvider| error.to_string())
}

fn prompted_pairing_code() -> Result<Option<String>> {
    if !std::io::stdin().is_terminal() {
        return Ok(None);
    }
    commands::pair::read_pairing_code(&mut std::io::stdin().lock(), &mut std::io::stderr())
}

fn default_device_name() -> String {
    nix::unistd::gethostname()
        .ok()
        .and_then(|hostname| hostname.into_string().ok())
        .filter(|hostname| !hostname.is_empty())
        .unwrap_or_else(|| DEFAULT_DEVICE_NAME.to_string())
}
