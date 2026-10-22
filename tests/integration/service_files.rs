use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use mootbox_daemon::config::HOME_OVERRIDE_VARIABLE;
use mootbox_daemon::service::{self, ServiceDefinition};

pub fn definition(home_override: Option<&str>, shutdown_grace_seconds: u64) -> ServiceDefinition {
    ServiceDefinition {
        executable: PathBuf::from("/opt/mootbox/bin/mootbox"),
        home: PathBuf::from("/Users/dev/.mootbox"),
        home_override: home_override.map(PathBuf::from),
        search_path: "/opt/homebrew/bin:/usr/bin:/bin".to_string(),
        shutdown_grace_seconds,
    }
}

#[test]
fn the_launchd_plist_starts_the_daemon_at_login_and_restarts_it_only_after_a_failure() {
    let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>dev.mootbox.daemon</string>
    <key>ProgramArguments</key>
    <array>
      <string>/opt/mootbox/bin/mootbox</string>
      <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
      <key>SuccessfulExit</key>
      <false/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>10</integer>
    <key>ExitTimeOut</key>
    <integer>75</integer>
    <key>StandardOutPath</key>
    <string>/Users/dev/.mootbox/logs/service.out.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/dev/.mootbox/logs/service.err.log</string>
    <key>EnvironmentVariables</key>
    <dict>
      <key>PATH</key>
      <string>/opt/homebrew/bin:/usr/bin:/bin</string>
      <key>MOOTBOX_DAEMON_HOME</key>
      <string>/Users/dev/.mootbox</string>
    </dict>
  </dict>
</plist>
"#;

    assert_eq!(
        service::launchd_plist(&definition(Some("/Users/dev/.mootbox"), 30)),
        expected
    );
}

#[test]
fn the_launchd_plist_escapes_xml_and_omits_the_home_when_it_is_the_default() {
    let mut definition = definition(None, 120);
    definition.search_path = "/tools/R&D<1>:/bin".to_string();

    let plist = service::launchd_plist(&definition);

    assert!(
        plist.contains("<string>/tools/R&amp;D&lt;1&gt;:/bin</string>"),
        "{plist}"
    );
    assert!(
        plist.contains("<key>ExitTimeOut</key>\n    <integer>165</integer>"),
        "{plist}"
    );
    assert!(!plist.contains(HOME_OVERRIDE_VARIABLE), "{plist}");
}

#[test]
fn the_systemd_unit_lets_the_daemon_handle_sigterm_before_the_stop_timeout() {
    let expected = r#"[Unit]
Description=Mootbox daemon

[Service]
ExecStart="/opt/mootbox/bin/mootbox" start
Restart=on-failure
RestartSec=10
KillMode=mixed
TimeoutStopSec=105
Environment="PATH=/opt/homebrew/bin:/usr/bin:/bin"
Environment="MOOTBOX_DAEMON_HOME=/srv/mootbox home"

[Install]
WantedBy=default.target
"#;

    assert_eq!(
        service::systemd_unit(&definition(Some("/srv/mootbox home"), 60)),
        expected
    );
}

#[test]
fn the_systemd_unit_escapes_quotes_specifiers_and_dollars() {
    let mut definition = definition(None, 30);
    definition.executable = PathBuf::from("/opt/50%/$bin/mootbox");
    definition.search_path = "/a \"b\"/bin".to_string();

    let unit = service::systemd_unit(&definition);

    assert!(unit.contains("ExecStart=\"/opt/50%%/$$bin/mootbox\" start\n"), "{unit}");
    assert!(unit.contains("Environment=\"PATH=/a \\\"b\\\"/bin\"\n"), "{unit}");
    assert!(!unit.contains(HOME_OVERRIDE_VARIABLE), "{unit}");
}

fn executable_at(path: &Path) {
    fs::create_dir_all(path.parent().expect("has a parent")).expect("creates directory");
    fs::write(path, "#!/bin/sh\n").expect("writes executable");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("executable");
}

#[test]
fn install_prefers_the_path_entry_that_resolves_to_this_executable() {
    let root = tempfile::tempdir().expect("temp dir");
    let cellar = root.path().join("Cellar/mootbox/0.1.0/bin/mootbox");
    executable_at(&cellar);
    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).expect("creates bin");
    std::os::unix::fs::symlink(&cellar, bin.join("mootbox")).expect("links");
    let search_path = format!("/nonexistent:{}:/usr/bin", bin.display());

    assert_eq!(service::stable_executable(&cellar, &search_path), bin.join("mootbox"));
}

#[test]
fn install_keeps_this_executable_when_path_finds_another_or_none() {
    let root = tempfile::tempdir().expect("temp dir");
    let current = root.path().join("target/release/mootbox");
    executable_at(&current);
    let other = root.path().join("bin/mootbox");
    executable_at(&other);

    let other_on_path = root.path().join("bin").display().to_string();
    assert_eq!(service::stable_executable(&current, &other_on_path), current);
    assert_eq!(service::stable_executable(&current, "/nonexistent"), current);
}
