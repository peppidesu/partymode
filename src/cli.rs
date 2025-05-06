use clap::{Command, arg};
use std::path::PathBuf;

use crate::{dbus, default_config_path};

/// Command line arguments for partymode.
pub struct Args {
    cmd: Cmd,
    config: Option<PathBuf>,
    #[allow(dead_code)]
    verbose: bool,
}

#[derive(Debug)]
pub enum Cmd {
    /// Run the partymode daemon.
    Daemon,
    /// Enable party mode.
    On,
    /// Disable party mode.
    Off,
    /// Toggle party mode.
    Toggle,
    /// Show the status of partymode.
    Status,
}

impl Cmd {
    pub fn name(&self) -> &'static str {
        match self {
            Cmd::Daemon => "daemon",
            Cmd::On => "on",
            Cmd::Off => "off",
            Cmd::Toggle => "toggle",
            Cmd::Status => "status",
        }
    }

    pub fn about(&self) -> &'static str {
        match self {
            Cmd::Daemon => "Run the partymode daemon",
            Cmd::On => "Enable party mode",
            Cmd::Off => "Disable party mode",
            Cmd::Toggle => "Toggle party mode",
            Cmd::Status => "Show the current status",
        }
    }
}

impl<S> From<S> for Cmd
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        match value.as_ref() {
            "daemon" => Cmd::Daemon,
            "on" => Cmd::On,
            "off" => Cmd::Off,
            "toggle" => Cmd::Toggle,
            "status" => Cmd::Status,
            _ => panic!("Unknown command"),
        }
    }
}

pub fn parse() -> Args {
    macro_rules! mk_subcommand {
        ($subc:ident) => {
            Command::new(Cmd::$subc.name()).about(Cmd::$subc.about())
        };
    }

    let matches = clap::command!()
        .subcommand_required(true)
        .subcommand(mk_subcommand!(Daemon))
        .subcommand(mk_subcommand!(On))
        .subcommand(mk_subcommand!(Off))
        .subcommand(mk_subcommand!(Toggle))
        .subcommand(mk_subcommand!(Status))
        .arg(arg!(-c --config <PATH> "Provide a custom location for the config file"))
        .arg(arg!(-v --verbose ... "Enable verbose logging").action(clap::ArgAction::SetTrue))
        .get_matches();

    let cmd = Cmd::from(matches.subcommand().unwrap().0);

    let config = matches
        .get_one::<String>("config")
        .map(|s| PathBuf::from(s));

    let verbose = matches.get_flag("verbose");

    let args = Args {
        cmd,
        config,
        verbose,
    };

    return args;
}

pub async fn run(args: Args) -> Result<(), String> {
    match args.cmd {
        Cmd::Daemon => {
            // Start the daemon
            println!("Starting daemon...");
            crate::daemon::daemon(&args.config.unwrap_or_else(|| default_config_path()))
                .await
                .map_err(|e| format!("Failed to start daemon: {}", e))
        }
        Cmd::On => {
            let connection = zbus::Connection::session()
                .await
                .map_err(|e| format!("Failed to connect to D-Bus: {}", e))?;

            let proxy = dbus::partymode::PartymodeProxy::new(&connection)
                .await
                .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

            proxy
                .set(true)
                .await
                .map_err(|e| format!("Could not enable partymode: {}", e))
        }
        Cmd::Off => {
            let connection = zbus::Connection::session()
                .await
                .map_err(|e| format!("Failed to connect to D-Bus: {}", e))?;

            let proxy = dbus::partymode::PartymodeProxy::new(&connection)
                .await
                .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

            proxy
                .set(false)
                .await
                .map_err(|e| format!("Could not enable partymode: {}", e))
        }
        Cmd::Toggle => {
            let connection = zbus::Connection::session()
                .await
                .map_err(|e| format!("Failed to connect to D-Bus: {}", e))?;

            let proxy = dbus::partymode::PartymodeProxy::new(&connection)
                .await
                .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

            proxy
                .toggle()
                .await
                .map_err(|e| format!("Could not enable partymode: {}", e))
        }
        Cmd::Status => {
            let connection = zbus::Connection::session()
                .await
                .map_err(|e| format!("Failed to connect to D-Bus: {}", e))?;

            let proxy = dbus::partymode::PartymodeProxy::new(&connection)
                .await
                .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

            let status = proxy
                .status()
                .await
                .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

            println!(
                "{}",
                if status.partymode {
                    "enabled"
                } else {
                    "disabled"
                }
            );

            Ok(())
        }
    }
}
