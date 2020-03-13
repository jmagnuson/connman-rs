use dbus::arg::{RefArg, Variant};
use dbus::nonblock::{Proxy, SyncConnection};
use futures::{future, Future, TryFutureExt};

#[cfg(feature = "introspection")]
use xml::reader::EventReader;

use super::gen::manager::Manager as IManager;
use super::service::{Properties as ServiceProperties, Service};
use super::technology::Technology;
use super::Error;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Duration;

/// Futures-aware wrapper struct for connman Manager object.
#[derive(Clone)]
pub struct Manager {
    proxy: Proxy<'static, Rc<SyncConnection>>,
    // TODO: Signal subscription/dispatcher
}

impl Manager {
    pub fn new(connection: Rc<SyncConnection>) -> Self {
        Manager {
            proxy: Self::proxy(connection),
        }
    }

    pub fn proxy(conn: Rc<SyncConnection>) -> Proxy<'static, Rc<SyncConnection>> {
        let proxy = Proxy::new("net.connman", "/", Duration::from_millis(5000), conn);
        proxy
    }
}

impl Manager {
    pub fn get_technologies(&self) -> impl Future<Output = Result<Vec<Technology>, Error>> {
        let connclone = self.proxy.connection.clone();

        IManager::get_technologies(&self.proxy)
            .map_err(Error::from)
            .map_ok(move |v| {
                v.into_iter()
                    .filter_map(|(path, args)| Technology::new(connclone.clone(), path, args).ok())
                    .collect()
            })
    }

    pub fn get_services(&self) -> impl Future<Output = Result<Vec<Service>, Error>> {
        let connclone = self.proxy.connection.clone();

        IManager::get_services(&self.proxy)
            .map_err(Error::from)
            .map_ok(move |v| {
                v.into_iter()
                    .filter_map(|(path, args)| Service::new(connclone.clone(), path, args).ok())
                    .collect()
            })
    }
}

impl Manager {
    #[cfg(feature = "introspection")]
    pub fn introspect(
        &self,
    ) -> impl Future<Item = EventReader<std::io::Cursor<Vec<u8>>>, Error = Error> {
        use crate::api::gen::manager::OrgFreedesktopDBusIntrospectable as Introspectable;

        Introspectable::introspect(&self.proxy)
            .map_err(Error::from)
            .map(|s| {
                let rdr = std::io::Cursor::new(s.into_bytes());
                EventReader::new(rdr)
            })
    }

    pub fn get_state(&self) -> impl Future<Output = Result<State, Error>> {
        IManager::get_properties(&self.proxy)
            .map_err(Error::from)
            .and_then(move |a| {
                future::ready(
                    super::get_property_fromstr::<State>(&a, "State").map_err(Error::from),
                )
            })
    }

    pub fn get_offline_mode(&self) -> impl Future<Output = Result<bool, Error>> {
        IManager::get_properties(&self.proxy)
            .map_err(Error::from)
            .and_then(move |a| {
                future::ready(super::get_property::<bool>(&a, "OfflineMode").map_err(Error::from))
            })
    }

    pub fn set_offline_mode(&self, offline_mode: bool) -> impl Future<Output = Result<(), Error>> {
        IManager::set_property(&self.proxy, "OfflineMode", Variant(Box::new(offline_mode)))
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
