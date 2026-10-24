use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use reqwest::Url;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::placement::Placement;
use super::{DETACHED_HEAD, NO_COMMITS, NO_REMOTE, NOT_IN_REPOSITORY, WHICH_REMOTE};
use crate::commands::Console;
use crate::config::Connection;
use crate::contracts::error::ConversionError;
use crate::contracts::{
    MatchRepositoriesRequestRemotesItem, UpsertConnectionRequestBranch, UpsertConnectionRequestHeadSha,
};
use crate::git;

const ORIGIN: &str = "origin";
const DEFAULT_BRANCH: &str = "main";
const CREDENTIAL_SCHEMES: [&str; 2] = ["http", "https"];

pub struct LocalCheckout {
    pub root: PathBuf,
    pub name: String,
    pub remotes: Vec<git::Remote>,
    pub remote: git::Remote,
    pub default_branch: String,
    pub branch: UpsertConnectionRequestBranch,
    pub head_sha: UpsertConnectionRequestHeadSha,
    pub is_dirty: bool,
}

impl LocalCheckout {
    pub async fn of(directory: &Path, console: &dyn Console) -> Result<Self> {
        let root = git::top_level(directory)
            .await
            .map_err(|_| anyhow!(NOT_IN_REPOSITORY))?
            .canonicalize()
            .context("cannot resolve the repository folder")?;
        let remotes = git::remotes(&root).await?;
        let remote = identifying_remote(&remotes, console)?;
        let default_branch = git::remote_default_branch(&root, &remote.name)
            .await
            .unwrap_or_else(|| DEFAULT_BRANCH.to_string());
        let name = root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| anyhow!(NOT_IN_REPOSITORY))?;
        let state = git::checkout_state(&root).await?;
        let head = state.head.ok_or_else(|| anyhow!(NO_COMMITS))?;
        let branch = state.branch.ok_or_else(|| anyhow!(DETACHED_HEAD))?;
        Ok(Self {
            branch: accepted("branch", &branch)?,
            head_sha: accepted("commit", &head)?,
            is_dirty: state.is_dirty,
            root,
            name,
            remotes,
            remote,
            default_branch,
        })
    }

    pub fn connection(&self, workspace_id: Uuid, placement: &Placement) -> Connection {
        Connection {
            workspace_id,
            repository_id: placement.repository_id,
            project_ids: BTreeSet::from([placement.project_id]),
            path: self.root.clone(),
            remote: Some(self.remote.name.clone()),
        }
    }

    pub fn local_key(&self) -> String {
        local_key(&self.root)
    }
}

pub fn matched_remote(remote: &git::Remote) -> Result<MatchRepositoriesRequestRemotesItem> {
    Ok(MatchRepositoriesRequestRemotesItem {
        name: accepted("remote name", &remote.name)?,
        url: accepted("remote URL", &without_credentials(&remote.url))?,
    })
}

pub fn accepted<'a, T: TryFrom<&'a str, Error = ConversionError>>(field: &str, value: &'a str) -> Result<T> {
    T::try_from(value).map_err(|error| anyhow!("Mootbox cannot accept the {field} {value:?} ({error})."))
}

pub fn local_key(root: &Path) -> String {
    Sha256::digest(root.as_os_str().as_encoded_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .concat()
}

pub fn without_credentials(remote_url: &str) -> String {
    let Ok(mut parsed) = Url::parse(remote_url) else {
        return remote_url.to_string();
    };
    if parsed.password().is_none() && !CREDENTIAL_SCHEMES.contains(&parsed.scheme()) {
        return remote_url.to_string();
    }
    let _ = parsed.set_password(None);
    if CREDENTIAL_SCHEMES.contains(&parsed.scheme()) {
        let _ = parsed.set_username("");
    }
    parsed.to_string()
}

fn identifying_remote(remotes: &[git::Remote], console: &dyn Console) -> Result<git::Remote> {
    if remotes.is_empty() {
        bail!(NO_REMOTE);
    }
    if let Some(origin) = remotes.iter().find(|remote| remote.name == ORIGIN) {
        return Ok(origin.clone());
    }
    if let [only] = remotes {
        return Ok(only.clone());
    }
    let choices: Vec<String> = remotes
        .iter()
        .map(|remote| format!("{}  {}", remote.name, without_credentials(&remote.url)))
        .collect();
    Ok(remotes[console.choose(WHICH_REMOTE, &choices)?].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_never_leave_in_a_remote_url() {
        assert_eq!(
            without_credentials("https://user:token@github.com/acme/frontend.git"),
            "https://github.com/acme/frontend.git"
        );
        assert_eq!(
            without_credentials("ssh://git@github.com/acme/frontend"),
            "ssh://git@github.com/acme/frontend"
        );
        assert_eq!(
            without_credentials("git@github.com:Acme/Frontend.git"),
            "git@github.com:Acme/Frontend.git"
        );
    }

    #[test]
    fn the_local_key_is_the_sha256_of_the_path() {
        assert_eq!(
            local_key(Path::new("/work/checkout")),
            "b0413587c248573032bbc860a95ae824bb5b73648733786531d2995e1d08c832"
        );
    }
}
