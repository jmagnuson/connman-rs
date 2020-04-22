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
//! use connman::Manager;
//! use dbus_tokio::connection;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (resource, aconn) = connection::new_system_sync().unwrap();
//!
//!     tokio::spawn(async {
//!         let err = resource.await;
//!         panic!("Lost connection to d-bus: {}", err);
//!     });
//!
//!     let manager = Manager::new(aconn);
//!
//!     let services = manager.get_services().await.unwrap();
//!
//!     for svc in services {
//!         // Dump service info
//!         println!("Found service: {:?}", svc)
//!     }
//!     Ok(())
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
