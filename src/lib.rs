mod cli;
mod config;
mod daemon;
mod socket;
mod state;

use std::path::PathBuf;
use std::sync::LazyLock;

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

static XDG_RUNTIME_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::var("XDG_RUNTIME_DIR")
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
});

static PARTYMODE_RUNTIME_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| XDG_RUNTIME_DIR.join("partymode"));

static SOCKET_PATH: LazyLock<PathBuf> = LazyLock::new(|| PARTYMODE_RUNTIME_DIR.join("sock"));

static PID_LOCK_PATH: LazyLock<PathBuf> = LazyLock::new(|| PARTYMODE_RUNTIME_DIR.join("pid.lock"));
