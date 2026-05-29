//! `parker_park_entry_one_syscall` — syscall-count harness for the Parker.
//!
//! macOS has no in-process, portable way to assert "exactly one futex_wait", so
//! the syscall claims are verified by running this example under a tracer and
//! counting wake/wait entries. The program does the bare minimum work so the
//! trace is dominated by the one operation under test.
//!
//! Build once:
//!   cargo build --release -p concurrent --example parker_one_syscall
//!
//! ── slow path: exactly one wait syscall ───────────────────────────────────
//! A single `park()` blocks (one futex_wait), then a sibling thread unparks it
//! after a delay (one wake). Run:
//!   Linux:  strace -f -c -e trace=futex target/release/examples/parker_one_syscall park
//!   macOS:  sudo dtruss -c target/release/examples/parker_one_syscall park
//! Expect exactly one futex_wait (Linux) / one __ulock_wait (macOS).
//!
//! ── fast path: zero wait/wake syscalls ─────────────────────────────────────
//! 1,000,000 `unpark()` calls on a never-parked Parker. Run the same tracers
//! with the `fast` argument; expect ZERO futex/__ulock_* entries.

use concurrent::Parker;
use std::thread;
use std::time::Duration;

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("fast") => fast_path(),
        // Default to the slow path so a bare `… parker_one_syscall` is the
        // one-syscall case the test name describes.
        _ => slow_path(),
    }
}

/// One `park()` that actually blocks and is woken once → one wait syscall.
fn slow_path() {
    let parker = Parker::new();
    let unparker = parker.unparker();
    let waker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        unparker.unpark();
    });
    parker.park();
    waker.join().unwrap();
    eprintln!("slow_path: parked once, woken once");
}

/// 1M `unpark()` on a never-parked Parker → zero wake syscalls.
fn fast_path() {
    let parker = Parker::new();
    let unparker = parker.unparker();
    for _ in 0..1_000_000 {
        unparker.unpark();
    }
    eprintln!("fast_path: 1M unparks, no waiter");
}
