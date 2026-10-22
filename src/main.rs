use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    mootbox_daemon::cli::run().await
}
