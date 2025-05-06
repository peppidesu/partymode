use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::SeqCst},
    },
    time::Duration,
};
use zbus::{Connection, zvariant::OwnedFd};

use tokio::{sync::RwLock, time::sleep};

use crate::{
    config::{Config, InhibitMode},
    dbus::{self, logind::LoginManagerProxy, partymode::Partymode},
};

#[derive(Clone)]
pub struct State {
    pub config: Arc<RwLock<Config>>,
    pub partymode: Arc<AtomicBool>,
    pub locks: Arc<RwLock<Vec<InhibitLock>>>,
    pub connection: Arc<Connection>,
}

pub async fn daemon(config_path: &Path) -> Result<(), String> {
    let config =
        Config::load(config_path).map_err(|e| format!("Failed to load config: {:?}", e))?;

    let connection = Connection::session().await.unwrap();

    let state = State {
        partymode: Arc::new(AtomicBool::new(config.default_enabled)),
        config: Arc::new(RwLock::new(config)),
        locks: Arc::new(RwLock::new(vec![])),
        connection: Arc::new(connection),
    };

    state
        .connection
        .object_server()
        .at("/dev/peppidesu/partymode", Partymode::new(state.clone()))
        .await
        .map_err(|e| format!("D-Bus error: {}", e))?;

    state
        .connection
        .request_name("dev.peppidesu.partymode")
        .await
        .map_err(|_| "Daemon already running".to_string())?;

    let inhibit_handle = {
        let state = state.clone();
        tokio::task::spawn(async move { inhibit_thread(state).await.unwrap() })
    };

    let mut signals = signal_hook::iterator::Signals::new([SIGTERM, SIGINT, SIGHUP]).unwrap();

    for signal in signals.forever() {
        match signal {
            SIGTERM | SIGINT => {
                inhibit_handle.abort();
                std::process::exit(130);
            }
            SIGHUP => match Config::load(config_path) {
                Ok(config) => {
                    let mut lock = state.config.write().await;
                    *lock = config;
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            },
            _ => {}
        }
    }

    Ok(())
}

pub struct InhibitLock {
    #[allow(dead_code)]
    lock: OwnedFd,
}

async fn inhibit_thread(state: State) -> zbus::Result<()> {
    let connection = state.connection;
    let system_connection = Connection::system().await?;
    let logind = LoginManagerProxy::new(&system_connection).await?;
    loop {
        let mut new_locks = vec![];
        for player in dbus::mpris::find_players(&connection).await? {
            let playback_status = player.playback_status().await?;
            match playback_status {
                dbus::mpris::PlaybackStatus::Playing => {
                    let config = state.config.read().await;
                    let name = player.name();
                    let name = name.strip_prefix("org.mpris.MediaPlayer2.").unwrap_or(name);

                    let rule = if let Some(rule) = config.rules.get(name) {
                        &rule.with_defaults(&config.default_rule)
                    } else {
                        &config.default_rule
                    };

                    if rule.always || state.partymode.load(SeqCst) {
                        let Some(what) = rule.targets_str() else {
                            continue;
                        };
                        let mode = rule
                            .mode
                            .as_ref()
                            .unwrap_or(&InhibitMode::Block)
                            .to_string();

                        let fd = logind
                            .inhibit(
                                &what,
                                "partymode",
                                &format!("{} is playing media", name),
                                &mode,
                            )
                            .await?;
                        new_locks.push(InhibitLock { lock: fd });
                    }
                }
                _ => (),
            }
        }

        let mut locks = state.locks.write().await;
        locks.clear();
        locks.extend(new_locks);
        drop(locks);

        let config = state.config.read().await;
        sleep(Duration::from_millis(config.poll_interval)).await;
    }
}
