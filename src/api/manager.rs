use dbus::arg::{cast, RefArg, Variant};
use dbus::ConnPath;
use dbus_tokio::AConnection;
use futures::Future;

use super::gen::manager as genmgr;
use super::gen::manager::Manager as IManager;
use super::service::Service;
use super::technology::Technology;
use super::Error;
use std::str::FromStr;
use std::rc::Rc;
use std::convert::TryFrom;

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

pub enum PropertyKind {
    State,
    OfflineMode,
    SessionMode,
}

impl FromStr for PropertyKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "State" => Ok(PropertyKind::State),
            "OfflineMode" => Ok(PropertyKind::OfflineMode),
            "SessionMode" => Ok(PropertyKind::SessionMode),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Property {
    State(State),
    OfflineMode(bool),
    SessionMode(bool),
}

impl TryFrom<genmgr::ManagerPropertyChanged> for Property {
    type Error = ();

    fn try_from(val: genmgr::ManagerPropertyChanged) -> Result<Self, Self::Error> {
        PropertyKind::from_str(val.name.as_str()).and_then(|prop| {
            match prop {
                PropertyKind::State => {
                    val.value.as_str().ok_or(())
                        .and_then(|valstr| State::from_str(valstr))
                        .map(|v| Property::State(v))
                },
                PropertyKind::OfflineMode => cast::<bool>(&val.value)
                    .ok_or(()).map(|v| Property::OfflineMode(*v)),
                PropertyKind::SessionMode => cast::<bool>(&val.value)
                    .ok_or(()).map(|v| Property::SessionMode(*v)),
            }
        })
    }
}


#[derive(Debug)]
pub enum Signal {
    TechnologyAdded(TechnologyAdded),
    TechnologyRemoved(TechnologyRemoved),
    ServicesChanged(ServicesChanged),
    PropertyChanged(PropertyChanged),
    PeersChanged(PeersChanged),
}

#[derive(Debug)]
pub struct TechnologyAdded {
    pub inner: super::gen::manager::ManagerTechnologyAdded,
}

#[derive(Debug)]
pub struct TechnologyRemoved {
    pub inner: super::gen::manager::ManagerTechnologyRemoved,
}

#[derive(Debug)]
pub struct ServicesChanged {
    pub inner: super::gen::manager::ManagerServicesChanged,
}

#[derive(Debug)]
pub struct PropertyChanged {
    //    inner: manager::ManagerPropertyChanged,
    // TODO: map ^ to v
    pub inner: Property,
}

impl TryFrom<genmgr::ManagerPropertyChanged> for PropertyChanged {
    type Error = ();

    fn try_from(val: genmgr::ManagerPropertyChanged) -> Result<Self, Self::Error> {
        Property::try_from(val).map(|v| PropertyChanged { inner: v })
    }
}

#[derive(Debug)]
pub struct PeersChanged {
    pub inner: super::gen::manager::ManagerPeersChanged,
}
