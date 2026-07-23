//! Drill #3 numbers — per-critical-section cost of the three orderings.
//!
//!   cargo bench -p concurrent --bench dekker
//!
//! Two threads contend for the lock, each entering the critical section `iters` times.
//! Criterion divides the measured wall time by `iters`, so the reported figure is
//! **nanoseconds per critical-section enter/exit pair under two-way contention** — not
//! the uncontended cost of a single atomic operation.
//!
//! Read the numbers against the machine you are on. The plan's table quotes x86
//! (`MFENCE` ~10–20 ns against an essentially free Release/Acquire baseline); on aarch64
//! the baseline is *not* free, so the ratio between variants compresses even though the
//! absolute fence cost is higher. Record which architecture produced the numbers.

use concurrent::{DekkerAcqRel, DekkerFence, DekkerSeqCst, TwoFlagLock};
use core::cell::UnsafeCell;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

struct Shared<L> {
    lock: L,
    counter: UnsafeCell<usize>,
}

// Sound for the variants that actually provide mutual exclusion. `DekkerAcqRel` does not
// — it is benched anyway, as the baseline whose cost the fix is measured against.
unsafe impl<L: Send> Sync for Shared<L> {}

/// Two threads, `iters` critical sections each. Returns total wall time.
fn contend<L: TwoFlagLock>(iters: u64) -> Duration {
    let shared = Arc::new(Shared {
        lock: L::new(),
        counter: UnsafeCell::new(0usize),
    });
    let other = Arc::clone(&shared);

    let start = Instant::now();

    let handle = thread::spawn(move || {
        for _ in 0..iters {
            other.lock.enter(1);
            unsafe { *other.counter.get() += 1 };
            other.lock.exit(1);
        }
    });

    for _ in 0..iters {
        shared.lock.enter(0);
        unsafe { *shared.counter.get() += 1 };
        shared.lock.exit(0);
    }
    handle.join().unwrap();

    let elapsed = start.elapsed();
    black_box(unsafe { *shared.counter.get() });
    elapsed
}

/// One thread, `iters` uncontended enter/exit pairs.
///
/// The contended benchmark measures barrier cost *plus* retry-loop dynamics, and the
/// retry dynamics dominate — the confidence intervals for the two correct variants
/// overlap almost entirely. This isolates the instruction cost: the other flag is always
/// false, so `enter` is exactly store + (fence?) + load + break, with no backoff.
fn uncontended<L: TwoFlagLock>(iters: u64) -> Duration {
    let lock = L::new();
    let start = Instant::now();
    for _ in 0..iters {
        lock.enter(0);
        black_box(&lock);
        lock.exit(0);
    }
    start.elapsed()
}

fn bench(c: &mut Criterion) {
    let mut uncontended_group = c.benchmark_group("dekker_uncontended");
    uncontended_group.bench_function("acq_rel", |b| b.iter_custom(uncontended::<DekkerAcqRel>));
    uncontended_group.bench_function("fence_seqcst", |b| b.iter_custom(uncontended::<DekkerFence>));
    uncontended_group.bench_function("seqcst_store_load", |b| {
        b.iter_custom(uncontended::<DekkerSeqCst>)
    });
    uncontended_group.finish();

    let mut group = c.benchmark_group("dekker");

    // Thread spawn is ~50 µs; at ~50 ns per enter, 100k iterations puts spawn overhead
    // near 1% of each sample. Fewer iterations and the measurement is mostly spawn.
    group.sample_size(10);

    group.bench_function("acq_rel", |b| b.iter_custom(contend::<DekkerAcqRel>));
    group.bench_function("fence_seqcst", |b| b.iter_custom(contend::<DekkerFence>));
    group.bench_function("seqcst_store_load", |b| {
        b.iter_custom(contend::<DekkerSeqCst>)
    });

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
