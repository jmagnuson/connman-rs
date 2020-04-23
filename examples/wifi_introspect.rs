use dbus_tokio::connection;


use connman::api::Error as ConnmanError;
use connman::{Manager, Technology};
use connman::api::technology::Type as TechnologyType;

use xml::reader::XmlEvent;

pub async fn get_technology_wifi(
    manager: &Manager,
) -> Result<Option<Technology>, ConnmanError> {
    let technologies = manager.get_technologies().await?;
    Ok(technologies.into_iter().find(move |t| {
        t.props.type_ == TechnologyType::Wifi
    }))
}

// Shamelessly borrowed from the example in `xml-rs` doc:
// https://github.com/netvl/xml-rs#reading-xml-documents
fn indent(size: usize) -> String {
    const INDENT: &'static str = "    ";
    (0..size).map(|_| INDENT)
        .fold(String::with_capacity(size*INDENT.len()), |r, s| r + s)
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let (resource, conn) = connection::new_system_sync().unwrap();

    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let manager = Manager::new(conn);

    let wifi = get_technology_wifi(&manager).await.unwrap().unwrap();

    let event_reader = wifi.introspect().await.unwrap();

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
}
