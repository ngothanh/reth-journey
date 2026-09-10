//! Throughput of the lock-free MPMC ring — the numbers to capture BEFORE and
//! AFTER cache-line padding, so the false-sharing fix can be judged.
//!
//! ## The false-sharing hypothesis this bench is built to expose
//!
//! `head` and `tail` are two independent `AtomicUsize`. Logically they never
//! conflict: producers only ever write `tail`, consumers only ever write
//! `head`. But if the allocator lays them on the SAME cache line (they are
//! adjacent fields today), the hardware doesn't see "independent" — every
//! producer's `tail` write invalidates that line in the consumer's core, and
//! every consumer's `head` write invalidates it in the producer's core. The two
//! cores ping-pong one line back and forth (MESI) on every single op, paying
//! coherence traffic for a conflict that isn't real. Padding `head` and `tail`
//! onto separate lines should make that traffic vanish.
//!
//! The three scenarios, weakest-to-strongest signal:
//!   1. `uncontended` — single thread, push then pop. No second core, so no
//!      inter-core traffic at all. This is the CONTROL: padding must NOT change
//!      it (if it does, something else moved). It also gives the raw per-op cost.
//!   2. `spsc` — 1 producer, 1 consumer. Exactly one writer of `tail`, one of
//!      `head`, on two cores. This is the CLEANEST false-sharing signal: any
//!      head/tail line-bouncing shows here with nothing else in the way.
//!      Prediction: padding drops per-item cost noticeably.
//!   3. `mpmc` — 2 producers, 2 consumers. Adds producer↔producer and
//!      consumer↔consumer CAS contention on top of the head/tail traffic.
//!      Padding still helps but the CAS contention partly masks it.
//!
//! ## Method (mirrors the seq_lock / false_sharing benches)
//! - `iter_custom`: the closure transfers `iters` items end-to-end through the
//!   ring and returns the elapsed time; criterion reports elapsed/iters = per-
//!   item transfer cost. Threads are spawned once per measurement batch (not per
//!   item), so spawn cost is amortised across a large `iters`.
//! - Ring is sized well above the thread count so producers almost never block
//!   on "full" — we want to measure the handoff, not backpressure.
//! - `black_box` on the popped value so LLVM can't prove the drain dead.
//! - Threads are NOT pinned; read the SHAPE (padded vs unpadded gap), not the
//!   third significant figure. Run each side several times.
//!
//! Capture results in `notes/mpmc_bench_results.md` for both the unpadded and
//! padded revisions.

use concurrent::MpmcRing;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::{black_box, spin_loop};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const CAP: usize = 1024;

// ---------------------------------------------------------------------------
// 1. Uncontended (control): one thread, push then immediately pop.
// ---------------------------------------------------------------------------

fn uncontended(c: &mut Criterion) {
    let ring: MpmcRing<usize> = MpmcRing::with_capacity(CAP);
    c.bench_function("bench_mpmc_uncontended", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for i in 0..iters as usize {
                let _ = ring.try_push(black_box(i));
                black_box(ring.try_pop());
            }
            start.elapsed()
        });
    });
}

// ---------------------------------------------------------------------------
// 2. SPSC: 1 producer, 1 consumer — cleanest head/tail false-sharing signal.
// ---------------------------------------------------------------------------

fn spsc(c: &mut Criterion) {
    c.bench_function("bench_mpmc_spsc", |b| {
        b.iter_custom(|iters| {
            let n = iters as usize;
            let ring = Arc::new(MpmcRing::<usize>::with_capacity(CAP));

            let start = Instant::now();
            let prod = {
                let r = Arc::clone(&ring);
                thread::spawn(move || {
                    for i in 0..n {
                        while r.try_push(i).is_err() {
                            spin_loop();
                        }
                    }
                })
            };

            let mut got = 0usize;
            while got < n {
                match ring.try_pop() {
                    Some(v) => {
                        black_box(v);
                        got += 1;
                    }
                    None => spin_loop(),
                }
            }
            prod.join().unwrap();
            start.elapsed()
        });
    });
}

// ---------------------------------------------------------------------------
// 3. MPMC: 2 producers, 2 consumers.
// ---------------------------------------------------------------------------

fn mpmc(c: &mut Criterion) {
    const NPROD: usize = 2;
    const NCON: usize = 2;

    c.bench_function("bench_mpmc_2p2c", |b| {
        b.iter_custom(|iters| {
            let per_prod = (iters as usize / NPROD).max(1);
            let total = per_prod * NPROD;
            let ring = Arc::new(MpmcRing::<usize>::with_capacity(CAP));
            let consumed = Arc::new(AtomicUsize::new(0));

            let start = Instant::now();

            let producers: Vec<_> = (0..NPROD)
                .map(|_| {
                    let r = Arc::clone(&ring);
                    thread::spawn(move || {
                        for i in 0..per_prod {
                            while r.try_push(i).is_err() {
                                spin_loop();
                            }
                        }
                    })
                })
                .collect();

            let consumers: Vec<_> = (0..NCON)
                .map(|_| {
                    let r = Arc::clone(&ring);
                    let done = Arc::clone(&consumed);
                    thread::spawn(move || loop {
                        match r.try_pop() {
                            Some(v) => {
                                black_box(v);
                                if done.fetch_add(1, Ordering::Relaxed) + 1 >= total {
                                    break;
                                }
                            }
                            None => {
                                if done.load(Ordering::Relaxed) >= total {
                                    break;
                                }
                                spin_loop();
                            }
                        }
                    })
                })
                .collect();

            for p in producers {
                p.join().unwrap();
            }
            for c in consumers {
                c.join().unwrap();
            }
            let elapsed = start.elapsed();
            // Report per-item cost against the actual item count transferred.
            elapsed.mul_f64(iters as f64 / total as f64).max(Duration::ZERO)
        });
    });
}

criterion_group!(benches, uncontended, spsc, mpmc);
criterion_main!(benches);
