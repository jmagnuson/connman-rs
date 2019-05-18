use dbus::arg::{cast, RefArg, Variant};
use dbus::{ConnPath, Message, SignalArgs};
use dbus_tokio::AConnection;
use futures::Future;

use super::gen::manager as genmgr;
use super::gen::manager::Manager as IManager;
use super::service::Service;
use super::technology::Technology;
use super::Error;
use std::borrow::Cow;
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

impl Signal {
    pub fn from_message(msg: &Message) -> Result<Self, Error> {
        if let Some(manager_technology_added) = genmgr::ManagerTechnologyAdded::from_message(&msg) {
            return Ok(Signal::TechnologyAdded(TechnologyAdded {inner: manager_technology_added}));
        }
        if let Some(manager_technology_removed) = genmgr::ManagerTechnologyRemoved::from_message(&msg) {
            return Ok(Signal::TechnologyRemoved(TechnologyRemoved {inner: manager_technology_removed}));
        }
        if let Some(manager_services_changed) = genmgr::ManagerServicesChanged::from_message(&msg) {
            return Ok(Signal::ServicesChanged(ServicesChanged {inner: manager_services_changed}));
        }
        //if let Some(manager_property_changed) = gen::manager::ManagerPropertyChanged::from_message(&msg) {
        //    return Some(Signal::Manager(manager::Signal::PropertyChanged(manager::PropertyChanged {inner: manager_property_changed})));
        //}
        if let Some(manager_peers_changed) = genmgr::ManagerPeersChanged::from_message(&msg) {
            return Ok(Signal::PeersChanged(PeersChanged {inner: manager_peers_changed}));
        }

        Err(super::SignalError::NoMatch(Cow::Borrowed("Manager")).into())
    }
}

#[derive(Debug)]
pub struct TechnologyAdded {
    pub inner: genmgr::ManagerTechnologyAdded,
}

#[derive(Debug)]
pub struct TechnologyRemoved {
    pub inner: genmgr::ManagerTechnologyRemoved,
}

#[derive(Debug)]
pub struct ServicesChanged {
    pub inner: genmgr::ManagerServicesChanged,
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
    pub inner: genmgr::ManagerPeersChanged,
}
