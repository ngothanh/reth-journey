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

/// R6: a stub endpoint that answers the first `fail_first` requests with HTTP 429 and
/// everything after with a valid JSON-RPC result. Returns the bound address and a handle
/// to the request counter.
fn mock_rpc(fail_first: usize) -> (String, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let seen = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&seen);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let counter = Arc::clone(&counter);

            // One request per connection, then close — keep-alive would make "requests"
            // and "connections" diverge and the count meaningless.
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut content_length = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                if let Some(v) = line.strip_prefix("content-length: ") {
                    content_length = v.trim().parse().unwrap_or(0);
                } else if let Some(v) = line.strip_prefix("Content-Length: ") {
                    content_length = v.trim().parse().unwrap_or(0);
                }
                if line == "\r\n" {
                    break;
                }
            }

            let mut body = vec![0u8; content_length];
            let _ = reader.read_exact(&mut body);
            let body = String::from_utf8_lossy(&body).to_string();

            let n = counter.fetch_add(1, Ordering::SeqCst);
            let response = if n < fail_first {
                "HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_string()
            } else {
                // Echo the request id back; alloy matches responses by it.
                let id = body
                    .split("\"id\":")
                    .nth(1)
                    .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).find(|s| !s.is_empty()))
                    .unwrap_or("1")
                    .to_string();
                let payload = format!(r#"{{"jsonrpc":"2.0","id":{id},"result":"0x1"}}"#);
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    payload.len(),
                    payload
                )
            };

            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    (format!("http://{addr}"), seen)
}

#[test]
fn handles_429_with_backoff() {
    let (url, seen) = mock_rpc(2);
    let out = run(&[ADDRESS], Some(&url));

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        out.status.success(),
        "should recover after the 429s.\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stderr.contains("retrying"),
        "a retry must be visible to the operator, got: {stderr}"
    );
    assert!(stdout.contains("balance"), "got: {stdout}");
    assert!(
        seen.load(std::sync::atomic::Ordering::SeqCst) > 2,
        "expected retries beyond the two rejected requests"
    );
}

#[test]
fn gives_up_after_the_retry_cap() {
    // Every request 429s. Three attempts per call, then a clean failure — not a hang.
    let (url, _seen) = mock_rpc(usize::MAX);
    let out = run(&[ADDRESS], Some(&url));

    assert!(!out.status.success(), "must fail once retries are exhausted");
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
