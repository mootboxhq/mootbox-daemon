#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

pub const EXECUTABLE_NAME: &str = if cfg!(windows) { "mootbox.exe" } else { "mootbox" };

pub fn provider_program(program: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        if program.extension().is_none() {
            let directories = if program.components().count() > 1 {
                vec![PathBuf::new()]
            } else {
                std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect()
            };
            for directory in directories {
                for extension in ["exe", "cmd", "bat"] {
                    let candidate = directory.join(program).with_extension(extension);
                    if candidate.is_file() {
                        return candidate;
                    }
                }
            }
        }
    }
    program.to_path_buf()
}

#[cfg(windows)]
pub fn powershell(script: &str) -> std::process::Command {
    let mut command = std::process::Command::new("powershell.exe");
    command.env_remove("PSModulePath");
    command.creation_flags(0x0800_0000);
    command.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script]);
    command
}

#[cfg(windows)]
pub fn restrict_to_owner(path: &Path) -> std::io::Result<()> {
    let output = powershell(
        "$ErrorActionPreference = 'Stop'; $sid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User; $directory = Test-Path -LiteralPath $env:MOOTBOX_PRIVATE_PATH -PathType Container; if ($directory) { $acl = New-Object System.Security.AccessControl.DirectorySecurity; $inherit = [System.Security.AccessControl.InheritanceFlags]'ContainerInherit,ObjectInherit' } else { $acl = New-Object System.Security.AccessControl.FileSecurity; $inherit = [System.Security.AccessControl.InheritanceFlags]::None }; $acl.SetOwner($sid); $acl.SetAccessRuleProtection($true, $false); $rule = New-Object System.Security.AccessControl.FileSystemAccessRule($sid, [System.Security.AccessControl.FileSystemRights]::FullControl, $inherit, [System.Security.AccessControl.PropagationFlags]::None, [System.Security.AccessControl.AccessControlType]::Allow); $acl.AddAccessRule($rule); Set-Acl -LiteralPath $env:MOOTBOX_PRIVATE_PATH -AclObject $acl",
    )
    .env("MOOTBOX_PRIVATE_PATH", path)
    .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "cannot protect {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}
