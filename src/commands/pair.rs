use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use reqwest::Url;

use super::{Console, Output};
use crate::config;
use crate::pairing::{self, PairingRequest};

pub struct PairOptions {
    pub mootbox_url: Url,
    pub device_name: String,
    pub poll_interval: Duration,
}

pub async fn run(home: &Path, options: &PairOptions, console: &dyn Console) -> Result<Output> {
    let request = PairingRequest {
        mootbox_url: &options.mootbox_url,
        device_name: &options.device_name,
        poll_interval: options.poll_interval,
    };
    let profile = pairing::pair_device(&request, |started| {
        console.show(&format!(
            "enter code {} on the Devices page of the Mootbox dashboard for {} (expires {})",
            started.user_code, options.mootbox_url, started.expires_at
        ));
        Ok(())
    })
    .await?;
    let runner_id = profile.runner_id;
    let mut config = config::load_or_start_fresh(home)?;
    config.upsert_workspace(profile);
    config::save(home, &config)?;
    Ok(Output::lines(vec![format!(
        "paired as runner {runner_id}; credential stored in {}",
        config::config_path(home).display()
    )]))
}
