# where-am-I

A small, open positioning layer. It tells you your latitude and longitude, using IP geolocation by default and, in a later version, a GNSS receiver for real satellite accuracy.

It is **not** a maps app. No tiles, routing or search. Just "where am I?", as a library other people can build on and a CLI on top of it.

```console
$ where-am-i
51.51, -0.13 — London, England, GB (city-level, source: ip)
```

## Privacy and accuracy

IP geolocation works by sending your public IP address to a third-party service ([ipinfo.io](https://ipinfo.io)). There is no way around that with an online lookup, so `where-am-i --help` says so too.

| Source | Typical error | Status |
| --- | --- | --- |
| IP geolocation | median about 3–16 km on home broadband, with a long tail; 100+ km on mobile data, and further off behind a VPN | v1, default |
| GNSS receiver (USB dongle, or a phone streaming NMEA) | about 2–5 m in open sky; no fix indoors | planned |

The `source` in the output tells you which one you got. `accuracy` is only a number when the source reports one; IP lookups never do, so you see `city-level` instead of an invented radius.

The IP figures come from third-party studies, not from measuring this tool: a median of 3–16 km on fixed broadband versus about 180–210 km on mobile ([Ipregistry](https://ipregistry.co/blog/mobile-ip-geolocation-cgnat)), and for US addresses MaxMind reports about 80% right at state level and 66% within 50 km at city level ([MaxMind](https://support.maxmind.com/hc/en-us/articles/4407630607131)). That is roughly one lookup in three being more than 50 km off, so treat the result as "which city, probably", never "which street".

Coordinates from an IP lookup are printed to 2 decimals (about 1 km), because more digits would imply a precision the lookup doesn't have. The library still returns the values exactly as the service reported them.

The place (city, region, country as a two-letter code) comes from the same IP lookup, so it is exactly as trustworthy as the coordinates: it is where the IP address is registered, not necessarily where you are.

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

`Location` is `{ lat, lon, accuracy_m: Option<f64>, place: Option<Place>, source }`, where `Place` is an optional city, region and country. The CLI is a thin wrapper over `where_am_i_core::locate()`. To add a way of finding a position, implement `Provider`.

```text
crates/
  where-am-i-core/   the library: Location, Provider, the ipinfo.io provider
  where-am-i-cli/    the `where-am-i` binary
```

## Roadmap

- **v1:** IP geolocation, CLI, library. *(this)*
- **v2:** GNSS provider (gpsd, and NMEA over serial). The NMEA parser is meant to be `no_std` so it can also run on a microcontroller. A GNSS fix carries no place name, so `place` stays empty there until we decide how to do reverse geocoding.
- **Later:** web app (core compiled to WASM), desktop app (Tauri), VS Code extension (spawns the CLI).
- **Out of scope:** maps, routing, search, "where's my stuff".

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
