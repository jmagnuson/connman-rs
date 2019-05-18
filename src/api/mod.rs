//! D-Bus API implementations

#[rustfmt::skip]
mod gen;

pub mod manager;
pub mod service;
pub mod technology;

use dbus;
use dbus::{Message, SignalArgs};
use dbus::arg::{cast, RefArg, Variant};

use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;

type RefArgMap = HashMap<String, Variant<Box<RefArg + 'static>>>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    DbusError(#[cause] dbus::Error),
    #[fail(display = "'{}'", _0)]
    PropertyError(#[cause] PropertyError),
    #[fail(display = "'{}'", _0)]
    SignalError(#[cause] SignalError),
}

#[derive(Debug, Fail)]
pub enum PropertyError {
    #[fail(display = "Property not present: '{}'", _0)]
    NotPresent(Cow<'static, str>),
    #[fail(display = "Failed to cast property: '{}'", _0)]
    Cast(Cow<'static, str>)
}

#[derive(Debug, Fail)]
pub enum SignalError {
    #[fail(display = "No match for: '{}'", _0)]
    NoMatch(Cow<'static, str>),
    #[fail(display = "Failed to cast property: '{}'", _0)]
    Cast(Cow<'static, str>)
}

impl From<PropertyError> for Error {
    fn from(e: PropertyError) -> Self {
        Error::PropertyError(e)
    }
}

impl From<dbus::Error> for Error {
    fn from(e: dbus::Error) -> Self {
        Error::DbusError(e)
    }
}

impl From<SignalError> for Error {
    fn from(e: SignalError) -> Self {
        Error::SignalError(e)
    }
}

/// Convenience function for getting property values.
fn get_property<T: Clone + 'static>(
    properties: &RefArgMap,
    prop_name: &'static str,
) -> Result<T, Error> {
    properties.get(prop_name)
        .ok_or(PropertyError::NotPresent(Cow::Borrowed(prop_name)).into())
        .and_then(|variant|
            cast::<T>(&variant.0).cloned()
                .ok_or(PropertyError::Cast(Cow::Borrowed(prop_name)).into())
        )
}

/// Convenience function for getting property values that impl `FromStr`.
fn get_property_fromstr<T: FromStr + 'static>(
    properties: &RefArgMap,
    prop_name: &'static str,
) -> Result<T, Error> {
    properties.get(prop_name)
        .ok_or(PropertyError::NotPresent(Cow::Borrowed(prop_name)).into())
        .and_then(|variant| variant.as_str()
            .and_then(|s| T::from_str(s).ok())
            .ok_or(PropertyError::Cast(Cow::Borrowed(prop_name)).into())
        )
}

pub enum Interface {
    Manager,
    //Technology,
    //Service
}

impl FromStr for Interface {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "net.connman.Manager" => Ok(Interface::Manager),
            //"net.connman.Technology" => Ok(Interface::Technology),
            //"net.connman.Service" => Ok(Interface::Service),
            _ => Err(()),
        }
    }
}

impl From<Interface> for &'static str {
    fn from(iface: Interface) -> Self {
        match iface {
            Interface::Manager => "net.connman.Manager",
            //Interface::Technology => "net.connman.Technology",
            //Interface::Service => "net.connman.Service",
        }
    }
}

impl Interface {
    /// Creates an iterator for all defined `Interface` variants. Useful for
    /// setting up signals filter.
    pub fn values() -> std::slice::Iter<'static, Interface> {
        static INTERFACES: [Interface;  /*3*/ 1] = [
            Interface::Manager,
            //Interface::Technology,
            //Interface::Service,
        ];
        INTERFACES.into_iter()
    }
}

#[derive(Debug)]
pub enum Signal {
    Manager(manager::Signal),
    //Technology(TechnologySignal),
    //Service(ServiceSignal),
}

impl Signal {
    pub(crate) fn from_message(msg: &Message) -> Result<Self, Error> {
        msg.interface()
            .ok_or(Error::SignalError(SignalError::NoMatch(Cow::Borrowed("[Unknown]"))))
            .and_then(|ref dbus_iface| {
                let dbus_iface_s = dbus_iface.as_cstr().to_string_lossy();
                Interface::from_str(&dbus_iface_s)
                    .map_err(|_| Error::SignalError(SignalError::NoMatch(Cow::Borrowed("afddsf"))))
            })
            .and_then(|iface| {
                match iface {
                    Interface::Manager => manager::Signal::from_message(msg).map(|m| Signal::Manager(m)),
                    //Interface::Technology => technology::Signal::from_message(msg).map(|m| Signal::Technology(m)),
                    //Interface::Service => serivce::Signal::from_message(msg).map(|m| Signal::Service(m)),

                }
            })
    }
}
