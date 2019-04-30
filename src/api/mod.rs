//! D-Bus API implementations

#[rustfmt::skip]
mod gen;

pub mod manager;
pub mod service;
pub mod technology;

use dbus;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    DbusError(#[cause] dbus::Error),
}

impl From<dbus::Error> for Error {
    fn from(e: dbus::Error) -> Self {
        Error::DbusError(e)
    }
}
