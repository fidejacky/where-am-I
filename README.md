# where-am-I

A small, open positioning layer. It tells you your latitude and longitude, using IP geolocation by default and, in a later version, a GNSS receiver for real satellite accuracy.

It is **not** a maps app. No tiles, routing, search or place names. Just "where am I?", as a library other people can build on and a CLI on top of it.

```console
$ where-am-i
51.5074, -0.1278 (city-level, source: ip)
```

## Privacy and accuracy

IP geolocation works by sending your public IP address to a third-party service ([ipinfo.io](https://ipinfo.io)). There is no way around that with an online lookup, so `where-am-i --help` says so too.

| Source | Typical error | Status |
| --- | --- | --- |
| IP geolocation | about 5–25 km (city-level), can be hundreds of km off behind a VPN or on mobile | v1, default |
| GNSS receiver (USB dongle, or a phone streaming NMEA) | about 2–5 m in open sky; no fix indoors | planned |

The `source` in the output tells you which one you got. `accuracy` is only a number when the source reports one; IP lookups never do, so you see `city-level` instead of an invented radius.

The free ipinfo.io endpoint is capped at roughly 1,000 lookups a day without a token.

## Use it

```console
cargo run -p where-am-i-cli
```

As a library:

```rust
let here = where_am_i_core::locate()?;
println!("{here}");
println!("{}, {}", here.lat, here.lon);
```

## Design

Everything is built around one trait. A `Provider` is anything that can work out where the machine is:

```rust
pub trait Provider {
    fn name(&self) -> &'static str;
    fn locate(&self) -> Result<Location, Error>;
}
```

`Location` is `{ lat, lon, accuracy_m: Option<f64>, source }`. The CLI is a thin wrapper over `where_am_i_core::locate()`. To add a way of finding a position, implement `Provider`.

```text
crates/
  where-am-i-core/   the library: Location, Provider, the ipinfo.io provider
  where-am-i-cli/    the `where-am-i` binary
```

## Roadmap

- **v1:** IP geolocation, CLI, library. *(this)*
- **v2:** GNSS provider (gpsd, and NMEA over serial). The NMEA parser is meant to be `no_std` so it can also run on a microcontroller.
- **Later:** web app (core compiled to WASM), desktop app (Tauri), VS Code extension (spawns the CLI).
- **Out of scope:** maps, routing, search, place names, "where's my stuff".

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
