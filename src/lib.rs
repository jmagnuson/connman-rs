//!  ConnMan D-Bus API
//!
//! ## Usage
//!
//! Add connman-rs to your `Cargo.toml` with:
//!
//! ```toml
//! [dependencies]
//! connman = "0.2"
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
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (resource, conn) = connection::new_system_sync().unwrap();
//!     tokio::spawn(async {
//!         let err = resource.await;
//!         panic!("Lost connection to D-Bus: {}", err);
//!     });
//!
//!     let manager = Manager::new(conn, Duration::from_secs(10));
//!
//!     let services = manager.get_services().await.unwrap();
//!     for svc in services {
//!         // Dump service info
//!         println!("Found service: {:?}", svc.path())
//!     }
//! }
//! ```

#![allow(unused)]
#![allow(clippy::redundant_field_names, clippy::let_and_return)]

pub mod api;

pub use crate::api::{manager::Manager, service::Service, technology::Technology};
