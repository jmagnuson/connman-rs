# connman-rs

[![crates.io](http://meritbadge.herokuapp.com/connman)](https://crates.io/crates/connman)

A [ConnMan] API library that abstracts the D-Bus layer using `dbus-tokio` and
`futures`.

The API is still under development, and may be subject to change.

[Documentation](https://docs.rs/connman)

[ConnMan]: https://01.org/connman

## Usage

Add connman-rs to your `Cargo.toml` with:

```toml
[dependencies]
connman = "0.1"
```

## Example

The following example demonstrates how to create a `Manager` and list
the available services.
 
```rust,no_run
extern crate connman;
extern crate dbus;
extern crate dbus_tokio;
extern crate tokio;

use connman::Manager;
use dbus::{BusType, Connection};
use dbus_tokio::AConnection;
use tokio::reactor::Handle;
use tokio::runtime::current_thread::Runtime;

use std::rc::Rc;

fn main() {
    let mut runtime = Runtime::new().unwrap();

    let conn = Rc::new(Connection::get_private(BusType::System).unwrap());
    let aconn = Rc::new(AConnection::new(conn.clone(), Handle::default(), &mut runtime).unwrap());
    
    let manager = Manager::new(aconn);
    
    let f = manager.get_services()
        .and_then(|services| {
            for svc in services {
                // Dump service info
                println!("Found service: {:?}", svc)
            }
            Ok(())
        });
    
    runtime.block_on(f).unwrap();
}
```

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
