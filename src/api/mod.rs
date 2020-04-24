//! D-Bus API implementations

#[rustfmt::skip]
mod gen;

pub mod manager;
pub mod service;
pub mod technology;

use dbus;
use dbus::arg::{cast, RefArg, Variant};
use thiserror::Error;

use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;

type RefArgMap = HashMap<String, Variant<Box<dyn RefArg + 'static>>>;
type RefArgMapRef<'a> = HashMap<String, &'a dyn RefArg>;
type RefArgIter<'a> = Box<dyn Iterator<Item=&'a dyn RefArg> + 'a>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    DbusError(#[from] dbus::Error),
    #[error("'{0}'")]
    PropertyError(#[from] PropertyError),
    #[error("Failed resolve before timeout: '{0}'")]
    Timeout(Cow<'static, str>),
}

#[derive(Debug, Error)]
pub enum PropertyError {
    #[error("Property not present: '{0}'")]
    NotPresent(Cow<'static, str>),
    #[error("Failed to cast property: '{0}'")]
    Cast(Cow<'static, str>),
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
