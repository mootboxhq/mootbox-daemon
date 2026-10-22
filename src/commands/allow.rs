use std::path::Path;

use anyhow::{Result, bail};

use super::Output;
use crate::config;

const PUSH_WORD: &str = "push";
const SHELL_RULE_PREFIX: &str = "Bash(";
const SHELL_RULE_SUFFIX: &str = ")";
const SHELL_RULE_FORBIDDEN: [char; 5] = ['(', ')', '&', '|', ';'];
const PUSH_WARNING: &str = "warning: this permits pushing; Mootbox cannot fence or undo external pushes";

pub fn run(home: &Path, rule: &str) -> Result<Output> {
    if !is_shell_rule(rule) {
        bail!("expected a single command rule like \"Bash(pnpm test:*)\", got {rule:?}");
    }
    let mut config = config::load(home)?;
    if !config.allowed_commands.iter().any(|allowed| allowed == rule) {
        config.allowed_commands.push(rule.to_string());
    }
    config::save(home, &config)?;
    let mut output = Output::lines(vec![format!(
        "managed runs may now use: {}",
        config.allowed_commands.join(", ")
    )]);
    if rule.contains(PUSH_WORD) {
        output.warnings.push(PUSH_WARNING.to_string());
    }
    Ok(output)
}

fn is_shell_rule(rule: &str) -> bool {
    let Some(command) = rule
        .strip_prefix(SHELL_RULE_PREFIX)
        .and_then(|rest| rest.strip_suffix(SHELL_RULE_SUFFIX))
    else {
        return false;
    };
    !command.is_empty() && !command.contains(SHELL_RULE_FORBIDDEN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_rules_are_single_bash_commands() {
        for accepted in ["Bash(pnpm test:*)", "Bash(git status)", "Bash(cargo test --workspace)"] {
            assert!(is_shell_rule(accepted), "{accepted}");
        }
        for rejected in [
            "Bash()",
            "Bash(a && b)",
            "Bash(a | b)",
            "Bash(a; b)",
            "Bash((a))",
            "Bash(a",
            "bash(a)",
            "Read(file)",
            " Bash(a)",
            "Bash(a) ",
        ] {
            assert!(!is_shell_rule(rejected), "{rejected}");
        }
    }
}
