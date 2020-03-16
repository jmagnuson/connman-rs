use connman::api::technology::Type as TechnologyType;
use connman::api::Error as ConnmanError;
use connman::{Manager, Technology};
use dbus::nonblock::NonblockReply;
use dbus_tokio::connection;
use xml::reader::XmlEvent;

use std::ops::Deref;

pub async fn get_technology_wifi<T: NonblockReply, C: Deref<Target = T> + Clone>(
    manager: &Manager<C>,
) -> Result<Option<Technology<C>>, ConnmanError> {
    manager.get_technologies().await.map(|v| {
        v.into_iter()
            .find(move |t| t.props.type_ == TechnologyType::Wifi)
    })
}

// Shamelessly borrowed from the example in `xml-rs` doc:
// https://github.com/netvl/xml-rs#reading-xml-documents
fn indent(size: usize) -> String {
    const INDENT: &'static str = "    ";
    (0..size)
        .map(|_| INDENT)
        .fold(String::with_capacity(size * INDENT.len()), |r, s| r + s)
}

#[tokio::main]
async fn main() {
    let (resource, conn) = connection::new_system_sync().unwrap();
    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let manager = Manager::new(conn);

    let wifi = get_technology_wifi(&manager).await.unwrap();
    let event_reader = wifi.unwrap().introspect().await.unwrap();

    let mut depth = 0;
    for e in event_reader {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
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
}
