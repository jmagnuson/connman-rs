use dbus::arg::{RefArg, Variant};
use dbus::nonblock::{NonblockReply, Proxy, SyncConnection};

#[cfg(feature = "introspection")]
use xml::reader::EventReader;

use super::gen::manager::Manager as IManager;
use super::service::{Properties as ServiceProperties, Service};
use super::technology::Technology;
use super::Error;
use std::future::Future;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

/// Futures-aware wrapper struct for connman Manager object.
#[derive(Clone)]
pub struct Manager<C> {
    proxy: Proxy<'static, C>,
    timeout: Duration,
    // TODO: Signal subscription/dispatcher
}

impl<C> Manager<C> {
    pub fn new(connection: C, timeout: Duration) -> Self {
        Manager {
            proxy: Self::proxy(timeout, connection),
            timeout,
        }
    }

    pub fn proxy(timeout: Duration, conn: C) -> Proxy<'static, C> {
        let proxy = Proxy::new("net.connman", "/", timeout, conn);
        proxy
    }
}

impl<T: NonblockReply, C: Deref<Target = T> + Clone> Manager<C> {
    pub async fn get_technologies(&self) -> Result<Vec<Technology<C>>, Error> {
        let connclone = self.proxy.connection.clone();

        let v = IManager::get_technologies(&self.proxy).await?;
        Ok(v.into_iter()
            .filter_map(|(path, args)| {
                Technology::new(connclone.clone(), path, args, self.timeout).ok()
            })
            .collect())
    }

    pub async fn get_services(&self) -> Result<Vec<Service<C>>, Error> {
        let connclone = self.proxy.connection.clone();

        let services = IManager::get_services(&self.proxy).await?;
        Ok(services
            .into_iter()
            .filter_map(|(path, args)| {
                Service::new(connclone.clone(), path, args, self.timeout).ok()
            })
            .collect())
    }

    pub async fn register_agent(&self, path: dbus::Path<'static>) -> Result<(), Error> {
        IManager::register_agent(&self.proxy, path).await?;
        Ok(())
    }
}

impl<T: NonblockReply, C: Deref<Target = T>> Manager<C> {
    #[cfg(feature = "introspection")]
    pub async fn introspect(&self) -> Result<EventReader<std::io::Cursor<Vec<u8>>>, Error> {
        use crate::api::gen::manager::OrgFreedesktopDBusIntrospectable as Introspectable;

        let s = Introspectable::introspect(&self.proxy).await?;
        let rdr = std::io::Cursor::new(s.into_bytes());
        Ok(EventReader::new(rdr))
    }

    pub async fn get_state(&self) -> Result<State, Error> {
        let a = IManager::get_properties(&self.proxy).await?;
        Ok(super::get_property_fromstr::<State>(&a, "State")?)
    }

    pub async fn get_offline_mode(&self) -> Result<bool, Error> {
        let a = IManager::get_properties(&self.proxy).await?;
        Ok(super::get_property::<bool>(&a, "OfflineMode")?)
    }

    pub async fn set_offline_mode(&self, offline_mode: bool) -> Result<(), Error> {
        Ok(IManager::set_property(&self.proxy, "OfflineMode", offline_mode).await?)
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
