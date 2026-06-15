//! Per-op p99 latency of a slow-path `AtomicCell::<[u8; 16]>` store under
//! 4-thread contention — the one threshold criterion's batched mean/median can't
//! report directly (Threshold: p99 ≤ 2 µs).
//!
//!   cargo run --release --example atomic_cell_p99
//!
//! The measured thread times each individual `store` with `Instant` (so the
//! ~tens-of-ns clock overhead is conservative — it inflates, never hides, the
//! tail) while 3 background threads hammer the same lock. Samples are sorted and
//! p50/p99/p999/max are printed. Numbers go into `notes/atomic_cell_bench_results.md`.

use concurrent::AtomicCell;
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const SAMPLES: usize = 500_000;

fn main() {
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

    let mut latencies: Vec<Duration> = Vec::with_capacity(SAMPLES);
    for i in 0..SAMPLES {
        let t = Instant::now();
        cell.store(black_box([i as u8; 16]));
        latencies.push(t.elapsed());
    }

    stop.store(true, Ordering::Relaxed);
    for h in contenders {
        h.join().unwrap();
    }

    latencies.sort_unstable();
    let pct = |p: f64| latencies[((SAMPLES as f64 * p) as usize).min(SAMPLES - 1)];
    println!("host: aarch64-apple-darwin  threads: 4  samples: {SAMPLES}");
    println!("p50  = {:?}", pct(0.50));
    println!("p90  = {:?}", pct(0.90));
    println!("p99  = {:?}  (threshold: <= 2us)", pct(0.99));
    println!("p999 = {:?}", pct(0.999));
    println!("max  = {:?}", latencies[SAMPLES - 1]);
}
