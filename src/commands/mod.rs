use anyhow::Result;
use reqwest::Url;

pub mod add;
pub mod allow;
pub mod doctor;
pub mod login;
pub mod pair;
pub mod service;
pub mod start;
pub mod status;
pub mod update;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Output {
    pub lines: Vec<String>,
    pub warnings: Vec<String>,
}

impl Output {
    pub fn lines(lines: Vec<String>) -> Self {
        Self {
            lines,
            warnings: Vec::new(),
        }
    }
}

pub trait Console {
    fn choose(&self, question: &str, choices: &[String]) -> Result<usize>;
    fn ask(&self, question: &str, suggestion: &str) -> Result<String>;
    fn open_link(&self, prompt: &str, link: &Url);
    fn show(&self, line: &str);
    fn warn(&self, line: &str);
}
