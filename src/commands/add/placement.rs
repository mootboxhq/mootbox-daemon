use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Result, anyhow, bail};
use uuid::Uuid;

use super::checkout::{LocalCheckout, accepted, matched_remote, without_credentials};
use super::{Adding, NEW_PROJECT, NEW_PROJECT_NAME, NO_AUTHORITY, WHICH_PROJECT, WHICH_TEAM, server_message};
use crate::api::MootboxClient;
use crate::config::DaemonConfig;
use crate::contracts::{
    MatchRepositoriesRequest, MatchRepositoriesResponse, MatchRepositoriesResponseMatchesItem,
    MatchRepositoriesResponseOneMatch, MatchRepositoriesResponseRegistrableItem as RegistrableTeam,
    MatchRepositoriesResponseRegistrableItemProjectsItem as RegistrableProject, RegisterRepositoryRequest,
    RegisterRepositoryRequestNewProject, UpsertConnectionRequest,
};

pub struct Placement {
    pub repository_id: Uuid,
    pub repository_name: String,
    pub project_id: Uuid,
    pub project_name: String,
    pub team_name: String,
}

#[derive(Clone, Copy)]
pub struct Attachment {
    pub repository_id: Uuid,
    pub project_id: Uuid,
}

impl Placement {
    pub fn attachment(&self) -> Attachment {
        Attachment {
            repository_id: self.repository_id,
            project_id: self.project_id,
        }
    }
}

struct RegistrationTarget {
    team_name: String,
    project_name: String,
    project_id: Option<Uuid>,
    new_project: Option<RegisterRepositoryRequestNewProject>,
}

impl RegistrationTarget {
    fn existing(team: &RegistrableTeam, project: &RegistrableProject) -> Self {
        Self {
            team_name: team.team_name.clone(),
            project_name: project.name.clone(),
            project_id: Some(project.id),
            new_project: None,
        }
    }

    fn new_project(team: &RegistrableTeam, name: String) -> Result<Self> {
        let new_project = RegisterRepositoryRequestNewProject {
            team_id: team.team_id,
            name: accepted("project name", &name)?,
        };
        Ok(Self {
            team_name: team.team_name.clone(),
            project_name: name,
            project_id: None,
            new_project: Some(new_project),
        })
    }
}

impl From<MatchRepositoriesResponseOneMatch> for Placement {
    fn from(found: MatchRepositoriesResponseOneMatch) -> Self {
        Self {
            repository_id: found.repository_id,
            repository_name: found.repository_name,
            project_id: found.project.id,
            project_name: found.project.name,
            team_name: found.team.name,
        }
    }
}

impl From<MatchRepositoriesResponseMatchesItem> for Placement {
    fn from(found: MatchRepositoriesResponseMatchesItem) -> Self {
        Self {
            repository_id: found.repository_id,
            repository_name: found.repository_name,
            project_id: found.project.id,
            project_name: found.project.name,
            team_name: found.team.name,
        }
    }
}

impl Adding<'_> {
    pub(super) async fn placement(
        &self,
        client: &MootboxClient,
        checkout: &LocalCheckout,
        remembered: &BTreeSet<Uuid>,
    ) -> Result<Placement> {
        let request = MatchRepositoriesRequest {
            remotes: checkout.remotes.iter().map(matched_remote).collect::<Result<_>>()?,
        };
        match client.match_repository(&request).await.map_err(server_message)? {
            MatchRepositoriesResponse::One { match_ } => self.matched(vec![Placement::from(*match_)], remembered),
            MatchRepositoriesResponse::Many { matches } => {
                self.matched(matches.into_iter().map(Placement::from).collect(), remembered)
            }
            MatchRepositoriesResponse::None { registrable } => self.register(client, checkout, &registrable).await,
        }
    }

    fn matched(&self, mut matches: Vec<Placement>, remembered: &BTreeSet<Uuid>) -> Result<Placement> {
        let index = self.matched_index(&matches, remembered)?;
        Ok(matches.swap_remove(index))
    }

    fn matched_index(&self, matches: &[Placement], remembered: &BTreeSet<Uuid>) -> Result<usize> {
        if let Some(project_id) = self.options.project {
            return matches
                .iter()
                .position(|found| found.project_id == project_id)
                .ok_or_else(|| anyhow!(not_offered(project_id)));
        }
        if matches.len() == 1 {
            return Ok(0);
        }
        if let Some(index) = matches.iter().position(|found| remembered.contains(&found.project_id)) {
            return Ok(index);
        }
        let choices: Vec<String> = matches
            .iter()
            .map(|found| format!("{} / {}", found.team_name, found.project_name))
            .collect();
        self.console.choose(WHICH_PROJECT, &choices)
    }

    async fn register(
        &self,
        client: &MootboxClient,
        checkout: &LocalCheckout,
        registrable: &[RegistrableTeam],
    ) -> Result<Placement> {
        if registrable.is_empty() {
            bail!(NO_AUTHORITY);
        }
        let target = self.registration_target(registrable, &checkout.name)?;
        let request = RegisterRepositoryRequest {
            remote_url: accepted("remote URL", &without_credentials(&checkout.remote.url))?,
            name: accepted("repository name", &checkout.name)?,
            default_branch: accepted("default branch", &checkout.default_branch)?,
            project_id: target.project_id,
            new_project: target.new_project,
        };
        let registered = client.register_repository(&request).await.map_err(server_message)?;
        Ok(Placement {
            repository_id: registered.repository_id,
            repository_name: checkout.name.clone(),
            project_id: registered.project_id,
            project_name: target.project_name,
            team_name: target.team_name,
        })
    }

    fn registration_target(&self, registrable: &[RegistrableTeam], folder_name: &str) -> Result<RegistrationTarget> {
        if let Some(project_id) = self.options.project {
            return flagged_target(registrable, project_id);
        }
        let team = self.chosen_team(registrable)?;
        let mut choices: Vec<String> = team.projects.iter().map(|project| project.name.clone()).collect();
        choices.push(NEW_PROJECT.to_string());
        let question = format!("Which {} project should this repository join?", team.team_name);
        let picked = self.console.choose(&question, &choices)?;
        if let Some(project) = team.projects.get(picked) {
            return Ok(RegistrationTarget::existing(team, project));
        }
        let name = self.console.ask(NEW_PROJECT_NAME, folder_name)?;
        RegistrationTarget::new_project(team, name)
    }

    fn chosen_team<'r>(&self, registrable: &'r [RegistrableTeam]) -> Result<&'r RegistrableTeam> {
        if let [only] = registrable {
            return Ok(only);
        }
        let choices: Vec<String> = registrable.iter().map(|team| team.team_name.clone()).collect();
        Ok(&registrable[self.console.choose(WHICH_TEAM, &choices)?])
    }
}

pub async fn connect_checkout(
    client: &MootboxClient,
    checkout: &LocalCheckout,
    attachment: Attachment,
) -> Result<Uuid> {
    let request = connection_request(checkout, attachment)?;
    let connected = client.upsert_connection(&request).await.map_err(server_message)?;
    Ok(connected.connection_id)
}

pub fn connection_request(checkout: &LocalCheckout, attachment: Attachment) -> Result<UpsertConnectionRequest> {
    Ok(UpsertConnectionRequest {
        repository_id: attachment.repository_id,
        project_id: attachment.project_id,
        local_key: accepted("local key", &checkout.local_key())?,
        branch: checkout.branch.clone(),
        head_sha: checkout.head_sha.clone(),
        dirty: checkout.is_dirty,
    })
}

pub fn remembered_projects(config: &DaemonConfig, workspace_id: Uuid, root: &Path) -> BTreeSet<Uuid> {
    config
        .connections
        .values()
        .filter(|connection| connection.workspace_id == workspace_id && connection.path == root)
        .flat_map(|connection| connection.project_ids.iter().copied())
        .collect()
}

fn flagged_target(registrable: &[RegistrableTeam], project_id: Uuid) -> Result<RegistrationTarget> {
    registrable
        .iter()
        .find_map(|team| {
            team.projects
                .iter()
                .find(|project| project.id == project_id)
                .map(|project| RegistrationTarget::existing(team, project))
        })
        .ok_or_else(|| anyhow!(not_offered(project_id)))
}

fn not_offered(project_id: Uuid) -> String {
    format!("Project {project_id} is not one you can connect this repository to.")
}
