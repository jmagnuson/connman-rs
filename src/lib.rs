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
//! extern crate tokio;
//!
//! use connman::Manager;
//! use dbus::{BusType, Connection};
//! use dbus_tokio::AConnection;
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

pub mod api;

pub use crate::api::{
    manager::Manager,
    service::Service,
    technology::Technology,
};

use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use dbus::{BusType, Connection, Message};
use dbus_tokio::{AMessageStream, AConnection};
use futures::future::Either;
use futures::sync::mpsc;
use futures::{Future, IntoFuture, Sink, Stream};

use tokio::reactor::Handle;
use tokio::runtime::current_thread::Runtime;
use tokio::prelude::FutureExt;
use futures::stream;

use crate::api::Signal;

#[derive(Clone, Debug)]
pub struct SignalsHandle {
    subscriptions: Rc<RefCell<Vec<mpsc::Sender<String>>>>,
}

impl SignalsHandle {
    pub fn new(rt: &mut Runtime, message_stream: AMessageStream) -> Self {

        let subscriptions = Rc::new(RefCell::new(Vec::new()));

        let f = {
            let subs_clone = subscriptions.clone();
            message_stream.and_then(move |msg: Message| {
                let signal_opt = Signal::from_message(&msg);
                // FIXME: NEED TO CLONE MESSAGE SOMEHOW
//                let msg_s = format!("message: {:?}", msg);
//                let msg_clone = msg_s.clone();
//                info!("inner {}", msg_clone);

                let signal = if let Ok(sig) = Signal::from_message(&msg) {
                    format!("{:?}", sig)
                } else {
                    return Either::A(futures::future::ok::<(),()>(()))
                };

                let publish_fut =
                    subs_clone.try_borrow_mut().map_err(|_| (()))
                        .map(move |subs: RefMut<Vec<mpsc::Sender<String>>>| {
                            let msg_clone = signal.clone();
                            let fut_vec: Vec<_> = subs.iter().cloned()
                                .map(move |mut sub| {
                                    sub.send(msg_clone.clone())
                                        //.map_err(())
                                        .then(|res| {
                                            if res.is_err() {
                                                println!("Failed to dispatch message to subscriber");
                                            }
                                            Ok(())
                                        })
                                }).collect();
                            fut_vec
                        })
                        .into_future()
                        .and_then(|fut_vec| stream::futures_unordered(fut_vec).for_each(|_| Ok::<(),()>(())))
                        .then(|_| Ok::<(),()>(()));

                Either::B(publish_fut)
            }).for_each(|_| Ok::<(),()>(()))
        };

        rt.spawn(f);
        SignalsHandle {
            subscriptions
        }
    }

    pub fn subscribe(&mut self) -> impl Future<Item=mpsc::Receiver<String>, Error=&'static str> {
        let (tx, rx) = mpsc::channel::<String>(20);
        self.subscriptions.clone().try_borrow_mut()
            .map_err(|_| "fucked")
            .map(move |mut v| v.push(tx))
            .map(move |_| rx)
            .into_future()
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    connection: Rc<AConnection>,
    sig_handle: SignalsHandle,
}

impl Client {
    pub fn new(rt: &mut Runtime) -> Self {

        let c = Rc::new(Connection::get_private(BusType::System)
            .expect("Failed to initialize d-bus connection"));

        // TODO: c.add_match(Signal::match_str(None, None))
        c.add_match("type=signal,interface=net.connman.Manager").unwrap();
        c.add_match("type=signal,interface=net.connman.Service").unwrap();
        c.add_match("type=signal,interface=net.connman.Technology").unwrap();

        let aconn = AConnection::new(c.clone(), Handle::default(), rt)
            .expect("failed to create aconn");
        let messages = aconn.messages().unwrap();

        let connection = Rc::new(aconn);

        Client {
            connection,
            sig_handle: SignalsHandle::new(rt, messages),
        }
    }

    pub fn manager(&self) -> Manager {
        Manager::new(self.connection.clone())
    }

    pub fn subscribe(&mut self) -> impl Future<Item=mpsc::Receiver<String>, Error=&'static str> {
        self.sig_handle.subscribe()
    }
}

