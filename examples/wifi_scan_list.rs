use std::ops::Deref;
use std::time::Duration;

use connman::api::Error as ConnmanError;
use connman::{Manager, Technology};
use dbus::nonblock::NonblockReply;
use dbus_tokio::connection;

pub async fn get_technology_wifi<T: NonblockReply, C: Deref<Target = T> + Clone>(
    manager: &Manager<C>,
) -> Result<Option<Technology<C>>, ConnmanError> {
    manager
        .get_technologies()
        .await
        // Filter out the wifi technology (eventually this will be a simple library call)
        .map(|v| {
            v.into_iter()
                .find(move |t| t.props.type_ == connman::api::technology::Type::Wifi)
        })
}

#[tokio::main]
async fn main() {
    let (resource, conn) = connection::new_system_sync().unwrap();
    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let manager = Manager::new(conn, Duration::from_secs(10));

    let wifi = get_technology_wifi(&manager).await.unwrap();
    // Initiate scan
    wifi.unwrap().scan().await.unwrap();

    // List services once scan completes
    let services = manager.clone().get_services().await.unwrap();
    for svc in services {
        // Dump service info
        println!("Found service: {:?}", svc.path())
    }
}
