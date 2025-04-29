use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::SeqCst},
    },
};

use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};

use tokio::sync::RwLock;

use crate::{
    PID_LOCK_PATH, SOCKET_PATH,
    config::Config,
    socket::{self, Request, Response},
};

#[derive(Clone)]
struct State {
    config: Arc<RwLock<Config>>,
    partymode: Arc<AtomicBool>,
}

pub async fn daemon(config_path: &Path) -> Result<(), String> {
    if already_running() {
        return Err("Already running".to_string());
    }

    let socket =
        socket::Socket::create().map_err(|e| format!("Failed to create socket: {:?}", e))?;

    let config =
        Config::load(config_path).map_err(|e| format!("Failed to load config: {:?}", e))?;

    let state = State {
        partymode: Arc::new(AtomicBool::new(config.default_enabled)),
        config: Arc::new(RwLock::new(config)),
    };

    println!("Started listening for socket connections");

    let listen_handle = {
        let state = state.clone();
        tokio::task::spawn(async move {
            socket
                .listen(move |request| {
                    handle_stream(request, &state);
                })
                .await;
        })
    };

    let mut signals = signal_hook::iterator::Signals::new([SIGTERM, SIGINT, SIGHUP]).unwrap();

    for signal in signals.forever() {
        match signal {
            SIGTERM | SIGINT => {
                listen_handle.abort();
                remove_runtime_files();
                std::process::exit(130);
            }
            SIGHUP => {
                let config = Config::load(config_path)
                    .map_err(|e| format!("Failed to load config: {:?}", e))?;

                let mut lock = state.config.write().await;
                *lock = config;
            }
            _ => {}
        }
    }

    Ok(())
}

fn already_running() -> bool {
    let path = &*PID_LOCK_PATH;
    let running = if path.exists() {
        let pid = std::fs::read_to_string(&path).unwrap();
        Command::new("ps")
            .args(["-p", &pid])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        false
    };

    if !running {
        std::fs::write(&path, format!("{}", std::process::id()).into_bytes()).unwrap();
    }
    running
}

fn remove_runtime_files() {
    std::fs::remove_file(&*SOCKET_PATH).unwrap_or(());
    std::fs::remove_file(&*PID_LOCK_PATH).unwrap_or(());
}

fn handle_stream(request: Request, state: &State) -> Response {
    match request {
        Request::Set { enabled } => {
            state.partymode.store(enabled, SeqCst);
            println!("Set partymode to {}", enabled);
            Response::Ok
        }
        Request::Toggle => {
            let current = state.partymode.load(SeqCst);
            state.partymode.store(!current, SeqCst);
            println!("Toggled partymode to {}", !current);
            Response::Ok
        }
        Request::Status => {
            let enabled = state.partymode.load(SeqCst);
            Response::Status { enabled }
        }
    }
}
