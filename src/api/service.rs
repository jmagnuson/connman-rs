use dbus::{arg, ConnPath};
use dbus_tokio::AConnection;
use futures::Future;
use std::collections::HashMap;
use std::rc::Rc;

use super::gen::service::Service as IService;
use super::Error;

/// Futures-aware wrapper struct for connman Service object.
#[derive(Debug)]
pub struct Service {
    connection: Rc<AConnection>,
    pub path: dbus::Path<'static>,
    pub args: HashMap<String, arg::Variant<Box<arg::RefArg + 'static>>>,
}

impl Service {
    pub fn new(
        connection: Rc<AConnection>,
        path: dbus::Path<'static>,
        args: HashMap<String, arg::Variant<Box<arg::RefArg + 'static>>>,
    ) -> Self {
        Service {
            connection,
            path,
            args,
        }
    }

    pub fn connpath(&self, conn: Rc<AConnection>) -> ConnPath<'static, Rc<AConnection>> {
        let connpath = ConnPath {
            conn: conn,
            dest: "net.connman".into(),
            path: self.path.clone(),
            timeout: 5000,
        };
        connpath
    }
}

impl Service {
    pub fn connect(&self) -> impl Future<Item=(), Error=Error> {
        let connpath = self.connpath(self.connection.clone());
        IService::connect(&connpath).map_err(|e| e.into())
    }

    pub fn disconnect(&self) -> impl Future<Item=(), Error=Error> {
        let connpath = self.connpath(self.connection.clone());
        IService::disconnect(&connpath).map_err(|e| e.into())
    }

    pub fn remove(&self) -> impl Future<Item=(), Error=Error> {
        let connpath = self.connpath(self.connection.clone());
        IService::remove(&connpath).map_err(|e| e.into())
    }

    pub fn move_before(&self, service: &Service) -> impl Future<Item=(), Error=Error> {
        let connpath = self.connpath(self.connection.clone());
        IService::move_before(&connpath, service.path.clone()).map_err(|e| e.into())
    }

    pub fn move_after(&self, service: &Service) -> impl Future<Item=(), Error=Error> {
        let connpath = self.connpath(self.connection.clone());
        IService::move_after(&connpath, service.path.clone()).map_err(|e| e.into())
    }
}
