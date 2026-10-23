use std::time::Duration;

use anyhow::Result;
use reqwest::Url;

use crate::api::MootboxClient;
use crate::config::WorkspaceProfile;
use crate::contracts::{
    ClaimPairingCodeRequest, ExchangePairingRequest, ExchangePairingResponse, StartPairingRequest, StartPairingResponse,
};
use crate::device::DeviceReport;

pub const PAIRING_POLL_INTERVAL: Duration = Duration::from_secs(2);

pub struct PairingRequest<'a> {
    pub mootbox_url: &'a Url,
    pub device_name: &'a str,
    pub poll_interval: Duration,
}

pub async fn pair_device(
    request: &PairingRequest<'_>,
    announce: impl FnOnce(&StartPairingResponse) -> Result<()>,
) -> Result<WorkspaceProfile> {
    let client = MootboxClient::unauthenticated(request.mootbox_url.clone());
    let start = StartPairingRequest {
        name: request.device_name.try_into()?,
    };
    let started = client.start_pairing(&start).await?;
    announce(&started)?;
    let exchange = ExchangePairingRequest {
        device_secret: started.device_secret.as_str().try_into()?,
    };
    loop {
        tokio::time::sleep(request.poll_interval).await;
        if let ExchangePairingResponse::Paired {
            runner_id,
            credential,
            workspace_id,
            workspace_name,
        } = client.exchange_pairing(started.pairing_id, &exchange).await?
        {
            return Ok(WorkspaceProfile {
                mootbox_url: request.mootbox_url.to_string(),
                workspace_id,
                runner_id,
                credential,
                workspace_name: Some(workspace_name),
            });
        }
    }
}

pub struct CodeClaim<'a> {
    pub mootbox_url: &'a Url,
    pub device_name: &'a str,
    pub user_code: &'a str,
    pub device: &'a DeviceReport,
}

pub async fn claim_pairing_code(claim: &CodeClaim<'_>) -> Result<WorkspaceProfile> {
    let client = MootboxClient::unauthenticated(claim.mootbox_url.clone());
    let request = ClaimPairingCodeRequest {
        user_code: claim.user_code.try_into()?,
        name: claim.device_name.try_into()?,
        device: claim.device.contract()?,
    };
    let paired = client.claim_pairing_code(&request).await?;
    Ok(WorkspaceProfile {
        mootbox_url: claim.mootbox_url.to_string(),
        workspace_id: paired.workspace_id,
        runner_id: paired.runner_id,
        credential: paired.credential,
        workspace_name: Some(paired.workspace_name),
    })
}
