use dbus_tokio::connection;

use connman::api::Error as ConnmanError;
use connman::{Manager, Technology};

use std::io;
use structopt::StructOpt;
use tokio::time::timeout;
use std::time::Duration;

// TODO: Is there a way to determine this path from connman?
const WIFI_SERVICE_CONFIG_FILE: &str = "/usr/local/var/lib/connman/wifi.config";

#[derive(Debug, StructOpt)]
#[structopt(about = "Connects to a SSID with ConnMan over D-Bus")]
struct WifiConnectOpts {
    /// Issue a disconnect (defauls is connect)
    #[structopt(short, long)]
    disconnect: bool,

    /// SSID password (if necessary)
    #[structopt(short = "p", long = "password")]
    password: Option<String>,

    /// SSID
    #[structopt(required = true)]
    ssid: String,
}

pub async fn get_technology_wifi(
    manager: &Manager,
) -> Result<Option<Technology>, ConnmanError> {
    let technologies = manager.get_technologies().await?;

    // Filter out the wifi technology (eventually this will be a simple library call)
    Ok(technologies.into_iter().find(move |t| {
        t.props.type_ == connman::api::technology::Type::Wifi
    }))
}

#[rustfmt::skip]
pub fn generate_wifi_config(ssid: &str, password: Option<&str>) -> String {
    let hex_ssid = hex::encode(ssid).to_uppercase();
    if let Some(pass) = password {
        format!("[global]
Name = Wi-Fi
Description = Wi-Fi configuration

[service_wifi]
Type = wifi
SSID = {}
Security = psk
IPv4 = 192.168.1.2/255.255.255.0/192.168.1.1
IPv6=off
Passphrase = {}
Hidden = false", hex_ssid, pass)
    } else {
        format!("[global]
Name = Wi-Fi
Description = Wi-Fi configuration

[service_wifi]
Type = wifi
SSID = {}
Security = open
IPv4 = 192.168.1.2/255.255.255.0/192.168.1.1
IPv6=off
Hidden = false", hex_ssid)
    }
}

pub fn write_wifi_service_config(s: &str) -> Result<(), io::Error> {
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    let path = Path::new(WIFI_SERVICE_CONFIG_FILE);

    let mut file = File::create(&path)?;

    file.write_all(s.as_bytes())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = WifiConnectOpts::from_args();

    if !args.disconnect {
        let prov = generate_wifi_config(
            args.ssid.as_str(),
            args.password.as_ref().map(|s| s.as_str()),
        );

        write_wifi_service_config(prov.as_str()).expect("Failed to write wifi service config");

        println!("{}", prov);
    }

    let hex_ssid = hex::encode(&args.ssid);

    let (resource, conn) = connection::new_system_sync().unwrap();

    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let manager = Manager::new(conn);

    let wifi = get_technology_wifi(&manager).await.unwrap().unwrap();

    wifi.scan().await.unwrap();

    let services = manager.clone().get_services().await.unwrap();

    let mut maybe_svc = None;

    for svc in services {
        //wifi_ffffffffffff_00112233aabbccdd_managed_psk
        //tech_mac.addr...._hex.ssid........_security...
        let pathv = svc
            .path()
            .as_cstr()
            .to_str()
            .unwrap()
            .split("_")
            .collect::<Vec<&str>>();
        let svc_hex_ssid = *pathv.get(2).unwrap();
        if svc_hex_ssid == hex_ssid {
            println!("Found service: {:?}", svc);
            let _none = maybe_svc.replace(svc);
            break;
        } else {
            let svc_ssid_str = hex::decode(&svc_hex_ssid)
                .map(|s| String::from_utf8(s).expect("Failed to turn ssid into string"));
            if let Ok(ssid) = svc_ssid_str {
                println!("{} != {}", ssid, args.ssid);
            } else {
                println!("Failed to decode hex string: {}", svc_hex_ssid);
            }
        }
    }
    if let Some(svc) = maybe_svc {
        if args.disconnect {
            println!("Disconnecting service: {:?}", svc.path());
            svc.disconnect().await.unwrap()
        } else {
            println!("Connecting to service: {:?}", svc.path());
            svc.connect().await.unwrap()
        }
    }

    let state = manager.get_state().await.unwrap();
    println!("Connection state: {:?}", state);

    Ok(())
}
