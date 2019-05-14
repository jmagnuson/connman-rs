use dbus::arg::{RefArg, Variant};
use dbus::ConnPath;
use dbus_tokio::AConnection;
use futures::Future;

use super::gen::manager::Manager as IManager;
use super::service::Service;
use super::technology::Technology;
use super::Error;
use std::str::FromStr;
use std::rc::Rc;

/// Futures-aware wrapper struct for connman Manager object.
#[derive(Clone, Debug)]
pub struct Manager {
    connection: Rc<AConnection>,
    // TODO: Signal subscription/dispatcher
}

impl Manager {
    pub fn new(connection: Rc<AConnection>) -> Self {
        Manager {
            connection
        }
    }

    pub fn connpath(conn: Rc<AConnection>) -> ConnPath<'static, Rc<AConnection>> {
        let connpath = ConnPath {
            conn: conn,
            dest: "net.connman".into(),
            path: "/".into(),
            timeout: 5000,
        };
        connpath
    }
}

impl Manager {
    pub fn get_technologies(&self) -> impl Future<Item=Vec<Technology>, Error=Error> {
        let connclone = self.connection.clone();

        let connpath = Self::connpath(connclone.clone());
        IManager::get_technologies(&connpath)
            .map_err(|e| e.into())
            .map(move |v|
                v.into_iter()
                    .map(|(path, args)| Technology::new(connclone.clone(), path, args))
                    .collect()
            )
    }

    pub fn get_services(&self) -> impl Future<Item=Vec<Service>, Error=Error> {
        let connclone = self.connection.clone();

        let connpath = Self::connpath(connclone.clone());
        IManager::get_services(&connpath)
            .map_err(|e| e.into())
            .map(move |v|
                v.into_iter()
                    .map(|(path, args)| Service::new(connclone.clone(), path, args))
                    .collect()
            )
    }
}

impl Manager {
    pub fn get_state(&self) -> impl Future<Item=State, Error=Error> {
        let connpath = Self::connpath(self.connection.clone());
        IManager::get_properties(&connpath)
            .map_err(|e| e.into())
            .and_then(move |a|
                super::get_property_fromstr::<State>(&a, "State")
            )
    }

    pub fn get_offline_mode(&self) -> impl Future<Item=bool, Error=Error> {
        let connpath = Self::connpath(self.connection.clone());
        IManager::get_properties(&connpath)
            .map_err(|e| e.into())
            .and_then(move |a|
                super::get_property::<bool>(&a, "OfflineMode")
            )
    }

    pub fn set_offline_mode(&self, offline_mode: bool) -> impl Future<Item=(), Error=Error> {
        let connpath = Self::connpath(self.connection.clone());
        IManager::set_property(&connpath, "OfflineMode", Variant(offline_mode))
            .map_err(|e| e.into())
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
