extern crate connman;
extern crate dbus;
extern crate dbus_tokio;
extern crate futures;

use std::rc::Rc;

use connman::api::Error as ConnmanError;
use connman::{Manager, Technology};
use dbus::arg::RefArg;
use dbus::{BusType, Connection};
use dbus_tokio::AConnection;
use futures::Future;
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
                t.args.get("Type").and_then(|variant| variant.as_str()) == Some("wifi")
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
        .and_then(|wifi| wifi.unwrap().scan())
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
