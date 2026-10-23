use std::io::{BufRead, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use reqwest::Url;

use super::{Console, Output};
use crate::config::{self, WorkspaceProfile};
use crate::device::{self, DeviceReport};
use crate::pairing::{self, CodeClaim, PairingRequest};

pub const PAIRING_CODE_PROMPT: &str = "Pairing code (leave empty to get one to enter on the web): ";

pub struct PairOptions {
    pub mootbox_url: Url,
    pub device_name: String,
    pub code: Option<String>,
    pub poll_interval: Duration,
}

pub fn read_pairing_code(input: &mut impl BufRead, prompt: &mut impl Write) -> Result<Option<String>> {
    write!(prompt, "{PAIRING_CODE_PROMPT}")?;
    prompt.flush()?;
    let mut line = String::new();
    input.read_line(&mut line)?;
    let code = line.trim();
    Ok((!code.is_empty()).then(|| code.to_string()))
}

pub async fn run(home: &Path, options: &PairOptions, console: &dyn Console) -> Result<Output> {
    let profile = match &options.code {
        Some(code) => claim_code(options, code, &device::report_device(device::default_program).await).await?,
        None => pair_from_daemon(options, console).await?,
    };
    let runner_id = profile.runner_id;
    let mut config = config::load_or_start_fresh(home)?;
    config.upsert_workspace(profile);
    config::save(home, &config)?;
    Ok(Output::lines(vec![format!(
        "paired as runner {runner_id}; credential stored in {}",
        config::config_path(home).display()
    )]))
}

async fn claim_code(options: &PairOptions, code: &str, device: &DeviceReport) -> Result<WorkspaceProfile> {
    pairing::claim_pairing_code(&CodeClaim {
        mootbox_url: &options.mootbox_url,
        device_name: &options.device_name,
        user_code: code,
        device,
    })
    .await
}

async fn pair_from_daemon(options: &PairOptions, console: &dyn Console) -> Result<WorkspaceProfile> {
    let request = PairingRequest {
        mootbox_url: &options.mootbox_url,
        device_name: &options.device_name,
        poll_interval: options.poll_interval,
    };
    pairing::pair_device(&request, |started| {
        console.show(&format!(
            "enter code {} on the Devices page of the Mootbox dashboard for {} (expires {})",
            started.user_code, options.mootbox_url, started.expires_at
        ));
        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer(typed: &str) -> (Option<String>, String) {
        let mut prompt = Vec::new();
        let code = read_pairing_code(&mut typed.as_bytes(), &mut prompt).expect("reads");
        (code, String::from_utf8(prompt).expect("UTF-8"))
    }

    #[test]
    fn a_typed_code_is_trimmed_and_used() {
        let (code, prompt) = answer("  abcd-efgh \n");

        assert_eq!(code.as_deref(), Some("abcd-efgh"));
        assert_eq!(prompt, PAIRING_CODE_PROMPT);
    }

    #[test]
    fn an_empty_answer_falls_back_to_the_daemon_first_flow() {
        assert_eq!(answer("\n").0, None);
        assert_eq!(answer("").0, None);
    }
}
