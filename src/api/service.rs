use dbus::arg;
use dbus::nonblock::{NonblockReply, Proxy as DBusProxy, SyncConnection};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;

#[cfg(feature = "introspection")]
use xml::reader::EventReader;

use super::gen::service::Service as IService;
use super::Error as ApiError;
use super::{FromProperties, PropertyError};
use crate::api::{get_property_argiter, RefArgIter, RefArgMap};
use dbus::arg::{cast, RefArg, Variant};
use std::convert::TryFrom;
use std::ops::Deref;
use std::time::Duration;

/// Futures-aware wrapper struct for connman Service object.
pub struct Service<C> {
    proxy: DBusProxy<'static, C>,
    pub props: Properties,
}

impl<C> Service<C> {
    pub fn new(
        connection: C,
        path: dbus::Path<'static>,
        args: RefArgMap,
        timeout: Duration,
    ) -> Result<Self, ApiError> {
        let properties = Properties::try_from(args).map_err(ApiError::from)?;

        Ok(Service {
            proxy: Self::proxy(path, timeout, connection),
            props: properties,
        })
    }

    pub fn proxy(path: dbus::Path<'static>, timeout: Duration, conn: C) -> DBusProxy<'static, C> {
        let proxy = DBusProxy::new("net.connman", path, timeout, conn);
        proxy
    }

    pub fn path(&self) -> &dbus::Path<'static> {
        &self.proxy.path
    }
}

impl<T: NonblockReply, C: Deref<Target = T>> Service<C> {
    #[cfg(feature = "introspection")]
    pub async fn introspect(&self) -> Result<EventReader<std::io::Cursor<Vec<u8>>>, ApiError> {
        use crate::api::gen::service::OrgFreedesktopDBusIntrospectable as Introspectable;

        let s = Introspectable::introspect(&self.proxy).await?;
        let rdr = std::io::Cursor::new(s.into_bytes());
        Ok(EventReader::new(rdr))
    }

    pub async fn connect(&self) -> Result<(), ApiError> {
        Ok(IService::connect(&self.proxy).await?)
    }

    pub async fn disconnect(&self) -> Result<(), ApiError> {
        Ok(IService::disconnect(&self.proxy).await?)
    }

    pub async fn remove(&self) -> Result<(), ApiError> {
        Ok(IService::remove(&self.proxy).await?)
    }

    pub async fn move_before(&self, service: &Service<C>) -> Result<(), ApiError> {
        Ok(IService::move_before(&self.proxy, service.path().clone()).await?)
    }

    pub async fn move_after(&self, service: &Service<C>) -> Result<(), ApiError> {
        Ok(IService::move_after(&self.proxy, service.path().clone()).await?)
    }
}

#[derive(Debug)]
pub struct Properties {
    /// Connection state
    pub state: State,
    /// Error reason; only valid for `State::Failure`
    pub error: Option<Error>,
    /// Service name
    pub name: Option<String>,
    /// Service name
    pub type_: Option<Type>,
    /// Service name
    // TODO: enum variants?
    pub security: Option<Vec<String>>,
    /// Signal strength
    pub strength: Option<u8>,
    /// Set if favorite or User-selected
    pub favorite: bool,
    /// Set if configured externally
    pub immutable: bool,
    /// Whether or not to automatically connect if no other connection
    pub autoconnect: bool,
    /// Set if service is roaming
    pub roaming: Option<bool>,
    /// List of currently-active nameservers
    pub nameservers: Vec<String>, // TODO: Deserialize `String` into `IpAddr`?
    /// List of manually-configured nameservers
    pub nameservers_config: Vec<String>, // TODO: Deserialize `String` into `IpAddr`?
    /// List of currently-active timeservers
    pub timeservers: Vec<String>,
    /// List of manually-configured timeservers
    pub timeservers_config: Vec<String>,
    /// List of currently-used search domains
    pub domains: Vec<String>,
    /// List of manually-configured search domains
    pub domains_config: Vec<String>,
    /// Ipv4 related information
    pub ipv4: Ipv4,
    /// Ipv4 config related information
    pub ipv4_config: Ipv4,
    /// Ipv6 related information
    pub ipv6: Ipv6,
    /// Ipv6 config related information
    pub ipv6_config: Ipv6,
    /// Proxy related information
    pub proxy: Proxy,
    /// Proxy config related information
    pub proxy_config: Proxy,
    /// Provider (VPN) related information
    pub provider: Provider,
    /// Ethernet related information
    pub ethernet: Ethernet,
    /// Whether or not mDNS support is enabled
    pub mdns: Option<bool>,
    /// Whether or not mDNS (config) support is enabled
    pub mdns_config: Option<bool>,
}

impl FromProperties for State {
    fn from_properties(
        properties: &RefArgMap,
        prop_name: &'static str,
    ) -> Result<Self, PropertyError> {
        super::get_property_fromstr::<Self>(properties, prop_name)
    }
}

impl FromProperties for Error {
    fn from_properties(
        properties: &RefArgMap,
        prop_name: &'static str,
    ) -> Result<Self, PropertyError> {
        super::get_property_fromstr::<Self>(properties, prop_name)
    }
}

impl FromProperties for Type {
    fn from_properties(
        properties: &RefArgMap,
        prop_name: &'static str,
    ) -> Result<Self, PropertyError> {
        super::get_property_fromstr::<Self>(properties, prop_name)
    }
}
impl FromProperties for Ipv4 {
    fn from_properties(
        properties: &RefArgMap,
        prop_name: &'static str,
    ) -> Result<Self, PropertyError> {
        let mut i = get_property_argiter(properties, prop_name)?;
        let mut m: HashMap<String, String> = HashMap::new();
        while let Some(key) = i.next().and_then(|k| k.as_str()) {
            if let Some(val) = i.next().and_then(|v| v.as_str()) {
                let _ = m.insert(key.to_string(), val.to_string());
            }
        }

        let method = if let Some(method) = m.get(Ipv4Kind::Method.into()) {
            Some(method.parse::<Ipv4Method>()?)
        } else {
            None
        };
        let address = m.get(Ipv4Kind::Address.into()).cloned();
        let netmask = m.get(Ipv4Kind::Netmask.into()).cloned();
        let gateway = m.get(Ipv4Kind::Gateway.into()).cloned();

        Ok(Ipv4 {
            method,
            address,
            netmask,
            gateway,
        })
    }
}

impl FromProperties for Ipv6 {
    fn from_properties(
        properties: &RefArgMap,
        prop_name: &'static str,
    ) -> Result<Self, PropertyError> {
        let mut i = get_property_argiter(properties, prop_name)?;
        let mut m: HashMap<&str, &str> = HashMap::new();
        while let Some(key) = i.next().and_then(|k| k.as_str()) {
            if let Some(val) = i.next().and_then(|v| v.as_str()) {
                let _ = m.insert(key, val);
            }
        }

        let method = m
            .get(Ipv6Kind::Method.into())
            .and_then(|method| method.parse::<Ipv6Method>().ok());

        let address = m.get(Ipv6Kind::Address.into()).copied().map(String::from);
        let prefix_length = m
            .get(Ipv6Kind::PrefixLength.into())
            .and_then(|val| val.parse::<u8>().ok());

        let gateway = m.get(Ipv6Kind::Gateway.into()).copied().map(String::from);
        let privacy = m
            .get(Ipv6Kind::Privacy.into())
            .and_then(|privacy| privacy.parse::<Ipv6Privacy>().ok());

        Ok(Ipv6 {
            method,
            address,
            prefix_length,
            gateway,
            privacy,
        })
    }
}

impl FromProperties for Proxy {
    fn from_properties(
        properties: &RefArgMap,
        prop_name: &'static str,
    ) -> Result<Self, PropertyError> {
        let mut i = get_property_argiter(properties, prop_name)?;
        let mut m: HashMap<&str, &dyn RefArg> = HashMap::new();
        while let Some(key) = i.next().and_then(|k| k.as_str()) {
            if let Some(val) = i.next() {
                let _ = m.insert(key, val);
            }
        }

        let method = m
            .get(ProxyKind::Method.into())
            .and_then(|refarg| refarg.as_str())
            .and_then(|s| ProxyMethod::from_str(s).ok());

        let url = m
            .get(ProxyKind::Url.into())
            .and_then(|refarg| refarg.as_str())
            .map(String::from);

        let servers = m
            .get(ProxyKind::Servers.into())
            .and_then(|refarg| cast::<Vec<String>>(&refarg.box_clone()).cloned());

        let excludes = m
            .get(ProxyKind::Excludes.into())
            .and_then(|refarg| cast::<Vec<String>>(&refarg.box_clone()).cloned());

        Ok(Proxy {
            method,
            url,
            servers,
            excludes,
        })
    }
}

impl FromProperties for Provider {
    fn from_properties(
        properties: &RefArgMap,
        prop_name: &'static str,
    ) -> Result<Self, PropertyError> {
        let mut i = get_property_argiter(properties, prop_name)?;
        let mut m: HashMap<&str, &str> = HashMap::new();
        while let Some(key) = i.next().and_then(|k| k.as_str()) {
            if let Some(val) = i.next().and_then(|v| v.as_str()) {
                let _ = m.insert(key, val);
            }
        }

        let host = m.get(ProviderKind::Host.into()).copied().map(String::from);
        let domain = m
            .get(ProviderKind::Domain.into())
            .copied()
            .map(String::from);
        let name = m.get(ProviderKind::Name.into()).copied().map(String::from);
        let type_ = m.get(ProviderKind::Type.into()).copied().map(String::from);

        Ok(Provider {
            host,
            domain,
            name,
            type_,
        })
    }
}

impl FromProperties for Ethernet {
    fn from_properties(
        properties: &RefArgMap,
        prop_name: &'static str,
    ) -> Result<Self, PropertyError> {
        let mut i = get_property_argiter(properties, prop_name)?;
        let mut eth = Ethernet {
            method: None,
            interface: None,
            address: None,
            mtu: None,
        };
        while let Some(key) = i.next().and_then(|k| k.as_str()) {
            let kind = EthernetKind::from_str(key).expect("Unhandled EthernetKind variant");
            match kind {
                EthernetKind::Method => {
                    if let Some(method) = i
                        .next()
                        .and_then(|v| v.as_str())
                        .and_then(|v| EthernetMethod::from_str(v).ok())
                    {
                        eth.method = Some(method);
                    };
                }
                EthernetKind::Interface => {
                    if let Some(iface) = i.next().and_then(|v| v.as_str()) {
                        eth.interface = Some(iface.to_string());
                    };
                }
                EthernetKind::Address => {
                    if let Some(addr) = i.next().and_then(|v| v.as_str()) {
                        eth.address = Some(addr.to_string());
                    };
                }
                EthernetKind::Mtu => {
                    if let Some(mtu) = i.next().and_then(|v| v.as_u64()) {
                        eth.mtu = Some(mtu as u16);
                    };
                }
            }
        }
        Ok(eth)
    }
}

impl Properties {
    pub fn try_from(props: RefArgMap) -> Result<Self, PropertyError> {
        let state = State::from_properties(&props, PropertyKind::State.into())?;

        let error: Option<Error> =
            FromProperties::from_properties(&props, PropertyKind::Error.into())?;

        let name: Option<String> =
            FromProperties::from_properties(&props, PropertyKind::Name.into())?;

        let type_: Option<Type> =
            FromProperties::from_properties(&props, PropertyKind::Type.into())?;

        let security: Option<Vec<String>> =
            FromProperties::from_properties(&props, PropertyKind::Security.into())?;

        let strength: Option<u8> =
            FromProperties::from_properties(&props, PropertyKind::Strength.into())?;

        let favorite = bool::from_properties(&props, PropertyKind::Favorite.into())?;
        let immutable = bool::from_properties(&props, PropertyKind::Immutable.into())?;
        let autoconnect = bool::from_properties(&props, PropertyKind::AutoConnect.into())?;

        let roaming: Option<bool> =
            FromProperties::from_properties(&props, PropertyKind::Roaming.into())?;

        let nameservers: Vec<String> =
            FromProperties::from_properties(&props, PropertyKind::Nameservers.into())?;
        let nameservers_config: Vec<String> =
            FromProperties::from_properties(&props, PropertyKind::NameserversConfiguration.into())?;
        let timeservers: Vec<String> =
            FromProperties::from_properties(&props, PropertyKind::Timeservers.into())?;
        let timeservers_config: Vec<String> =
            FromProperties::from_properties(&props, PropertyKind::TimeserversConfiguration.into())?;
        let domains: Vec<String> =
            FromProperties::from_properties(&props, PropertyKind::Domains.into())?;
        let domains_config: Vec<String> =
            FromProperties::from_properties(&props, PropertyKind::DomainsConfiguration.into())?;

        let ipv4 = Ipv4::from_properties(&props, PropertyKind::Ipv4.into())?;
        let ipv4_config = Ipv4::from_properties(&props, PropertyKind::Ipv4Configuration.into())?;
        let ipv6 = Ipv6::from_properties(&props, PropertyKind::Ipv6.into())?;
        let ipv6_config = Ipv6::from_properties(&props, PropertyKind::Ipv6Configuration.into())?;

        let proxy = Proxy::from_properties(&props, PropertyKind::Proxy.into())?;
        let proxy_config = Proxy::from_properties(&props, PropertyKind::ProxyConfiguration.into())?;

        let provider = Provider::from_properties(&props, PropertyKind::Provider.into())?;

        let ethernet = Ethernet::from_properties(&props, PropertyKind::Ethernet.into())?;

        let mdns: Option<bool> =
            FromProperties::from_properties(&props, PropertyKind::Mdns.into())?;
        let mdns_config: Option<bool> =
            FromProperties::from_properties(&props, PropertyKind::MdnsConfiguration.into())?;

        Ok(Properties {
            state,
            error,
            name,
            type_,
            security,
            strength,
            favorite,
            immutable,
            autoconnect,
            roaming,
            nameservers,
            nameservers_config,
            timeservers,
            timeservers_config,
            domains,
            domains_config,
            ipv4,
            ipv4_config,
            ipv6,
            ipv6_config,
            proxy,
            proxy_config,
            provider,
            ethernet,
            mdns,
            mdns_config,
        })
    }
}

/// Service property fields.
enum PropertyKind {
    State,
    Error,
    Name,
    Type,
    Security,
    Strength,
    Favorite,
    Immutable,
    AutoConnect,
    Roaming,
    Nameservers,
    NameserversConfiguration,
    Timeservers,
    TimeserversConfiguration,
    Domains,
    DomainsConfiguration,
    Ipv4,
    Ipv4Configuration,
    Ipv6,
    Ipv6Configuration,
    Proxy,
    ProxyConfiguration,
    Provider,
    Ethernet,
    Mdns,
    MdnsConfiguration,
}

impl From<PropertyKind> for &'static str {
    fn from(prop: PropertyKind) -> Self {
        match prop {
            PropertyKind::State => "State",
            PropertyKind::Error => "Error",
            PropertyKind::Name => "Name",
            PropertyKind::Type => "Type",
            PropertyKind::Security => "Security",
            PropertyKind::Strength => "Strength",
            PropertyKind::Favorite => "Favorite",
            PropertyKind::Immutable => "Immutable",
            PropertyKind::AutoConnect => "AutoConnect",
            PropertyKind::Roaming => "Roaming",
            PropertyKind::Nameservers => "Nameservers",
            PropertyKind::NameserversConfiguration => "Nameservers.Configuration",
            PropertyKind::Timeservers => "Timeservers",
            PropertyKind::TimeserversConfiguration => "Timeservers.Configuration",
            PropertyKind::Domains => "Domains",
            PropertyKind::DomainsConfiguration => "Domains.Configuration",
            PropertyKind::Ipv4 => "IPv4",
            PropertyKind::Ipv4Configuration => "IPv4.Configuration",
            PropertyKind::Ipv6 => "IPv6",
            PropertyKind::Ipv6Configuration => "IPv6.Configuration",
            PropertyKind::Proxy => "Proxy",
            PropertyKind::ProxyConfiguration => "Proxy.Configuration",
            PropertyKind::Provider => "Provider",
            PropertyKind::Ethernet => "Ethernet",
            PropertyKind::Mdns => "mDNS",
            PropertyKind::MdnsConfiguration => "mDNS.Configuration",
        }
    }
}

impl FromStr for PropertyKind {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "State" => Ok(PropertyKind::State),
            "Error" => Ok(PropertyKind::Error),
            "Name" => Ok(PropertyKind::Name),
            "Type" => Ok(PropertyKind::Type),
            "Security" => Ok(PropertyKind::Security),
            "Strength" => Ok(PropertyKind::Strength),
            "Favorite" => Ok(PropertyKind::Favorite),
            "Immutable" => Ok(PropertyKind::Immutable),
            "AutoConnect" => Ok(PropertyKind::AutoConnect),
            "Roaming" => Ok(PropertyKind::Roaming),
            "Nameservers" => Ok(PropertyKind::Nameservers),
            "Nameservers.Configuration" => Ok(PropertyKind::NameserversConfiguration),
            "Timeservers" => Ok(PropertyKind::Timeservers),
            "Timeservers.Configuration" => Ok(PropertyKind::TimeserversConfiguration),
            "Domains" => Ok(PropertyKind::Domains),
            "Domains.Configuration" => Ok(PropertyKind::DomainsConfiguration),
            "IPv4" => Ok(PropertyKind::Ipv4),
            "IPv4.Configuration" => Ok(PropertyKind::Ipv4Configuration),
            "IPv6" => Ok(PropertyKind::Ipv6),
            "IPv6.Configuration" => Ok(PropertyKind::Ipv6Configuration),
            "Proxy" => Ok(PropertyKind::Proxy),
            "Proxy.Configuration" => Ok(PropertyKind::ProxyConfiguration),
            "Provider" => Ok(PropertyKind::Provider),
            "Ethernet" => Ok(PropertyKind::Ethernet),
            "mDNS" => Ok(PropertyKind::Mdns),
            "mDNS.Configuration" => Ok(PropertyKind::MdnsConfiguration),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string())).into()),
        }
    }
}

/// Service connection state, `from_str` maps the values given over d-bus by
/// connman.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum State {
    Idle,
    Failure,
    Association,
    Configuration,
    Ready,
    Disconnect,
    Online,
}

impl FromStr for State {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "idle" => Ok(State::Idle),
            "failure" => Ok(State::Failure),
            "association" => Ok(State::Association),
            "configuration" => Ok(State::Configuration),
            "ready" => Ok(State::Ready),
            "disconnect" => Ok(State::Disconnect),
            "online" => Ok(State::Online),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string())).into()),
        }
    }
}

/// Service error reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    OutOfRange,
    PinMissing,
    DhcpFailed,
    ConnectFailed,
    LoginFailed,
    AuthFailed,
    InvalidKey,
}

impl FromStr for Error {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "out-of-range" => Ok(Error::OutOfRange),
            "pin-missing" => Ok(Error::PinMissing),
            "dhcp-failed" => Ok(Error::DhcpFailed),
            "connect-failed" => Ok(Error::ConnectFailed),
            "login-failed" => Ok(Error::LoginFailed),
            "auth-failed" => Ok(Error::AuthFailed),
            "invalid-key" => Ok(Error::InvalidKey),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string())).into()),
        }
    }
}

/// Service type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Wifi,
    Ethernet,
    Unknown(String),
}

impl FromStr for Type {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "wifi" => Ok(Type::Wifi),
            "ethernet" => Ok(Type::Ethernet),
            _ => Ok(Type::Unknown(s.to_string())),
        }
    }
}

/// Ipv4 structure
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ipv4 {
    pub method: Option<Ipv4Method>,
    pub address: Option<String>,
    pub netmask: Option<String>,
    pub gateway: Option<String>,
}

pub enum Ipv4Kind {
    Method,
    Address,
    Netmask,
    Gateway,
}

impl FromStr for Ipv4Kind {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Method" => Ok(Ipv4Kind::Method),
            "Address" => Ok(Ipv4Kind::Address),
            "Netmask" => Ok(Ipv4Kind::Netmask),
            "Gateway" => Ok(Ipv4Kind::Gateway),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string()))),
        }
    }
}

impl From<Ipv4Kind> for &'static str {
    fn from(prop: Ipv4Kind) -> Self {
        match prop {
            Ipv4Kind::Method => "Method",
            Ipv4Kind::Address => "Address",
            Ipv4Kind::Netmask => "Netmask",
            Ipv4Kind::Gateway => "Gateway",
        }
    }
}

/// Ipv4 method type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ipv4Method {
    Dhcp,
    Manual,
    Auto,
    Off,
    Fixed,
}

impl FromStr for Ipv4Method {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dhcp" => Ok(Ipv4Method::Dhcp),
            "manual" => Ok(Ipv4Method::Manual),
            "auto" => Ok(Ipv4Method::Auto),
            "off" => Ok(Ipv4Method::Off),
            "fixed" => Ok(Ipv4Method::Fixed),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string()))),
        }
    }
}

/// Ipv6 structure
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ipv6 {
    pub method: Option<Ipv6Method>,
    pub address: Option<String>,
    pub prefix_length: Option<u8>,
    pub gateway: Option<String>,
    pub privacy: Option<Ipv6Privacy>,
}

pub enum Ipv6Kind {
    Method,
    Address,
    PrefixLength,
    Gateway,
    Privacy,
}

impl FromStr for Ipv6Kind {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Method" => Ok(Ipv6Kind::Method),
            "Address" => Ok(Ipv6Kind::Address),
            "PrefixLength" => Ok(Ipv6Kind::PrefixLength),
            "Gateway" => Ok(Ipv6Kind::Gateway),
            "Privacy" => Ok(Ipv6Kind::Privacy),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string()))),
        }
    }
}

impl From<Ipv6Kind> for &'static str {
    fn from(prop: Ipv6Kind) -> Self {
        match prop {
            Ipv6Kind::Method => "Method",
            Ipv6Kind::Address => "Address",
            Ipv6Kind::PrefixLength => "PrefixLength",
            Ipv6Kind::Gateway => "Gateway",
            Ipv6Kind::Privacy => "Privacy",
        }
    }
}

/// Ipv6 method type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ipv6Method {
    Auto,
    Manual,
    SixToFour,
    Off,
}

impl FromStr for Ipv6Method {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Ipv6Method::Auto),
            "manual" => Ok(Ipv6Method::Manual),
            "6to4" => Ok(Ipv6Method::SixToFour),
            "off" => Ok(Ipv6Method::Off),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string()))),
        }
    }
}

/// Ipv6 privacy type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ipv6Privacy {
    Disabled,
    Enabled,
    // The spelling used by connman
    Prefered,
}

impl FromStr for Ipv6Privacy {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disabled" => Ok(Ipv6Privacy::Disabled),
            "enabled" => Ok(Ipv6Privacy::Enabled),
            // The spelling used by connman
            "prefered" => Ok(Ipv6Privacy::Prefered),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string()))),
        }
    }
}

/// Proxy structure
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proxy {
    pub method: Option<ProxyMethod>,
    pub url: Option<String>,
    pub servers: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
}

pub enum ProxyKind {
    Method,
    Url,
    Servers,
    Excludes,
}

impl FromStr for ProxyKind {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Method" => Ok(ProxyKind::Method),
            "Url" => Ok(ProxyKind::Url),
            "Servers" => Ok(ProxyKind::Servers),
            "Excludes" => Ok(ProxyKind::Excludes),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string()))),
        }
    }
}

impl From<ProxyKind> for &'static str {
    fn from(prop: ProxyKind) -> Self {
        match prop {
            ProxyKind::Method => "Method",
            ProxyKind::Url => "Url",
            ProxyKind::Servers => "Servers",
            ProxyKind::Excludes => "Excludes",
        }
    }
}

/// Proxy method type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProxyMethod {
    Direct,
    Auto,
    Manual,
}

impl FromStr for ProxyMethod {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "direct" => Ok(ProxyMethod::Direct),
            "auto" => Ok(ProxyMethod::Auto),
            "manual" => Ok(ProxyMethod::Manual),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string()))),
        }
    }
}

impl From<ProxyMethod> for &'static str {
    fn from(method: ProxyMethod) -> Self {
        match method {
            ProxyMethod::Direct => "direct",
            ProxyMethod::Auto => "auto",
            ProxyMethod::Manual => "manual",
        }
    }
}

/// Provider structure
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provider {
    pub host: Option<String>,
    pub domain: Option<String>,
    pub name: Option<String>,
    pub type_: Option<String>,
}

pub enum ProviderKind {
    Host,
    Domain,
    Name,
    Type,
}

impl FromStr for ProviderKind {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Host" => Ok(ProviderKind::Host),
            "Domain" => Ok(ProviderKind::Domain),
            "Name" => Ok(ProviderKind::Name),
            "Type" => Ok(ProviderKind::Type),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string()))),
        }
    }
}

impl From<ProviderKind> for &'static str {
    fn from(prop: ProviderKind) -> Self {
        match prop {
            ProviderKind::Host => "Host",
            ProviderKind::Domain => "Domain",
            ProviderKind::Name => "Name",
            ProviderKind::Type => "Type",
        }
    }
}

/// Provider structure
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ethernet {
    pub method: Option<EthernetMethod>,
    pub interface: Option<String>,
    pub address: Option<String>,
    pub mtu: Option<u16>,
    // Deprecated:
    //pub speed: u16,
    //pub duplex: String
}

pub enum EthernetKind {
    Method,
    Interface,
    Address,
    Mtu,
}

impl FromStr for EthernetKind {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Method" => Ok(EthernetKind::Method),
            "Interface" => Ok(EthernetKind::Interface),
            "Address" => Ok(EthernetKind::Address),
            "MTU" => Ok(EthernetKind::Mtu),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string()))),
        }
    }
}

impl From<EthernetKind> for &'static str {
    fn from(prop: EthernetKind) -> Self {
        match prop {
            EthernetKind::Method => "Method",
            EthernetKind::Interface => "Interface",
            EthernetKind::Address => "Address",
            EthernetKind::Mtu => "MTU",
        }
    }
}

/// Proxy method type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EthernetMethod {
    Auto,
    Manual,
}

impl FromStr for EthernetMethod {
    type Err = PropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(EthernetMethod::Auto),
            "manual" => Ok(EthernetMethod::Manual),
            _ => Err(PropertyError::Cast(Cow::Owned(s.to_string()))),
        }
    }
}

impl From<EthernetMethod> for &'static str {
    fn from(method: EthernetMethod) -> Self {
        match method {
            EthernetMethod::Auto => "auto",
            EthernetMethod::Manual => "manual",
        }
    }
}
