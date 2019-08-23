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
type RefArgMapRef<'a> = HashMap<String, &'a RefArg>;
type RefArgIter<'a> = Box<Iterator<Item=&'a RefArg> + 'a>;

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
) -> Result<T, PropertyError> {
    properties.get(prop_name)
        .ok_or_else(|| PropertyError::NotPresent(Cow::Borrowed(prop_name)))
        .and_then(|variant|
            cast::<T>(&variant.0).cloned()
                .ok_or_else(|| PropertyError::Cast(Cow::Borrowed(prop_name)))
        )
}

/// Convenience function for getting property values that impl `FromStr`.
fn get_property_fromstr<T: FromStr + 'static>(
    properties: &RefArgMap,
    prop_name: &'static str,
) -> Result<T, PropertyError> {
    properties.get(prop_name)
        .ok_or_else(|| PropertyError::NotPresent(Cow::Borrowed(prop_name)))
        .and_then(|variant| variant.as_str()
            .and_then(|s| T::from_str(s).ok())
            .ok_or_else(|| PropertyError::Cast(Cow::Borrowed(prop_name)))
        )
}

/// Convenience function for getting property values from a Dict or Array.
fn get_property_argiter<'a>(
    properties: &'a RefArgMap,
    prop_name: &'static str,
) -> Result<RefArgIter<'a>, PropertyError> {
    properties.get(prop_name)
        .ok_or_else(|| PropertyError::NotPresent(Cow::Borrowed(prop_name)))
        .and_then(|variant|
            variant.0.as_iter()
                .ok_or_else(|| PropertyError::Cast(Cow::Borrowed(prop_name)))
        )
}

pub trait FromProperties: Sized {
    fn from_properties(properties: &RefArgMap, prop_name: &'static str) -> Result<Self, PropertyError>;
}

impl FromProperties for String {
    fn from_properties(properties: &RefArgMap, prop_name: &'static str) -> Result<Self, PropertyError> {
        get_property_fromstr::<Self>(properties, prop_name)
    }
}

impl <T: FromProperties + Clone + 'static> FromProperties for Vec<T> {
    fn from_properties(properties: &RefArgMap, prop_name: &'static str) -> Result<Self, PropertyError> {
        get_property::<Vec<T>>(properties, prop_name)
    }
}

impl FromProperties for u8 {
    fn from_properties(properties: &RefArgMap, prop_name: &'static str) -> Result<Self, PropertyError> {
        get_property::<Self>(properties, prop_name)
    }
}

impl FromProperties for bool {
    fn from_properties(properties: &RefArgMap, prop_name: &'static str) -> Result<Self, PropertyError> {
        get_property::<Self>(properties, prop_name)
    }
}

impl <T: FromProperties> FromProperties for Option<T> {
    fn from_properties(properties: &RefArgMap, prop_name: &'static str) -> Result<Self, PropertyError> {
        match T::from_properties(properties, prop_name) {
            Err(PropertyError::NotPresent(_)) => Ok(None),
            Ok(s) => Ok(Some(s)),
            res => res.map(Option::Some),
        }
    }
}
