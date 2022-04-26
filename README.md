# connman-rs

[![crates.io](http://meritbadge.herokuapp.com/connman)](https://crates.io/crates/connman)
[![Build Status](https://travis-ci.org/jmagnuson/connman-rs.svg?branch=master)](https://travis-ci.org/jmagnuson/connman-rs)

A [ConnMan] API library that abstracts the D-Bus layer using `dbus-tokio`.

The API is still under development, and may be subject to change.

[Documentation](https://docs.rs/connman)

[ConnMan]: https://01.org/connman

## Usage

Add connman-rs to your `Cargo.toml` with:

```toml
[dependencies]
connman = "0.2"
```

## Example

The following example demonstrates how to create a `Manager` and list
the available services.

```rust,no_run
use connman::Manager;
use dbus_tokio::connection;

#[tokio::main]
async fn main() {
    let (resource, conn) = connection::new_system_sync().unwrap();
    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let manager = Manager::new(conn);

    let services = manager.get_services().await.unwrap();
    for svc in services {
        // Dump service info
        println!("Found service: {:?}", svc.path())
    }
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
