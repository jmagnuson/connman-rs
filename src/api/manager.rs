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

#[derive(Clone, Debug)]
pub enum SignalKind {
    TechnologyAdded,
    TechnologyRemoved,
    ServicesChanged,
    PropertyChanged,
    PeersChanged,
}

impl FromStr for SignalKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TechnologyAdded" => Ok(SignalKind::TechnologyAdded),
            "TechnologyRemoved" => Ok(SignalKind::TechnologyAdded),
            "ServicesChanged" => Ok(SignalKind::TechnologyAdded),
            "PropertyChanged" => Ok(SignalKind::TechnologyAdded),
            "PeersChanged" => Ok(SignalKind::TechnologyAdded),
            _ => Err(()),
        }
    }
}

impl From<SignalKind> for &'static str {
    fn from(iface: SignalKind) -> Self {
        match iface {
            SignalKind::TechnologyAdded => "TechnologyAdded",
            SignalKind::TechnologyRemoved => "TechnologyRemoved",
            SignalKind::ServicesChanged => "ServicesChanged",
            SignalKind::PropertyChanged => "PropertyChanged",
            SignalKind::PeersChanged => "PeersChanged",
        }
    }
}

impl Signal {
    pub fn from_message(msg: &Message) -> Result<Self, Error> {
        msg.member()
            .ok_or(Error::SignalError(super::SignalError::NoMatch(Cow::Borrowed("Manager"))))
            .and_then(|ref dbus_name| {
                SignalKind::from_str(&**dbus_name)
                    .map_err(|_| Error::SignalError(super::SignalError::NoMatch(Cow::Borrowed("Manager"))))
            })
            .and_then(|name| {
                match name {
                    SignalKind::TechnologyAdded => {
                        genmgr::ManagerTechnologyAdded::from_message(&msg)
                            .map(|i| Signal::TechnologyAdded(TechnologyAdded{inner: i}))
                            .ok_or(Error::SignalError(super::SignalError::NoMatch(Cow::Borrowed("Manager"))))
                    },
                    SignalKind::TechnologyRemoved => {
                        genmgr::ManagerTechnologyRemoved::from_message(&msg)
                            .map(|i| Signal::TechnologyRemoved(TechnologyRemoved{inner: i}))
                            .ok_or(Error::SignalError(super::SignalError::NoMatch(Cow::Borrowed("Manager"))))
                    },
                    SignalKind::ServicesChanged => {
                        genmgr::ManagerServicesChanged::from_message(&msg)
                            .map(|i| Signal::ServicesChanged(ServicesChanged{inner: i}))
                            .ok_or(Error::SignalError(super::SignalError::NoMatch(Cow::Borrowed("Manager"))))
                    },
                    SignalKind::PropertyChanged => {
                        genmgr::ManagerPropertyChanged::from_message(&msg)
                            .ok_or(Error::SignalError(super::SignalError::NoMatch(Cow::Borrowed("Manager"))))
                            .and_then(|m| {
                                Property::try_from(m)
                                    .map_err(|_| Error::SignalError(super::SignalError::NoMatch(Cow::Borrowed("Manager"))))
                            })
                            .map(|i| Signal::PropertyChanged(PropertyChanged{inner: i}))
                    },
                    SignalKind::PeersChanged => {
                        genmgr::ManagerPeersChanged::from_message(&msg)
                            .map(|i| Signal::PeersChanged(PeersChanged{inner: i}))
                            .ok_or(Error::SignalError(super::SignalError::NoMatch(Cow::Borrowed("Manager"))))
                    },
                }
            })
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
