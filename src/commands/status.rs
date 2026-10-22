use std::path::Path;

use anyhow::Result;

use super::{Output, service};
use crate::config::{self, DaemonConfig};
use crate::service::ServiceHost;

pub fn run(home: &Path) -> Result<Output> {
    let mut lines = summary(&config::load(home)?);
    if let Ok(host) = ServiceHost::of_this_machine() {
        lines.push(service::status_line(host.state()));
    }
    Ok(Output::lines(lines))
}

fn summary(config: &DaemonConfig) -> Vec<String> {
    let mut lines = vec![
        format!("provider: {}", config.provider),
        format!("slots: {}", config.slots),
        format!("allowed_commands: {}", listed_or_none(&config.allowed_commands)),
        format!("workspaces: {}", config.workspaces.len()),
    ];
    lines.extend(config.workspaces.iter().map(|profile| {
        format!(
            "  {} on {} (runner {})",
            profile.workspace_id, profile.mootbox_url, profile.runner_id
        )
    }));
    lines.push(format!("connections: {}", config.connections.len()));
    lines.extend(config.connections.values().map(|connection| {
        let remote = connection
            .remote
            .as_ref()
            .map(|remote| format!(" (remote: {remote})"))
            .unwrap_or_default();
        format!(
            "  {} -> {}{remote} (workspace {})",
            connection.repository_id,
            connection.path.display(),
            connection.workspace_id
        )
    }));
    lines
}

fn listed_or_none(items: &[String]) -> String {
    if items.is_empty() {
        return "none".to_string();
    }
    items.join(", ")
}
