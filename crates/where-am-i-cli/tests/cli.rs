use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_where-am-i"))
        .args(args)
        .output()
        .expect("failed to run where-am-i")
}

#[test]
fn help_exits_zero_and_discloses_the_ip_lookup() {
    for flag in ["-h", "--help"] {
        let out = run(&[flag]);
        assert!(out.status.success());
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(
            stdout.contains("ipinfo.io"),
            "help should say who gets your IP"
        );
    }
}

#[test]
fn version_prints_name_and_version() {
    let out = run(&["--version"]);
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("where-am-i "));
}

#[test]
fn unknown_argument_exits_two_without_a_lookup() {
    let out = run(&["--bogus"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8(out.stderr).unwrap().contains("--bogus"));
}
