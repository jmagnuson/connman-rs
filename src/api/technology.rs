use dbus::{arg, ConnPath};
use dbus_tokio::AConnection;
use futures::Future;
use std::rc::Rc;

use std::collections::HashMap;

use super::gen::technology::Technology as ITechnology;
use super::{Error as ApiError, RefArgMap};
use std::str::FromStr;
use std::borrow::Cow;
use std::convert::TryFrom;
use crate::api::{PropertyError, FromProperties};

#[cfg(feature = "introspection")]
use xml::reader::EventReader;

/// Futures-aware wrapper struct for connman Technology object.
#[derive(Clone, Debug)]
pub struct Technology {
    connpath: ConnPath<'static, Rc<AConnection>>,
    pub props: Properties,
}

impl Technology {
    pub fn new(
        connection: Rc<AConnection>,
        path: dbus::Path<'static>,
        args: RefArgMap,
    ) -> Result<Self, ApiError> {
        Properties::try_from(args)
            .map_err(ApiError::from)
            .map(|props| {
                Technology {
                    connpath: Self::connpath(path, connection),
                    props,
                }
            })
    }

    pub fn connpath(path: dbus::Path<'static>, conn: Rc<AConnection>) -> ConnPath<'static, Rc<AConnection>> {
        let connpath = ConnPath {
            conn: conn,
            dest: "net.connman".into(),
            path,
            timeout: 5000,
        };
        connpath
    }

    pub fn path(&self) -> &dbus::Path<'static> {
        &self.connpath.path
    }
}

impl Technology {
    #[cfg(feature = "introspection")]
    pub fn introspect(&self) -> impl Future<Item=EventReader<std::io::Cursor<Vec<u8>>>, Error=ApiError> {
        use crate::api::gen::technology::OrgFreedesktopDBusIntrospectable as Introspectable;

        Introspectable::introspect(&self.connpath)
            .map_err(ApiError::from)
            .map(|s| {
                let rdr = std::io::Cursor::new(s.into_bytes());
                EventReader::new(rdr)
            })
    }

    pub fn scan(&self) -> impl Future<Item=(), Error=ApiError> {
        ITechnology::scan(&self.connpath)
            .map_err(ApiError::from)
    }
}

impl Technology {
    pub fn set_powered(&self, powered: bool) -> impl Future<Item=(), Error=ApiError> {
        ITechnology::set_property(&self.connpath, PropertyKind::Powered.into(), arg::Variant(powered))
            .map_err(|e| e.into())
    }

    pub fn get_powered(&self) -> impl Future<Item=bool, Error=ApiError> {
        ITechnology::get_properties(&self.connpath)
            .map_err(ApiError::from)
            .and_then(move |a|
                super::get_property::<bool>(&a, PropertyKind::Powered.into())
                    .map_err(ApiError::from)
            )
    }

    pub fn get_connected(&self) -> impl Future<Item=bool, Error=ApiError> {
        ITechnology::get_properties(&self.connpath)
            .map_err(ApiError::from)
            .and_then(move |a|
                super::get_property::<bool>(&a, PropertyKind::Connected.into())
                    .map_err(ApiError::from)
            )
    }

    pub fn get_name(&self) -> impl Future<Item=String, Error=ApiError> {
        ITechnology::get_properties(&self.connpath)
            .map_err(ApiError::from)
            .and_then(move |a|
                super::get_property_fromstr::<String>(&a, PropertyKind::Name.into())
                    .map_err(ApiError::from)
            )
    }

    pub fn get_type(&self) -> impl Future<Item=Type, Error=ApiError> {
        ITechnology::get_properties(&self.connpath)
            .map_err(ApiError::from)
            .and_then(move |a|
                super::get_property_fromstr::<Type>(&a, PropertyKind::Type.into())
                    .map_err(ApiError::from)
            )
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
    fn from_properties(properties: &RefArgMap, prop_name: &'static str) -> Result<Self, PropertyError> {
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

        let tethering_identifier: Option<String> = FromProperties::from_properties(
            &props,
            PropertyKind::TetheringIdentifier.into()
        )?;
        let tethering_passphrase: Option<String> = FromProperties::from_properties(
            &props,
            PropertyKind::TetheringPassphrase.into()
        )?;

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
            _ => Type::Unknown(s.to_string())
        };
        Ok(t)
    }
}