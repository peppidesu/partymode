use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/**
Contains the error that occurred while loading the config file.
*/

#[derive(Debug)]
pub enum LoadConfigError {
    /// The config file failed to parse.
    Parse(toml::de::Error),
    /// The config file could not be read.
    IO(std::io::Error),
}

impl From<toml::de::Error> for LoadConfigError {
    fn from(value: toml::de::Error) -> Self {
        LoadConfigError::Parse(value)
    }
}

impl From<std::io::Error> for LoadConfigError {
    fn from(value: std::io::Error) -> Self {
        LoadConfigError::IO(value)
    }
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
    always: bool,
    /// Inhibit mode to use.
    mode: InhibitMode,
    /// What to inhibit.
    targets: Option<Vec<InhibitTarget>>,
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
                mode: InhibitMode::Block,
                targets: Some(vec![InhibitTarget::Idle]),
            },
            poll_interval: 5000,
            rules: HashMap::new(),
        }
    }
}
