mod cli;
mod config;
mod daemon;
mod dbus;
mod state;

use std::path::PathBuf;

pub use cli::Args;
pub use cli::parse;
pub use cli::run;
use config::Config;

pub fn default_config_path() -> PathBuf {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|_| {
            let home = PathBuf::from(std::env::var("HOME").unwrap());
            home.join(".config")
        });
    let path = config_dir.join("partymode").join("config.toml");

    if !path.parent().unwrap().exists() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    }
    if !path.exists() {
        let contents = toml::to_string(&Config::default()).unwrap();
        std::fs::write(&path, contents).ok();
    }

    path
}
