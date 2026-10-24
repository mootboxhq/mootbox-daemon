use std::io::{BufRead, IsTerminal, Write};
#[cfg(unix)]
use std::process::Command;
use std::process::Stdio;

use anyhow::{Result, bail};
use reqwest::Url;

use crate::commands::Console;
use crate::commands::add::PROJECT_HINT;

#[cfg(unix)]
const BROWSER_OPENER: &str = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };

pub struct TerminalConsole;

impl Console for TerminalConsole {
    fn choose(&self, question: &str, choices: &[String]) -> Result<usize> {
        let listing = numbered(question, choices);
        if !std::io::stdin().is_terminal() {
            bail!("{listing}\n{PROJECT_HINT}");
        }
        eprintln!("{listing}");
        loop {
            let answer = read_answer(&format!("Choose 1-{}: ", choices.len()))?;
            if let Some(number) = answer
                .parse::<usize>()
                .ok()
                .filter(|number| (1..=choices.len()).contains(number))
            {
                return Ok(number - 1);
            }
        }
    }

    fn ask(&self, question: &str, suggestion: &str) -> Result<String> {
        if !std::io::stdin().is_terminal() {
            bail!("{question}\n{PROJECT_HINT}");
        }
        let answer = read_answer(&format!("{question} [{suggestion}]: "))?;
        if answer.is_empty() {
            return Ok(suggestion.to_string());
        }
        Ok(answer)
    }

    fn open_link(&self, prompt: &str, link: &Url) {
        eprintln!("{prompt}: {link}");
        #[cfg(unix)]
        let mut opener = {
            let mut command = Command::new(BROWSER_OPENER);
            command.arg(link.as_str());
            command
        };
        #[cfg(windows)]
        let mut opener = {
            let mut command = crate::platform::powershell("Start-Process -FilePath $env:MOOTBOX_CONFIRM_URL");
            command.env("MOOTBOX_CONFIRM_URL", link.as_str());
            command
        };
        let _ = opener
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    fn show(&self, line: &str) {
        println!("{line}");
    }

    fn warn(&self, line: &str) {
        eprintln!("{line}");
    }
}

fn numbered(question: &str, choices: &[String]) -> String {
    let mut lines = vec![question.to_string()];
    lines.extend(
        choices
            .iter()
            .enumerate()
            .map(|(index, choice)| format!("  {}. {choice}", index + 1)),
    );
    lines.join("\n")
}

fn read_answer(prompt: &str) -> Result<String> {
    eprint!("{prompt}");
    std::io::stderr().flush()?;
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line)? == 0 {
        bail!("no answer given; {PROJECT_HINT}");
    }
    Ok(line.trim().to_string())
}
