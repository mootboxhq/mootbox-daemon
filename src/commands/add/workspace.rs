use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result, bail};
use reqwest::Url;

use super::{Adding, CONNECT_ANOTHER_WORKSPACE, NO_MOOTBOX_URL, WHICH_WORKSPACE, server_message};
use crate::config::{DaemonConfig, WorkspaceProfile};
use crate::pairing::{self, PairingRequest};

const CONFIRM_PATH: &str = "devices/confirm";
const CODE_PARAMETER: &str = "code";
const PAIRED_PARAMETER: &str = "paired";
const PAIRED_SEPARATOR: &str = ",";

impl Adding<'_> {
    pub(super) async fn chosen_profile(&self, config: &DaemonConfig, root: &Path) -> Result<WorkspaceProfile> {
        let profiles: Vec<&WorkspaceProfile> = config
            .workspaces
            .iter()
            .filter(|profile| is_on_mootbox(profile, &self.mootbox_url))
            .collect();
        if self.options.new_workspace || profiles.is_empty() {
            return self.pair(&profiles).await;
        }
        let connected: Vec<&WorkspaceProfile> = profiles
            .iter()
            .copied()
            .filter(|profile| has_checkout(config, profile, root))
            .collect();
        if let [only] = connected.as_slice() {
            return Ok((*only).clone());
        }
        if let ([only], Some(_)) = (profiles.as_slice(), self.options.project) {
            return Ok((*only).clone());
        }
        let mut choices: Vec<String> = profiles.iter().map(|profile| profile.display_name()).collect();
        choices.push(CONNECT_ANOTHER_WORKSPACE.to_string());
        let picked = self.console.choose(WHICH_WORKSPACE, &choices)?;
        match profiles.get(picked) {
            Some(profile) => Ok((*profile).clone()),
            None => self.pair(&profiles).await,
        }
    }

    async fn pair(&self, paired: &[&WorkspaceProfile]) -> Result<WorkspaceProfile> {
        let request = PairingRequest {
            mootbox_url: &self.mootbox_url,
            device_name: &self.options.device_name,
            poll_interval: self.options.poll_interval,
        };
        pairing::pair_device(&request, |started| {
            self.console
                .open_link(&confirm_link(&self.mootbox_url, &started.user_code, paired)?);
            Ok(())
        })
        .await
        .map_err(server_message)
    }
}

pub fn chosen_mootbox_url(flag: Option<&Url>, config: &DaemonConfig) -> Result<Url> {
    if let Some(mootbox_url) = flag {
        return Ok(mootbox_url.clone());
    }
    let saved: BTreeSet<Url> = config
        .workspaces
        .iter()
        .filter_map(|profile| Url::parse(&profile.mootbox_url).ok())
        .collect();
    match Vec::from_iter(saved).as_slice() {
        [only] => Ok(only.clone()),
        _ => bail!(NO_MOOTBOX_URL),
    }
}

pub fn confirm_link(mootbox_url: &Url, user_code: &str, paired: &[&WorkspaceProfile]) -> Result<Url> {
    let mut link = mootbox_url
        .join(CONFIRM_PATH)
        .context("cannot build the device confirm link")?;
    let paired_ids: Vec<String> = paired.iter().map(|profile| profile.workspace_id.to_string()).collect();
    link.query_pairs_mut()
        .append_pair(CODE_PARAMETER, user_code)
        .append_pair(PAIRED_PARAMETER, &paired_ids.join(PAIRED_SEPARATOR));
    Ok(link)
}

fn is_on_mootbox(profile: &WorkspaceProfile, mootbox_url: &Url) -> bool {
    Url::parse(&profile.mootbox_url).is_ok_and(|saved| saved == *mootbox_url)
}

fn has_checkout(config: &DaemonConfig, profile: &WorkspaceProfile, root: &Path) -> bool {
    config
        .connections
        .values()
        .any(|connection| connection.workspace_id == profile.workspace_id && connection.path == root)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn the_confirm_link_carries_the_code_and_the_workspaces_already_paired() {
        let mootbox_url = Url::parse("https://mootbox.example.com").expect("URL");
        let profile = WorkspaceProfile {
            mootbox_url: mootbox_url.to_string(),
            workspace_id: Uuid::nil(),
            runner_id: Uuid::nil(),
            credential: "c".to_string(),
            workspace_name: None,
        };

        let link = confirm_link(&mootbox_url, "ABCD-EFGH", &[&profile]).expect("link");

        assert_eq!(link.path(), "/devices/confirm");
        let query: Vec<(String, String)> = link.query_pairs().into_owned().collect();
        assert_eq!(
            query,
            [
                ("code".to_string(), "ABCD-EFGH".to_string()),
                ("paired".to_string(), Uuid::nil().to_string()),
            ]
        );
    }
}
