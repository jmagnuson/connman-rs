//! D-Bus API implementations

#[rustfmt::skip]
mod gen;

pub mod manager;
pub mod service;
pub mod technology;

use dbus;

use std::borrow::Cow;

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
