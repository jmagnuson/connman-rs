extern crate connman;
extern crate dbus;
extern crate dbus_tokio;
extern crate futures;
extern crate hex;
extern crate structopt;

use std::borrow::Cow;
use std::io;
use std::rc::Rc;

use connman::api::Error as ConnmanError;
use connman::{Manager, Technology};
use dbus::{BusType, Connection};
use dbus_tokio::AConnection;
use futures::{future::Either, Future, IntoFuture};
use std::time::Duration;
use structopt::StructOpt;
use tokio::prelude::FutureExt;
use tokio::reactor::Handle;
use tokio::runtime::current_thread::Runtime;

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

pub fn get_technology_wifi(
    manager: &Manager,
) -> impl Future<Item = Option<Technology>, Error = ConnmanError> {
    manager
        .get_technologies()
        // Filter out the wifi technology (eventually this will be a simple library call)
        .map(|v| {
            v.into_iter()
                .find(move |t| t.props.type_ == connman::api::technology::Type::Wifi)
        })
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

fn main() {
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
        .and_then(|_| manager.clone().get_services())
        .and_then(|services| {
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
                    return Ok(Some(svc));
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
            Ok(None)
        })
        .and_then(|maybe_svc| {
            if let Some(svc) = maybe_svc {
                let f = if args.disconnect {
                    println!("Disconnecting service: {:?}", svc.path());
                    Either::A(svc.disconnect())
                } else {
                    println!("Connecting to service: {:?}", svc.path());
                    Either::B(svc.connect())
                };
                Either::A(
                    f.timeout(Duration::from_secs(10))
                        .map_err(|e| {
                            let s = format!("{:?}", e);
                            ConnmanError::Timeout(Cow::Owned(s))
                        })
                        .map(|_| Some(svc)),
                )
            } else {
                Either::B(Ok(maybe_svc).into_future())
            }
        })
        .and_then(|maybe_svc| {
            manager.clone().get_state().map(|state| {
                println!("Connection state: {:?}", state);
                maybe_svc
            })
        });

    let res = runtime.block_on(wifi_scan).unwrap();

    println!("exit: {:?}", res);
}
