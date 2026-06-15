//! Per-op latency of `AtomicCell`: fast path (`u64`, lock-free `AtomicU64`) vs
//! slow path (`[u8; 16]`, spinlock), plus a 4-thread contended slow-path store.
//!
//! D1 — measured with `iter_custom`: criterion's ~10 ns clock+dispatch overhead
//! would swamp a sub-ns fast-path op, so each closure call times one large batch
//! (`iters` is in the millions after warm-up, ≫ 10k) and the elapsed/`iters`
//! criterion reports is the per-op latency.
//!
//! D2 — `black_box` keeps stored values opaque and forces loads to be observed,
//! so LLVM can't prove a store dead or fold a load to a constant.
//!
//! Committed numbers are for `aarch64-apple-darwin`; see
//! `notes/atomic_cell_bench_results.md`. Criterion prints the mean/median (p50);
//! the 4-thread p99 is measured separately in `examples/atomic_cell_p99.rs`.

use concurrent::AtomicCell;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn fast_path_store(c: &mut Criterion) {
    // `u64` is 8 bytes / align 8 → lock-free `AtomicU64` fast path.
    let cell = AtomicCell::new(0u64);
    c.bench_function("bench_atomic_cell_fast_path_store", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for i in 0..iters {
                cell.store(black_box(i));
            }
            start.elapsed()
        });
    });
}

fn fast_path_load(c: &mut Criterion) {
    let cell = AtomicCell::new(black_box(0xDEAD_BEEFu64));
    c.bench_function("bench_atomic_cell_fast_path_load", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(cell.load());
            }
            start.elapsed()
        });
    });
}

fn slow_path_store(c: &mut Criterion) {
    // `[u8; 16]` is 16 bytes → never hits the 8-byte fast path; rides the
    // `AtomicBool` spinlock + `UnsafeCell` slow path. Uncontended here.
    let cell = AtomicCell::new([0u8; 16]);
    c.bench_function("bench_atomic_cell_slow_path_store", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for i in 0..iters {
                cell.store(black_box([i as u8; 16]));
            }
            start.elapsed()
        });
    });
}

fn slow_path_load(c: &mut Criterion) {
    let cell = AtomicCell::new(black_box([0xABu8; 16]));
    c.bench_function("bench_atomic_cell_slow_path_load", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(cell.load());
            }
            start.elapsed()
        });
    });
}

fn slow_path_store_4thread(c: &mut Criterion) {
    // D3 — 4 threads contend on the same spinlock: 3 background contenders keep
    // the lock hot while criterion times the 4th (measured) thread's stores.
    let cell = Arc::new(AtomicCell::new([0u8; 16]));
    let stop = Arc::new(AtomicBool::new(false));

    let contenders: Vec<_> = (0..3)
        .map(|_| {
            let cell = Arc::clone(&cell);
            let stop = Arc::clone(&stop);
            thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    cell.store(black_box([0xCD; 16]));
                }
            })
        })
        .collect();

    c.bench_function("bench_atomic_cell_slow_path_store_4thread", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for i in 0..iters {
                cell.store(black_box([i as u8; 16]));
            }
            start.elapsed()
        });
    });

    stop.store(true, Ordering::Relaxed);
    for h in contenders {
        h.join().unwrap();
    }
}

criterion_group!(
    benches,
    fast_path_store,
    fast_path_load,
    slow_path_store,
    slow_path_load,
    slow_path_store_4thread
);
criterion_main!(benches);
