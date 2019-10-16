extern crate connman;
extern crate dbus;
extern crate dbus_tokio;
extern crate futures;

use std::borrow::Cow;
use std::rc::Rc;
use std::time::Duration;

use connman::api::Error as ConnmanError;
use connman::{Manager, Technology};
use dbus::{BusType, Connection};
use dbus_tokio::AConnection;
use futures::Future;
use tokio::prelude::FutureExt;
use tokio::reactor::Handle;
use tokio::runtime::current_thread::Runtime;

pub fn get_technology_wifi(
    manager: &Manager,
) -> impl Future<Item = Option<Technology>, Error = ConnmanError> {
    manager
        .get_technologies()
        // Filter out the wifi technology (eventually this will be a simple library call)
        .map(|v| {
            v.into_iter().find(move |t| {
                t.props.type_ == connman::api::technology::Type::Wifi
            })
        })
}

fn main() {
    let mut runtime = Runtime::new().unwrap();

    let conn = Rc::new(Connection::get_private(BusType::System).unwrap());
    let aconn = Rc::new(AConnection::new(conn.clone(), Handle::default(), &mut runtime).unwrap());

    let manager = Manager::new(aconn);

    let wifi_scan = get_technology_wifi(&manager)
        // Initiate scan
        .and_then(|wifi| wifi.unwrap().scan()
            .timeout(Duration::from_secs(10))
                .map_err(|e| {
                    let s = format!("{:?}", e);
                    ConnmanError::Timeout(Cow::Owned(s))
                }))
    // List services once scan completes
    .and_then(move |_| manager.clone().get_services())
        .and_then(|services| {
            for svc in services {
                // Dump service info
                println!("Found service: {:?}", svc)
            }
            Ok(())
        });

    runtime.block_on(wifi_scan).unwrap();
}
