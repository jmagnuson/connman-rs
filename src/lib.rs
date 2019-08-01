//!  ConnMan D-Bus API
//!
//! ## Usage
//!
//! Add connman-rs to your `Cargo.toml` with:
//!
//! ```toml
//! [dependencies]
//! connman = "0.1"
//! ```

//! ## Example
//!
//! The following example demonstrates how to create a `Manager` and list
//! the available services.
//!
//! ```rust,no_run
//! extern crate connman;
//! extern crate dbus;
//! extern crate dbus_tokio;
//! extern crate futures;
//! extern crate tokio;
//!
//! use connman::Manager;
//! use dbus::{BusType, Connection};
//! use dbus_tokio::AConnection;
//! use futures::Future;
//! use tokio::reactor::Handle;
//! use tokio::runtime::current_thread::Runtime;
//!
//! use std::rc::Rc;
//!
//! fn main() {
//!     let mut runtime = Runtime::new().unwrap();
//!
//!     let conn = Rc::new(Connection::get_private(BusType::System).unwrap());
//!     let aconn = Rc::new(AConnection::new(conn.clone(), Handle::default(), &mut runtime).unwrap());
//!
//!     let manager = Manager::new(aconn);
//!
//!     let f = manager.get_services()
//!         .and_then(|services| {
//!             for svc in services {
//!                 // Dump service info
//!                 println!("Found service: {:?}", svc)
//!             }
//!             Ok(())
//!         });
//!
//!     runtime.block_on(f).unwrap();
//! }
//! ```

#![allow(unused)]
#![allow(clippy::redundant_field_names, clippy::let_and_return)]

extern crate dbus;
extern crate dbus_tokio;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate tokio;

#[cfg(feature = "introspection")]
extern crate xml;

pub mod api;

pub use crate::api::{
    manager::Manager,
    service::Service,
    technology::Technology,
};