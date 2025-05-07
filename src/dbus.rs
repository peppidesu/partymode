use crate::daemon::State;
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;
use zbus::{
    Connection, Result, interface, proxy,
    zvariant::{OwnedFd, OwnedValue},
};

#[proxy(
    interface = "org.freedesktop.DBus",
    default_service = "org.freedesktop.DBus",
    default_path = "/"
)]

pub trait DBus {
    fn list_names(&self) -> Result<Vec<String>>;
}

/**
MPRIS D-Bus interface bindings
https://specifications.freedesktop.org/mpris-spec/latest/
*/
pub mod mpris {
    use super::*;

    /**
    A playback state.

    - Playing (Playing)

        A track is currently playing.

    - Paused (Paused)

        A track is currently paused.

    - Stopped (Stopped)

        There is no track currently playing.

    [(Specification)](https://specifications.freedesktop.org/mpris-spec/latest/Player_Interface.html#Enum:Playback_Status)
    */
    #[derive(OwnedValue)]
    #[zvariant(signature = "s")]
    pub enum PlaybackStatus {
        Playing,
        Paused,
        Stopped,
    }

    #[proxy(
        interface = "org.mpris.MediaPlayer2",
        default_path = "/org/mpris/MediaPlayer2"
    )]
    pub trait MediaPlayer2 {
        #[zbus(property)]
        fn identity(&self) -> Result<String>;
    }

    /**
    Partial implementation of the [(org.mpris.MediaPlayer2.Player)](https://specifications.freedesktop.org/mpris-spec/latest/Player_Interface.html) specification.
    */
    #[proxy(
        interface = "org.mpris.MediaPlayer2.Player",
        default_path = "/org/mpris/MediaPlayer2"
    )]
    pub trait MediaPlayer2Player {
        /**
        The current playback status.

        May be "Playing", "Paused" or "Stopped".

        [(Specification)](https://specifications.freedesktop.org/mpris-spec/latest/Player_Interface.html#Property:PlaybackStatus)
        */
        #[zbus(property)]
        fn playback_status(&self) -> Result<PlaybackStatus>;
    }

    pub struct Player<'a> {
        iface_base: MediaPlayer2Proxy<'a>,
        iface_player: MediaPlayer2PlayerProxy<'a>,
        name: String,
    }

    impl Player<'_> {
        #[allow(dead_code)]
        pub async fn identity(&self) -> Result<String> {
            self.iface_base.identity().await
        }
        pub async fn playback_status(&self) -> Result<PlaybackStatus> {
            self.iface_player.playback_status().await
        }
        pub fn name(&self) -> &str {
            &self.name
        }
    }

    /**
    Get a list of all players connected to D-Bus.
    */
    pub async fn find_players<'a>(connection: &'a Connection) -> Result<Vec<Player<'a>>> {
        let dbus_proxy = DBusProxy::new(connection).await?;
        let names = dbus_proxy.list_names().await?;
        let mut players = vec![];
        for name in names
            .into_iter()
            .filter(|name| name.starts_with("org.mpris.MediaPlayer2."))
        {
            let iface_base = MediaPlayer2Proxy::builder(connection)
                .destination(name.clone())?
                .build()
                .await?;

            let iface_player = MediaPlayer2PlayerProxy::builder(connection)
                .destination(name.clone())?
                .build()
                .await?;

            players.push(Player {
                name,
                iface_base,
                iface_player,
            })
        }

        Ok(players)
    }
}

/**
Logind D-Bus interface bindings.
*/
pub mod logind {
    use super::*;

    /**
    Partial implementation of the [org.freedesktop.login1](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.login1.html) specification.
    */
    #[proxy(
        interface = "org.freedesktop.login1.Manager",
        default_service = "org.freedesktop.login1",
        default_path = "/org/freedesktop/login1"
    )]
    pub trait LoginManager {
        /**
        Request an inhibitor lock.
        */
        fn inhibit(&self, what: &str, who: &str, why: &str, mode: &str) -> Result<OwnedFd>;
    }
}

/**
partymode D-Bus interface
*/
pub mod partymode {
    use super::*;

    pub struct Partymode {
        state: State,
    }
    impl Partymode {
        pub fn new(state: State) -> Partymode {
            Partymode { state }
        }
    }

    #[derive(Type, Serialize, Deserialize)]
    pub struct DaemonStatus {
        pub partymode: bool,
    }

    #[interface(
        name = "dev.peppidesu.partymode",
        proxy(
            gen_blocking = false,
            default_path = "/dev/peppidesu/partymode",
            default_service = "dev.peppidesu.partymode",
        )
    )]
    impl Partymode {
        async fn status(&self) -> DaemonStatus {
            DaemonStatus {
                partymode: self
                    .state
                    .partymode
                    .load(std::sync::atomic::Ordering::SeqCst),
            }
        }

        async fn set(&self, enabled: bool) {
            self.state
                .partymode
                .store(enabled, std::sync::atomic::Ordering::SeqCst);
        }

        async fn toggle(&self) {
            let enabled = self
                .state
                .partymode
                .load(std::sync::atomic::Ordering::SeqCst);

            self.state
                .partymode
                .store(!enabled, std::sync::atomic::Ordering::SeqCst);
        }
    }
}
