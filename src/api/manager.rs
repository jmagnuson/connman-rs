use dbus::arg::{RefArg, Variant};
use dbus::nonblock::Proxy as ConnPath;
use futures::Future;
use futures::TryFutureExt;
use std::sync::Arc;
use std::time::Duration;
use std::fmt;

type AConnection = Arc<dbus::nonblock::SyncConnection>;

#[cfg(feature = "introspection")]
use xml::reader::EventReader;

use super::gen::manager::Manager as IManager;
use super::service::{Service, Properties as ServiceProperties};
use super::technology::Technology;
use super::Error;
use std::str::FromStr;
use std::rc::Rc;

/// Futures-aware wrapper struct for connman Manager object.
#[derive(Clone)]
pub struct Manager {
    connpath: ConnPath<'static, AConnection>,
    // TODO: Signal subscription/dispatcher
}

impl fmt::Debug for Manager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Manager")
         .field("connpath", &"<elided>")
         .finish()
    }
}

impl Manager {
    pub fn new(connection: AConnection) -> Self {
        Manager {
            connpath: Self::connpath(connection),
        }
    }

    pub fn connpath(conn: AConnection) -> ConnPath<'static, AConnection> {
        let connpath = ConnPath {
            connection: conn,
            destination: "net.connman".into(),
            path: "/".into(),
            timeout: Duration::from_secs(5),
        };
        connpath
    }
}

impl Manager {
    pub async fn get_technologies(&self) -> Result<Vec<Technology>, Error> {
        let connclone = self.connpath.connection.clone();

        IManager::get_technologies(&self.connpath).await
            .map_err(Error::from)
            .map(move |v|
                v.into_iter()
                    .filter_map(|(path, args)| {
                        Technology::new(connclone.clone(), path, args).ok()
                    })
                    .collect()
            )
    }

    pub async fn get_services(&self) -> Result<Vec<Service>, Error> {
        let connclone = self.connpath.connection.clone();

        IManager::get_services(&self.connpath).await
            .map_err(Error::from)
            .map(move |v|
                v.into_iter()
                    .filter_map(|(path, args)| {
                        Service::new(connclone.clone(), path, args).ok()
                    })
                    .collect()
            )
    }
}

impl Manager {
    #[cfg(feature = "introspection")]
    pub async fn introspect(&self) -> Result<EventReader<std::io::Cursor<Vec<u8>>>, Error> {
        use crate::api::gen::manager::OrgFreedesktopDBusIntrospectable as Introspectable;

        Introspectable::introspect(&self.connpath).await
            .map_err(Error::from)
            .map(|s| {
                let rdr = std::io::Cursor::new(s.into_bytes());
                EventReader::new(rdr)
            })
    }

    pub async fn get_state(&self) -> Result<State, Error> {
        IManager::get_properties(&self.connpath).await
            .map_err(Error::from)
            .and_then(move |a|
                super::get_property_fromstr::<State>(&a, "State")
                    .map_err(Error::from)
            )
    }

    pub async fn get_offline_mode(&self) -> Result<bool, Error> {
        IManager::get_properties(&self.connpath).await
            .map_err(Error::from)
            .and_then(move |a|
                super::get_property::<bool>(&a, "OfflineMode")
                    .map_err(Error::from)
            )
    }

    pub async fn set_offline_mode(&self, offline_mode: bool) -> Result<(), Error> {
        IManager::set_property(&self.connpath, "OfflineMode", Variant(offline_mode)).await
            .map_err(Error::from)
    }
}

/// Manager connection state, `from_str` maps the values given over d-bus by
/// connman -- values are "offline", "idle", "ready" and "online".
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum State {
    Offline,
    Idle,
    Ready,
    Online,
}

impl FromStr for State {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "offline" => Ok(State::Offline),
            "idle" => Ok(State::Idle),
            "ready" => Ok(State::Ready),
            "online" => Ok(State::Online),
            _ => Err(()),
        }
    }
}
