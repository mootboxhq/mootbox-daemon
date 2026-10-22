use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::contracts::ClaimRequestProvidersItem;
use crate::contracts::error::ConversionError;

use super::{ALL, DEFAULT_PROVIDER, Provider};

#[derive(Clone, Copy)]
pub struct ProviderKind(&'static dyn Provider);

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("unknown provider; expected one of {}", known_names())]
pub struct UnknownProvider;

impl ProviderKind {
    pub const fn new(provider: &'static dyn Provider) -> Self {
        Self(provider)
    }

    pub fn contract(self) -> Result<ClaimRequestProvidersItem, ConversionError> {
        self.name().parse()
    }
}

fn known_names() -> String {
    ALL.map(|kind| kind.name()).join(", ")
}

impl Deref for ProviderKind {
    type Target = dyn Provider;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Default for ProviderKind {
    fn default() -> Self {
        DEFAULT_PROVIDER
    }
}

impl PartialEq for ProviderKind {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for ProviderKind {}

impl fmt::Debug for ProviderKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

impl FromStr for ProviderKind {
    type Err = UnknownProvider;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        ALL.into_iter().find(|kind| kind.name() == text).ok_or(UnknownProvider)
    }
}

impl Serialize for ProviderKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for ProviderKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        name.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_provider_round_trips_through_its_name() {
        for kind in ALL {
            assert_eq!(kind.name().parse::<ProviderKind>(), Ok(kind));
        }
    }

    #[test]
    fn every_registered_provider_has_a_runner_protocol_name() {
        for kind in ALL {
            let contract = kind.contract().expect("the runner protocol knows the provider");
            assert_eq!(contract.to_string(), kind.name());
        }
    }

    #[test]
    fn each_provider_ships_a_distinct_name_command_and_label() {
        let names = ALL.map(|kind| kind.name());
        let commands = ALL.map(|kind| kind.default_command());
        let labels = ALL.map(|kind| kind.label());

        assert_eq!(names, ["claude-code", "codex"]);
        assert_eq!(commands, ["claude", "codex"]);
        assert_eq!(labels, ["Claude Code", "Codex"]);
    }

    #[test]
    fn an_unknown_provider_name_is_rejected_with_the_known_ones() {
        let error = "gemini".parse::<ProviderKind>().expect_err("unknown");

        assert_eq!(
            error.to_string(),
            "unknown provider; expected one of claude-code, codex"
        );
    }

    #[test]
    fn a_config_without_a_provider_keeps_using_claude_code() {
        assert_eq!(ProviderKind::default().name(), "claude-code");
        assert!(serde_json::from_str::<ProviderKind>("null").is_err());
    }

    #[test]
    fn the_config_writes_the_provider_under_its_name() {
        for kind in ALL {
            assert_eq!(serde_json::to_string(&kind).unwrap(), format!("\"{}\"", kind.name()));
            assert_eq!(
                serde_json::from_str::<ProviderKind>(&format!("\"{kind}\"")).unwrap(),
                kind
            );
        }
    }
}
