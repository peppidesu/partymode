use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, default};
use thiserror::Error;

/**
Contains the error that occurred while loading the config file.
*/
#[derive(Debug, Error)]
pub enum LoadConfigError {
    /// The config file failed to parse.
    #[error("Error while reading config: {0}")]
    Parse(#[from] toml::de::Error),
    /// The config file could not be read.
    #[error("Could not open config file: {0}")]
    IO(#[from] std::io::Error),
}

/**
The configuration for partymode. Loaded from a config file at the default or user-specified location.
*/
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub default_enabled: bool,
    pub poll_interval: u64,
    #[serde(rename = "*")]
    pub default_rule: Rule,
    #[serde(flatten)]
    pub rules: HashMap<String, Rule>,
}

/**
Inhibit behavior rule applied to one or all players.
*/
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Rule {
    /// Apply regardless of whether partymode is currently on or off.
    #[serde(default = "rule_defaults::always")]
    pub always: bool,
    /// Inhibit mode to use.
    pub mode: Option<InhibitMode>,
    /// What to inhibit.
    pub targets: Vec<InhibitTarget>,
}
mod rule_defaults {
    pub fn always() -> bool {
        true
    }
}

impl Rule {
    pub fn targets_str(&self) -> Option<String> {
        if self.targets.is_empty() {
            return None;
        }

        Some(
            self.targets
                .iter()
                .map(|t| match t {
                    InhibitTarget::Idle => "idle",
                    InhibitTarget::Sleep => "sleep",
                    InhibitTarget::Shutdown => "shutdown",
                })
                .join(":"),
        )
    }

    pub fn with_defaults(&self, defaults: &Rule) -> Rule {
        Rule {
            always: self.always,
            mode: self.mode.clone().or(defaults.mode.clone()),
            targets: if self.targets.is_empty() {
                defaults.targets.clone()
            } else {
                self.targets.clone()
            },
        }
    }
}

/**
Inhibit mode as specified by [systemd-inhibit(1)](https://www.freedesktop.org/software/systemd/man/latest/systemd-inhibit.html#--mode=).
*/
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum InhibitMode {
    Block,
    Delay,
    BlockWeak,
}
impl ToString for InhibitMode {
    fn to_string(&self) -> String {
        match self {
            InhibitMode::Block => "block".to_string(),
            InhibitMode::Delay => "delay".to_string(),
            InhibitMode::BlockWeak => "block_weak".to_string(),
        }
    }
}
/**
Inhibit target as specified by [systemd-inhibit(1)](https://www.freedesktop.org/software/systemd/man/latest/systemd-inhibit.html#--what=).
*/
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum InhibitTarget {
    Idle,
    Sleep,
    Shutdown,
}

impl Config {
    pub fn load<P>(path: P) -> Result<Self, LoadConfigError>
    where
        P: AsRef<std::path::Path>,
    {
        let contents = std::fs::read_to_string(path)?;
        let result: Self = toml::from_str(&contents)?;
        Ok(result)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_enabled: true,
            default_rule: Rule {
                always: false,
                mode: Some(InhibitMode::Block),
                targets: vec![InhibitTarget::Idle],
            },
            poll_interval: 5000,
            rules: HashMap::new(),
        }
    }
}
