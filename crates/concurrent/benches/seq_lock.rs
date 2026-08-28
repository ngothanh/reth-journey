//! Why `SeqLock` exists, measured against `RwLock<T>` on the same payload.
//!
//! Payload is `[u64; 4]` (32 bytes): larger than a machine word — so `SeqLock`
//! is justified over `AtomicCell` (which falls back to a spinlock past 8 bytes)
//! — and the honest alternative for a read-mostly multi-field snapshot is a
//! `RwLock`. The three questions:
//!
//!   1. Uncontended read latency — SeqLock's read path has zero shared writes;
//!      RwLock's `read()` still does a `fetch_add`/`fetch_sub` on the reader
//!      count. How much does that cost with nobody contending?
//!   2. Read scaling — the load-bearing claim. N reader threads that don't
//!      conflict *logically* still serialize *physically* on RwLock, because
//!      every `read()` writes the shared reader counter, bouncing that one cache
//!      line between cores (MESI). SeqLock readers only ever LOAD, so the payload
//!      line stays Shared in every core's L1 at once. Prediction: SeqLock
//!      reader latency is flat as readers are added; RwLock's climbs.
//!   3. Read under a hot writer — RwLock's reader BLOCKS while the writer holds
//!      the lock; SeqLock's reader never blocks (it retries, but doesn't wait on
//!      a lock word). Prediction: a continuous writer barely moves SeqLock read
//!      latency but spikes RwLock's.
//!
//! ## Method (mirrors atomic_cell / false_sharing benches)
//! - `iter_custom`: criterion's ~10 ns clock+dispatch overhead would swamp a
//!   ~ns read, so each closure call times one large batch and reports
//!   elapsed/iters = per-op latency.
//! - `black_box` on every loaded value so LLVM can't fold the read to a constant
//!   or prove the loop dead.
//! - Background contenders (extra readers / a hot writer) are spawned ONCE
//!   outside the timed region and kept hot with a `stop` flag; only the measured
//!   thread's ops are inside the clock.
//! - Threads are NOT pinned (no `core_affinity` dep); on a quiet machine the
//!   scaling gap is still unambiguous. Run-to-run noise is real — read the shape
//!   (flat vs climbing), not the third significant figure.
//!
//! Committed numbers: `aarch64-apple-darwin` (Apple M2), see
//! `notes/seq_lock_bench_results.md`.

use concurrent::{RwLock, SeqLock};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

type Payload = [u64; 4];

const INIT: Payload = [0xA5A5_A5A5_A5A5_A5A5; 4];

// ---------------------------------------------------------------------------
// 1. Uncontended read latency
// ---------------------------------------------------------------------------

fn seqlock_read_uncontended(c: &mut Criterion) {
    let lock = SeqLock::new(black_box(INIT));
    c.bench_function("bench_seqlock_read_uncontended", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(lock.load());
            }
            start.elapsed()
        });
    });
}

fn rwlock_read_uncontended(c: &mut Criterion) {
    let lock = RwLock::new(black_box(INIT));
    c.bench_function("bench_rwlock_read_uncontended", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(*lock.read());
            }
            start.elapsed()
        });
    });
}

// ---------------------------------------------------------------------------
// 2. Read scaling: measured reader latency with `bg` extra readers hammering.
//    SeqLock should stay flat; RwLock should climb with reader count.
// ---------------------------------------------------------------------------

fn seqlock_read_with_readers(c: &mut Criterion) {
    for bg in [1usize, 3, 7] {
        let lock = Arc::new(SeqLock::new(INIT));
        let stop = Arc::new(AtomicBool::new(false));
        let contenders: Vec<_> = (0..bg)
            .map(|_| {
                let lock = Arc::clone(&lock);
                let stop = Arc::clone(&stop);
                thread::spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        black_box(lock.load());
                    }
                })
            })
            .collect();

        c.bench_function(&format!("bench_seqlock_read_{}readers", bg + 1), |b| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    black_box(lock.load());
                }
                start.elapsed()
            });
        });

        stop.store(true, Ordering::Relaxed);
        for h in contenders {
            h.join().unwrap();
        }
    }
}

fn rwlock_read_with_readers(c: &mut Criterion) {
    for bg in [1usize, 3, 7] {
        let lock = Arc::new(RwLock::new(INIT));
        let stop = Arc::new(AtomicBool::new(false));
        let contenders: Vec<_> = (0..bg)
            .map(|_| {
                let lock = Arc::clone(&lock);
                let stop = Arc::clone(&stop);
                thread::spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        black_box(*lock.read());
                    }
                })
            })
            .collect();

        c.bench_function(&format!("bench_rwlock_read_{}readers", bg + 1), |b| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    black_box(*lock.read());
                }
                start.elapsed()
            });
        });

        stop.store(true, Ordering::Relaxed);
        for h in contenders {
            h.join().unwrap();
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Read under a hot writer. RwLock reader blocks on the writer; SeqLock
//    reader retries but never waits on a lock word.
// ---------------------------------------------------------------------------

fn seqlock_read_hot_writer(c: &mut Criterion) {
    let lock = Arc::new(SeqLock::new(INIT));
    let stop = Arc::new(AtomicBool::new(false));
    let writer = {
        let lock = Arc::clone(&lock);
        let stop = Arc::clone(&stop);
        thread::spawn(move || {
            let mut n = 0u64;
            while !stop.load(Ordering::Relaxed) {
                n = n.wrapping_add(1);
                lock.store(black_box([n; 4]));
            }
        })
    };

    c.bench_function("bench_seqlock_read_hot_writer", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(lock.load());
            }
            start.elapsed()
        });
    });

    stop.store(true, Ordering::Relaxed);
    writer.join().unwrap();
}

fn rwlock_read_hot_writer(c: &mut Criterion) {
    let lock = Arc::new(RwLock::new(INIT));
    let stop = Arc::new(AtomicBool::new(false));
    let writer = {
        let lock = Arc::clone(&lock);
        let stop = Arc::clone(&stop);
        thread::spawn(move || {
            let mut n = 0u64;
            while !stop.load(Ordering::Relaxed) {
                n = n.wrapping_add(1);
                *lock.write() = black_box([n; 4]);
            }
        })
    };

    c.bench_function("bench_rwlock_read_hot_writer", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(*lock.read());
            }
            start.elapsed()
        });
    });

    stop.store(true, Ordering::Relaxed);
    writer.join().unwrap();
}

// ---------------------------------------------------------------------------
// 4. Uncontended write latency, for context (SeqLock trades a cheaper read for
//    a writer that always wins).
// ---------------------------------------------------------------------------

fn seqlock_store_uncontended(c: &mut Criterion) {
    let lock = SeqLock::new(INIT);
    c.bench_function("bench_seqlock_store_uncontended", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for i in 0..iters {
                lock.store(black_box([i; 4]));
            }
            start.elapsed()
        });
    });
}

fn rwlock_write_uncontended(c: &mut Criterion) {
    let lock = RwLock::new(INIT);
    c.bench_function("bench_rwlock_write_uncontended", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for i in 0..iters {
                *lock.write() = black_box([i; 4]);
            }
            start.elapsed()
        });
    });
}

criterion_group!(
    benches,
    seqlock_read_uncontended,
    rwlock_read_uncontended,
    seqlock_read_with_readers,
    rwlock_read_with_readers,
    seqlock_read_hot_writer,
    rwlock_read_hot_writer,
    seqlock_store_uncontended,
    rwlock_write_uncontended,
);
criterion_main!(benches);
