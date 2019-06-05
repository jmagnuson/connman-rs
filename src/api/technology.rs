use dbus::{arg, ConnPath};
use dbus_tokio::AConnection;
use futures::Future;
use std::rc::Rc;

use std::collections::HashMap;

use super::gen::technology::Technology as ITechnology;
use super::{Error, RefArgMap};

#[cfg(feature = "introspection")]
use xml::reader::EventReader;

/// Futures-aware wrapper struct for connman Technology object.
#[derive(Debug)]
pub struct Technology {
    connection: Rc<AConnection>,
    pub path: dbus::Path<'static>,
    pub args: RefArgMap,
}

impl Technology {
    pub fn new(
        connection: Rc<AConnection>,
        path: dbus::Path<'static>,
        args: RefArgMap,
    ) -> Self {
        Technology {
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

impl Technology {
    #[cfg(feature = "introspection")]
    pub fn introspect(&self) -> impl Future<Item=EventReader<std::io::Cursor<Vec<u8>>>, Error=Error> {
        use crate::api::gen::technology::OrgFreedesktopDBusIntrospectable as Introspectable;

        Introspectable::introspect(&self.connpath(self.connection.clone()))
            .map_err(|e| e.into())
            .map(|s| {
                let rdr = std::io::Cursor::new(s.into_bytes());
                EventReader::new(rdr)
            })
    }

    pub fn scan(&self) -> impl Future<Item=(), Error=Error> {
        let connclone = self.connection.clone();

        let connpath = self.connpath(connclone);
        ITechnology::scan(&connpath)
            .map_err(|e| e.into())
    }
}
