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
use xml::reader::XmlEvent;


pub fn get_technology_wifi(
    manager: &Manager,
) -> impl Future<Item = Option<Technology>, Error = ConnmanError> {
    manager
        .get_technologies()
        .map(|v| {
            v.into_iter().find(move |t| {
                t.args.get("Type").and_then(|variant| variant.as_str()) == Some("wifi")
            })
        })
}

// Shamelessly borrowed from the example in `xml-rs` doc:
// https://github.com/netvl/xml-rs#reading-xml-documents
fn indent(size: usize) -> String {
    const INDENT: &'static str = "    ";
    (0..size).map(|_| INDENT)
        .fold(String::with_capacity(size*INDENT.len()), |r, s| r + s)
}

fn main() {
    let mut runtime = Runtime::new().unwrap();

    let conn = Rc::new(Connection::get_private(BusType::System).unwrap());
    let aconn = Rc::new(AConnection::new(conn.clone(), Handle::default(), &mut runtime).unwrap());

    let manager = Manager::new(aconn);

    let wifi_scan = get_technology_wifi(&manager)
        .and_then(|wifi| wifi.unwrap().introspect())
        .and_then(|event_reader| {
            let mut depth = 0;
            for e in event_reader {
                match e {
                    Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                        println!("{}+{}, {:?}", indent(depth), name, attributes);
                        depth += 1;
                    }
                    Ok(XmlEvent::EndElement { name }) => {
                        depth -= 1;
                        println!("{}-{}", indent(depth), name);
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            Ok(())
        });

    runtime.block_on(wifi_scan).unwrap();
}
