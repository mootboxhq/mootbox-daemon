use std::path::PathBuf;

use anyhow::{Context, Result};
use reqwest::Url;
use serde_json::json;
use tempfile::TempDir;

use crate::private_files;
use crate::providers::MOOTBOX_MCP_SERVER_NAME;

const MCP_CONFIG_FILE: &str = "mcp.json";
const DIRECTORY_PREFIX: &str = "mootbox-run-";

pub fn write(mootbox_mcp_url: &Url, run_token: &str) -> Result<(TempDir, PathBuf)> {
    let directory = tempfile::Builder::new()
        .prefix(DIRECTORY_PREFIX)
        .tempdir()
        .context("cannot create the MCP config directory")?;
    let path = directory.path().join(MCP_CONFIG_FILE);
    private_files::write_owner_only(&path, contents(mootbox_mcp_url, run_token).as_bytes())?;
    Ok((directory, path))
}

fn contents(mootbox_mcp_url: &Url, run_token: &str) -> String {
    json!({
        "mcpServers": {
            MOOTBOX_MCP_SERVER_NAME: {
                "type": "http",
                "url": mootbox_mcp_url.as_str(),
                "headers": { "Authorization": format!("Bearer {run_token}") },
            }
        }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use super::*;

    const URL: &str = "http://localhost:3000/mcp";
    const TOKEN: &str = "mootbox_run_secret";

    fn read(path: &Path) -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(path).expect("readable")).expect("JSON")
    }

    #[test]
    fn the_private_config_file_names_the_mootbox_server_with_the_run_token() {
        let (_directory, path) = write(&Url::parse(URL).unwrap(), TOKEN).expect("writes");

        let parsed = read(&path);
        assert_eq!(parsed["mcpServers"][MOOTBOX_MCP_SERVER_NAME]["url"], URL);
        assert_eq!(
            parsed["mcpServers"][MOOTBOX_MCP_SERVER_NAME]["headers"]["Authorization"],
            format!("Bearer {TOKEN}")
        );
    }

    #[test]
    fn the_written_config_is_readable_only_by_its_owner_and_removed_with_its_directory() {
        let (directory, path) = write(&Url::parse(URL).unwrap(), TOKEN).expect("writes");

        #[cfg(unix)]
        {
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        drop(directory);
        assert!(!path.exists());
    }
}
