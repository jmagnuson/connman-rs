//! D-Bus API implementations

#[rustfmt::skip]
mod gen;

pub mod manager;
pub mod service;
pub mod technology;

use dbus;
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
}

#[derive(Debug, Fail)]
pub enum PropertyError {
    #[fail(display = "Property not present: '{}'", _0)]
    NotPresent(Cow<'static, str>),
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
