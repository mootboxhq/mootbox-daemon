use std::fs;
use std::path::{Path, PathBuf};

use nix::unistd::Pid;
use serde_json::Value;

use crate::harness::wait_until;

const INVOCATIONS_FILE: &str = "invocations";
const FAKE_CLAUDE: &str = "tests/fixtures/fake-claude.sh";
const FAKE_CODEX: &str = "tests/fixtures/fake-codex.sh";
const RELEASE_FILE: &str = "release";
const SCENARIO_FILE: &str = "scenario";

pub struct FakeProvider {
    directory: tempfile::TempDir,
    pub program: PathBuf,
}

impl FakeProvider {
    pub fn claude(scenario: &str) -> Self {
        Self::scripted(FAKE_CLAUDE, scenario)
    }

    pub fn codex(scenario: &str) -> Self {
        Self::scripted(FAKE_CODEX, scenario)
    }

    fn scripted(fixture: &str, scenario: &str) -> Self {
        let directory = tempfile::tempdir().expect("temp dir");
        let program = directory.path().join("provider");
        std::os::unix::fs::symlink(Path::new(env!("CARGO_MANIFEST_DIR")).join(fixture), &program)
            .expect("links the fake provider");
        fs::write(directory.path().join(SCENARIO_FILE), scenario).expect("writes the scenario");
        Self { directory, program }
    }

    pub fn release(&self) {
        fs::write(self.directory.path().join(RELEASE_FILE), "").expect("releases the provider");
    }

    pub fn invocations(&self) -> Vec<Invocation> {
        let order = fs::read_to_string(self.directory.path().join(INVOCATIONS_FILE)).unwrap_or_default();
        order
            .lines()
            .map(|pid| Invocation {
                arguments: self.read(pid, "args").lines().map(str::to_string).collect(),
                prompt: self.read(pid, "prompt"),
                working_directory: PathBuf::from(self.read(pid, "cwd").trim()),
                mcp_config_path: PathBuf::from(self.read(pid, "mcp-path").trim()),
                mcp_config_mode: self.read(pid, "mcp-mode").trim().to_string(),
                mcp_config: serde_json::from_str(&self.read(pid, "mcp-config")).unwrap_or(Value::Null),
                run_token: self.read(pid, "run-token"),
            })
            .collect()
    }

    pub fn only_invocation(&self) -> Invocation {
        let mut invocations = self.invocations();
        assert_eq!(invocations.len(), 1, "expected one provider invocation");
        invocations.remove(0)
    }

    fn read(&self, pid: &str, extension: &str) -> String {
        fs::read_to_string(self.directory.path().join(format!("{pid}.{extension}"))).unwrap_or_default()
    }

    pub async fn grandchild(&self) -> Pid {
        wait_until(|| async {
            self.captured("grandchild")
                .first()
                .and_then(|text| text.trim().parse().ok())
                .map(Pid::from_raw)
        })
        .await
    }

    fn captured(&self, extension: &str) -> Vec<String> {
        fs::read_dir(self.directory.path())
            .expect("capture dir")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|found| found == extension))
            .filter_map(|path| fs::read_to_string(path).ok())
            .collect()
    }
}

#[derive(Debug)]
pub struct Invocation {
    pub arguments: Vec<String>,
    pub prompt: String,
    pub working_directory: PathBuf,
    pub mcp_config_path: PathBuf,
    pub mcp_config_mode: String,
    pub mcp_config: Value,
    pub run_token: String,
}

impl Invocation {
    pub fn value_after(&self, flag: &str) -> Option<&str> {
        let position = self.arguments.iter().position(|argument| argument == flag)?;
        self.arguments.get(position + 1).map(String::as_str)
    }

    pub fn allowed_tools(&self) -> Vec<String> {
        let settings: Value = serde_json::from_str(self.value_after("--settings").expect("--settings")).expect("JSON");
        serde_json::from_value(settings["permissions"]["allow"].clone()).expect("tool list")
    }
}
