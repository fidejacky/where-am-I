use std::process::ExitCode;

const HELP: &str = "\
where-am-i: print your latitude and longitude

USAGE:
    where-am-i

OPTIONS:
    -h, --help       Print this help
    -V, --version    Print the version

Your position is looked up from your public IP address, which sends that
address to ipinfo.io. The result is city-level at best, and can be far off
behind a VPN or on a mobile network.

OUTPUT:
    51.51, -0.13 — London, England, GB (city-level, source: ip)
";

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        None => {}
        Some("-h" | "--help") => {
            print!("{HELP}");
            return ExitCode::SUCCESS;
        }
        Some("-V" | "--version") => {
            println!("where-am-i {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Some(other) => {
            eprintln!("where-am-i: unknown argument '{other}'");
            eprintln!("Try 'where-am-i --help'.");
            return ExitCode::from(2);
        }
    }

    match where_am_i_core::locate() {
        Ok(location) => {
            println!("{location}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("where-am-i: {err}");
            ExitCode::FAILURE
        }
    }
}
