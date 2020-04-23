use dbus_tokio::connection;
use dbus::nonblock;
use dbus::message::MatchRule;
use futures::StreamExt;

use connman::api::Error as ConnmanError;
use connman::{Manager, Technology};

use futures::future::{TryFuture, TryFutureExt};
use tokio::time::timeout;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Connect to the D-Bus session bus (this is blocking, unfortunately).
    let (resource, conn) = connection::new_system_sync().unwrap();

    // The resource is a task that should be spawned onto a tokio compatible
    // reactor ASAP. If the resource ever finishes, you lost connection to D-Bus.
    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let manager = Manager::new(conn);

    let wifi = get_technology_wifi(&manager).await.unwrap().unwrap();

    wifi.scan().await.unwrap();

    let services = manager.get_services().await.unwrap();

    for svc in services {
        // Dump service info
        println!("Found service: {:?}", svc)
    }

    Ok(())
}

use std::borrow::Cow;
use std::rc::Rc;
use std::time::Duration;

pub async fn get_technology_wifi(
    manager: &Manager,
) -> Result<Option<Technology>, ConnmanError> {
    let technologies = manager.get_technologies().await?;

    Ok(technologies.into_iter().find(move |t| {
        t.props.type_ == connman::api::technology::Type::Wifi
    }))
}
