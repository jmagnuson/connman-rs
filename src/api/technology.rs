use dbus::arg;
use dbus::nonblock::{NonblockReply, Proxy, SyncConnection};
use std::sync::Arc;

use std::collections::HashMap;

use super::gen::technology::Technology as ITechnology;
use super::{Error as ApiError, RefArgMap};
use crate::api::{FromProperties, PropertyError};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

#[cfg(feature = "introspection")]
use xml::reader::EventReader;

/// Futures-aware wrapper struct for connman Technology object.
#[derive(Clone)]
pub struct Technology<C> {
    proxy: Proxy<'static, C>,
    pub props: Properties,
}

impl<C> Technology<C> {
    pub fn new(
        connection: C,
        path: dbus::Path<'static>,
        args: RefArgMap,
    ) -> Result<Self, ApiError> {
        Properties::try_from(args)
            .map_err(ApiError::from)
            .map(|props| Technology {
                proxy: Self::proxy(path, connection),
                props,
            })
    }

    pub fn proxy(path: dbus::Path<'static>, conn: C) -> Proxy<'static, C> {
        let proxy = Proxy::new("net.connman", path, Duration::from_millis(5000), conn);
        proxy
    }

    pub fn path(&self) -> &dbus::Path<'static> {
        &self.proxy.path
    }
}

impl<T: NonblockReply, C: Deref<Target = T>> Technology<C> {
    #[cfg(feature = "introspection")]
    pub async fn introspect(&self) -> Result<EventReader<std::io::Cursor<Vec<u8>>>, ApiError> {
        use crate::api::gen::technology::OrgFreedesktopDBusIntrospectable as Introspectable;

        let s = Introspectable::introspect(&self.proxy).await?;
        let rdr = std::io::Cursor::new(s.into_bytes());
        Ok(EventReader::new(rdr))
    }

    pub async fn scan(&self) -> Result<(), ApiError> {
        Ok(ITechnology::scan(&self.proxy).await?)
    }
}

impl<T: NonblockReply, C: Deref<Target = T>> Technology<C> {
    pub async fn set_powered(&self, powered: bool) -> Result<(), ApiError> {
        Ok(ITechnology::set_property(&self.proxy, PropertyKind::Powered.into(), powered).await?)
    }

    pub async fn get_powered(&self) -> Result<bool, ApiError> {
        let a = ITechnology::get_properties(&self.proxy).await?;
        Ok(super::get_property::<bool>(
            &a,
            PropertyKind::Powered.into(),
        )?)
    }

    pub async fn get_connected(&self) -> Result<bool, ApiError> {
        let a = ITechnology::get_properties(&self.proxy).await?;
        Ok(super::get_property::<bool>(
            &a,
            PropertyKind::Connected.into(),
        )?)
    }

    pub async fn get_name(&self) -> Result<String, ApiError> {
        let a = ITechnology::get_properties(&self.proxy).await?;
        Ok(super::get_property_fromstr::<String>(
            &a,
            PropertyKind::Name.into(),
        )?)
    }

    pub async fn get_type(&self) -> Result<Type, ApiError> {
        let a = ITechnology::get_properties(&self.proxy).await?;
        Ok(super::get_property_fromstr::<Type>(
            &a,
            PropertyKind::Type.into(),
        )?)
    }
}

#[derive(Clone, Debug)]
pub struct Properties {
    pub powered: bool,
    pub connected: bool,
    pub name: String,
    pub type_: Type,
    pub tethering: bool,
    pub tethering_identifier: Option<String>,
    pub tethering_passphrase: Option<String>,
}

impl FromProperties for Type {
    fn from_properties(
        properties: &RefArgMap,
        prop_name: &'static str,
    ) -> Result<Self, PropertyError> {
        super::get_property_fromstr::<Self>(properties, prop_name)
    }
}

impl Properties {
    pub fn try_from(props: RefArgMap) -> Result<Self, PropertyError> {
        let powered = bool::from_properties(&props, PropertyKind::Powered.into())?;
        let connected = bool::from_properties(&props, PropertyKind::Connected.into())?;
        let name = String::from_properties(&props, PropertyKind::Name.into())?;
        let type_ = Type::from_properties(&props, PropertyKind::Type.into())?;
        let tethering = bool::from_properties(&props, PropertyKind::Connected.into())?;

        let tethering_identifier: Option<String> =
            FromProperties::from_properties(&props, PropertyKind::TetheringIdentifier.into())?;
        let tethering_passphrase: Option<String> =
            FromProperties::from_properties(&props, PropertyKind::TetheringPassphrase.into())?;

        Ok(Properties {
            powered,
            connected,
            name,
            type_,
            tethering,
            tethering_identifier,
            tethering_passphrase,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PropertyKind {
    Powered,
    Connected,
    Name,
    Type,
    Tethering,
    TetheringIdentifier,
    TetheringPassphrase,
}

impl From<PropertyKind> for &'static str {
    fn from(prop: PropertyKind) -> Self {
        match prop {
            PropertyKind::Powered => "Powered",
            PropertyKind::Connected => "Connected",
            PropertyKind::Name => "Name",
            PropertyKind::Type => "Type",
            PropertyKind::Tethering => "Tethering",
            PropertyKind::TetheringIdentifier => "TetheringIdentifier",
            PropertyKind::TetheringPassphrase => "TetheringPassphrase",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Ethernet,
    Wifi,
    P2p,
    Unknown(String),
}

impl From<Type> for Cow<'static, str> {
    fn from(ty: Type) -> Self {
        match ty {
            Type::Ethernet => Cow::Borrowed("ethernet"),
            Type::Wifi => Cow::Borrowed("wifi"),
            Type::P2p => Cow::Borrowed("p2p"),
            Type::Unknown(inner) => Cow::Owned(inner),
        }
    }
}

impl FromStr for Type {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = match s {
            "ethernet" => Type::Ethernet,
            "wifi" => Type::Wifi,
            "p2p" => Type::P2p,
            _ => Type::Unknown(s.to_string()),
        };
        Ok(t)
    }
}
