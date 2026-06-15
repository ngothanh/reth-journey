//! False-sharing evidence for `CachePadded` (Monday's Back-of-envelope claim:
//! "30-70% MPMC throughput regression on contested adjacent counters").
//!
//! Disruptor mirror: two threads ping-pong a pair of adjacent `AtomicU64`
//! counters via `fetch_add(1, Relaxed)`. In `bare_pingpong` the two counters sit
//! 8 bytes apart and share a cache line, so every write bounces the line between
//! cores (MESI ping-pong). In `padded_pingpong` each counter is `CachePadded`
//! (align 128 on aarch64), so the writes land on separate lines and the bounce
//! disappears. The throughput gap is the measured cliff.
//!
//! ## Method (per design D1–D6)
//! - D3 `Relaxed` ordering: we measure cache-coherence cost, not fence cost.
//! - D6 elision guard: `black_box` the add operand AND assert each counter's
//!   end-state — otherwise LLVM proves the counters dead and times an empty loop.
//! - D5 layout guard: `verify_layout_difference` asserts the wrapper actually
//!   moved `b` past a cache line; without it a refactor could measure nothing.
//! - D4 thread placement: threads are NOT pinned (no `core_affinity` dep); we
//!   accept some run-to-run noise. On a quiet machine the cliff is still clear.
//!
//! `iter_custom` times the join of two worker threads each doing `iters`
//! `fetch_add`s; criterion picks `iters` ≫ 1M for these sub-100ns ops, so the
//! per-closure thread-spawn/barrier cost (excluded from the timed region — the
//! clock starts after the start barrier) is amortized to nothing.
//!
//! Results: `notes/cache_padded_bench_results.md`.

use concurrent::CachePadded;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::mem::offset_of;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

#[repr(C)]
#[derive(Default)]
struct BarePair {
    a: AtomicU64,
    b: AtomicU64,
}

#[repr(C)]
#[derive(Default)]
struct PaddedPair {
    a: CachePadded<AtomicU64>,
    b: CachePadded<AtomicU64>,
}

/// Two adjacent atomic counters, one per contending thread.
trait Pair: Default + Send + Sync + 'static {
    fn a(&self) -> &AtomicU64;
    fn b(&self) -> &AtomicU64;
}

impl Pair for BarePair {
    fn a(&self) -> &AtomicU64 {
        &self.a
    }
    fn b(&self) -> &AtomicU64 {
        &self.b
    }
}

impl Pair for PaddedPair {
    fn a(&self) -> &AtomicU64 {
        &self.a // deref coercion: &CachePadded<AtomicU64> -> &AtomicU64
    }
    fn b(&self) -> &AtomicU64 {
        &self.b
    }
}

/// Pitfall #1 (Phase 3): the two arms differ by one type wrapper, so the bench
/// is meaningless unless that wrapper actually changed the layout. Bare fields
/// must be adjacent (< one line); padded fields must straddle a 128 B boundary.
fn verify_layout_difference() {
    let bare_b = offset_of!(BarePair, b);
    let padded_b = offset_of!(PaddedPair, b);
    assert!(
        bare_b < 128,
        "BarePair.b at offset {bare_b}; fields not adjacent — bench measures nothing"
    );
    assert!(
        padded_b >= 128,
        "PaddedPair.b at offset {padded_b}; padding didn't take — bench measures nothing"
    );
}

/// Run the 2-thread ping-pong for `iters` ops/thread and return the wall-clock
/// of the contended region (clock starts after the start barrier, so thread
/// spawn is excluded).
fn measure<P: Pair>(iters: u64) -> Duration {
    let pair = Arc::new(P::default());
    let barrier = Arc::new(Barrier::new(3)); // 2 workers + main

    let handles: Vec<_> = (0..2u8)
        .map(|tid| {
            let pair = Arc::clone(&pair);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                // Each thread owns exactly one counter — they contend on the
                // shared cache line, never on the same atomic (D1).
                let counter = if tid == 0 { pair.a() } else { pair.b() };
                barrier.wait();
                for _ in 0..iters {
                    counter.fetch_add(black_box(1), Ordering::Relaxed);
                }
            })
        })
        .collect();

    barrier.wait();
    let start = Instant::now();
    for h in handles {
        h.join().unwrap();
    }
    let elapsed = start.elapsed();

    // Pitfall #2 (Phase 3): each counter is hit by exactly one thread `iters`
    // times. If LLVM elided any write, these fail — proving the loop was real.
    assert_eq!(pair.a().load(Ordering::Relaxed), iters, "counter a: writes elided?");
    assert_eq!(pair.b().load(Ordering::Relaxed), iters, "counter b: writes elided?");
    black_box(pair.a().load(Ordering::Relaxed));

    elapsed
}

fn bench_bare_pingpong(c: &mut Criterion) {
    verify_layout_difference();
    c.bench_function("bench_bare_pingpong", |b| {
        b.iter_custom(measure::<BarePair>);
    });
}

fn bench_padded_pingpong(c: &mut Criterion) {
    verify_layout_difference();
    c.bench_function("bench_padded_pingpong", |b| {
        b.iter_custom(measure::<PaddedPair>);
    });
}

criterion_group!(benches, bench_bare_pingpong, bench_padded_pingpong);
criterion_main!(benches);
