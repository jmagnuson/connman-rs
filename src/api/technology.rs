use dbus::{arg, ConnPath};
use dbus_tokio::AConnection;
use futures::Future;
use qutex::Qutex;

use std::collections::HashMap;

use super::gen::technology::Technology as ITechnology;
use super::Error;

/// Futures-aware wrapper struct for connman Technology object.
#[derive(Debug)]
pub struct Technology {
    connection: Qutex<AConnection>,
    pub path: dbus::Path<'static>,
    pub args: HashMap<String, arg::Variant<Box<arg::RefArg + 'static>>>,
}

impl Technology {
    pub fn new(
        connection: Qutex<AConnection>,
        path: dbus::Path<'static>,
        args: HashMap<String, arg::Variant<Box<arg::RefArg + 'static>>>,
    ) -> Self {
        Technology {
            connection,
            path,
            args,
        }
    }

    pub fn connpath(&self, conn: Qutex<AConnection>) -> ConnPath<'static, Qutex<AConnection>> {
        let connpath = ConnPath {
            conn: conn,
            dest: "net.connman".into(),
            path: self.path.clone(),
            timeout: 5000,
        };
        connpath
    }
}

impl Technology {
    pub fn scan(&self) -> impl Future<Item=(), Error=Error> {
        let connclone = self.connection.clone();

        let connpath = self.connpath(connclone);
        ITechnology::scan(&connpath)
            .map_err(|e| e.into())
    }
}
