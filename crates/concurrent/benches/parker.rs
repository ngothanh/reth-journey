//! `parker_unpark_fast_path_no_syscall` — the fast path must be a few atomic
//! ops, never a syscall.
//!
//! This Criterion bench measures per-op latency of `unpark()` on a Parker that
//! has no parked waiter. Target: p50 ≤ 5 ns (a single `swap` + branch; no
//! `wake`/futex syscall).
//!
//! Latency is checked here; the *zero-syscall* half of the claim is verified
//! out-of-band with a syscall tracer, e.g.:
//!
//!   Linux:  strace -f -c -e trace=futex \
//!             target/release/examples/parker_one_syscall fast
//!   macOS:  sudo dtruss -c target/release/examples/parker_one_syscall fast
//!
//! Expect zero `futex`/`__ulock_wake` entries for the fast-path run.

use concurrent::Parker;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn unpark_fast_path(c: &mut Criterion) {
    let parker = Parker::new();
    let unparker = parker.unparker();
    // No `park()` is ever called, so every `unpark` takes the fast path:
    // `swap(NOTIFIED)` observes EMPTY/NOTIFIED and returns without waking.
    c.bench_function("parker_unpark_fast_path_no_syscall", |b| {
        b.iter(|| black_box(&unparker).unpark());
    });
}

criterion_group!(benches, unpark_fast_path);
criterion_main!(benches);
