use dbus::arg::{RefArg, Variant};
use dbus::ConnPath;
use dbus_tokio::AConnection;
use futures::Future;

#[cfg(feature = "introspection")]
use xml::reader::EventReader;

use super::gen::manager::Manager as IManager;
use super::service::Service;
use super::technology::Technology;
use super::Error;
use std::str::FromStr;
use std::rc::Rc;

/// Futures-aware wrapper struct for connman Manager object.
#[derive(Clone, Debug)]
pub struct Manager {
    connpath: ConnPath<'static, Rc<AConnection>>,
    // TODO: Signal subscription/dispatcher
}

impl Manager {
    pub fn new(connection: Rc<AConnection>) -> Self {
        Manager {
            connpath: Self::connpath(connection),
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
        let connclone = self.connpath.conn.clone();

        IManager::get_technologies(&self.connpath)
            .map_err(Error::from)
            .map(move |v|
                v.into_iter()
                    .map(|(path, args)| Technology::new(connclone.clone(), path, args))
                    .collect()
            )
    }

    pub fn get_services(&self) -> impl Future<Item=Vec<Service>, Error=Error> {
        let connclone = self.connpath.conn.clone();

        IManager::get_services(&self.connpath)
            .map_err(Error::from)
            .map(move |v|
                v.into_iter()
                    .map(|(path, args)| Service::new(connclone.clone(), path, args))
                    .collect()
            )
    }
}

impl Manager {
    #[cfg(feature = "introspection")]
    pub fn introspect(&self) -> impl Future<Item=EventReader<std::io::Cursor<Vec<u8>>>, Error=Error> {
        use crate::api::gen::manager::OrgFreedesktopDBusIntrospectable as Introspectable;

        Introspectable::introspect(&self.connpath)
            .map_err(Error::from)
            .map(|s| {
                let rdr = std::io::Cursor::new(s.into_bytes());
                EventReader::new(rdr)
            })
    }

    pub fn get_state(&self) -> impl Future<Item=State, Error=Error> {
        IManager::get_properties(&self.connpath)
            .map_err(Error::from)
            .and_then(move |a|
                super::get_property_fromstr::<State>(&a, "State")
                    .map_err(Error::from)
            )
    }

    pub fn get_offline_mode(&self) -> impl Future<Item=bool, Error=Error> {
        IManager::get_properties(&self.connpath)
            .map_err(Error::from)
            .and_then(move |a|
                super::get_property::<bool>(&a, "OfflineMode")
                    .map_err(Error::from)
            )
    }

    pub fn set_offline_mode(&self, offline_mode: bool) -> impl Future<Item=(), Error=Error> {
        IManager::set_property(&self.connpath, "OfflineMode", Variant(offline_mode))
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
