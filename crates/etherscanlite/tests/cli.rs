//! Binary-level behaviour. `etherscanlite` is a bin crate, so these drive the real
//! executable via `CARGO_BIN_EXE_*` rather than importing its modules.
//!
//! Run: `cargo test -p etherscanlite --test cli`

use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_etherscanlite");
const ADDRESS: &str = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";

/// Runs the binary with `ETH_RPC_URL` explicitly removed unless `rpc_url` says otherwise.
fn run(args: &[&str], rpc_url: Option<&str>) -> Output {
    let mut cmd = Command::new(BIN);
    cmd.args(args);
    match rpc_url {
        Some(url) => cmd.env("ETH_RPC_URL", url),
        None => cmd.env_remove("ETH_RPC_URL"),
    };
    cmd.output().expect("binary runs")
}

#[test]
fn missing_env_var_fails_fast() {
    let out = run(&[ADDRESS], None);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(!out.status.success(), "must exit non-zero");
    assert!(
        stderr.contains("ETH_RPC_URL"),
        "error must name the variable, got: {stderr}"
    );
}

#[test]
fn missing_env_var_is_reported_before_any_network_work() {
    // D4: the failure must be a config error, not a connection error surfacing later.
    // A bogus-but-parseable URL would produce a *different* message; absence must not.
    let out = run(&[ADDRESS], None);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        !stderr.contains("connect") && !stderr.contains("dns") && !stderr.contains("timed out"),
        "missing config leaked as a network error: {stderr}"
    );
}

#[test]
fn address_checksum_validation() {
    // A valid URL that will never be dialled, because argument parsing fails first.
    let url = Some("http://127.0.0.1:1");

    let mut chars: Vec<char> = ADDRESS.chars().collect();
    chars[3] = chars[3].to_ascii_uppercase();
    let tampered: String = chars.into_iter().collect();

    let out = run(&[&tampered], url);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(!out.status.success());
    assert!(
        stderr.contains("checksum"),
        "wrong-checksum address must say so, got: {stderr}"
    );
}

#[test]
fn malformed_address_is_rejected_without_connecting() {
    let out = run(&["0xnothex"], Some("http://127.0.0.1:1"));
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(!out.status.success());
    assert!(
        !stderr.contains("connect"),
        "should fail on parsing, not dialling: {stderr}"
    );
}

#[test]
fn help_lists_the_tx_flag() {
    let out = run(&["--help"], None);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(out.status.success());
    assert!(stdout.contains("--tx"), "got: {stdout}");
}

/// R2 smoke test. Ignored by default so CI does not burn the RPC quota; run with
/// `ETH_RPC_URL=... cargo test -p etherscanlite --test cli -- --ignored`.
#[test]
#[ignore = "requires a live ETH_RPC_URL"]
fn smoke_fetches_balance() {
    let url = std::env::var("ETH_RPC_URL").expect("ETH_RPC_URL must be set for this test");
    // The zero address: always exists, balance is stable enough to assert on shape.
    let out = run(&["0x0000000000000000000000000000000000000000"], Some(&url));
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("balance"), "got: {stdout}");
    assert!(stdout.contains("nonce"), "got: {stdout}");
}
