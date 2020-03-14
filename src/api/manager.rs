use dbus::arg::{RefArg, Variant};
use dbus::nonblock::{Proxy, SyncConnection};

#[cfg(feature = "introspection")]
use xml::reader::EventReader;

use super::gen::manager::Manager as IManager;
use super::service::{Properties as ServiceProperties, Service};
use super::technology::Technology;
use super::Error;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

/// Futures-aware wrapper struct for connman Manager object.
#[derive(Clone)]
pub struct Manager {
    proxy: Proxy<'static, Arc<SyncConnection>>,
    // TODO: Signal subscription/dispatcher
}

impl Manager {
    pub fn new(connection: Arc<SyncConnection>) -> Self {
        Manager {
            proxy: Self::proxy(connection),
        }
    }

    pub fn proxy(conn: Arc<SyncConnection>) -> Proxy<'static, Arc<SyncConnection>> {
        let proxy = Proxy::new("net.connman", "/", Duration::from_millis(5000), conn);
        proxy
    }
}

impl Manager {
    pub async fn get_technologies(&self) -> Result<Vec<Technology>, Error> {
        let connclone = self.proxy.connection.clone();

        let v = IManager::get_technologies(&self.proxy).await?;
        Ok(v.into_iter()
            .filter_map(|(path, args)| Technology::new(connclone.clone(), path, args).ok())
            .collect())
    }

    pub async fn get_services(&self) -> Result<Vec<Service>, Error> {
        let connclone = self.proxy.connection.clone();

        let v = IManager::get_services(&self.proxy).await?;
        Ok(v.into_iter()
            .filter_map(|(path, args)| Service::new(connclone.clone(), path, args).ok())
            .collect())
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

    pub async fn get_state(&self) -> Result<State, Error> {
        let a = IManager::get_properties(&self.proxy).await?;
        Ok(super::get_property_fromstr::<State>(&a, "State")?)
    }

    pub async fn get_offline_mode(&self) -> Result<bool, Error> {
        let a = IManager::get_properties(&self.proxy).await?;
        Ok(super::get_property::<bool>(&a, "OfflineMode")?)
    }

    pub async fn set_offline_mode(&self, offline_mode: bool) -> Result<(), Error> {
        Ok(
            IManager::set_property(&self.proxy, "OfflineMode", Variant(Box::new(offline_mode)))
                .await?,
        )
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
