use std::borrow::Cow;
use std::ops::Deref;
use std::time::Duration;

use connman::api::Error as ConnmanError;
use connman::{Manager, Technology};
use dbus::nonblock::NonblockReply;
use dbus_tokio::connection;
use tokio::time::timeout;

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

    let manager = Manager::new(conn);

    let wifi = get_technology_wifi(&manager).await.unwrap();
    // Initiate scan
    timeout(Duration::from_secs(10), wifi.unwrap().scan())
        .await
        .map_err(|e| {
            let s = format!("{:?}", e);
            ConnmanError::Timeout(Cow::Owned(s))
        })
        .unwrap()
        .unwrap();

    // List services once scan completes
    let services = manager.clone().get_services().await.unwrap();
    for svc in services {
        // Dump service info
        println!("Found service: {:?}", svc.path())
    }
}
