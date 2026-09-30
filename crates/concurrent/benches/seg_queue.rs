//! Throughput of the unbounded lock-free `SegQueue` — the numbers to capture
//! BEFORE and AFTER cache-line padding of `head`/`tail`, plus the lock-based
//! baseline it has to beat to justify existing at all.
//!
//! ## The two questions this bench answers
//!
//! **1. Is lock-free worth it?** `Mutex<VecDeque>` is the honest baseline: it is
//! also unbounded, also MPMC, also FIFO, and it is ~15 lines. If `SegQueue` does
//! not beat it under contention, all the CAS reasoning bought nothing. Note the
//! baseline is not a straw man — an uncontended mutex is only a few ns, so it
//! should WIN the single-threaded control. The lock-free structure earns its keep
//! only when several cores push at once.
//!
//! **2. Does `head`/`tail` false sharing cost measurable time?** They are two
//! independent `AtomicPtr`: producers only ever write `tail`, consumers only ever
//! write `head`. Logically disjoint — but they are adjacent fields, so the
//! allocator lays them 8 B apart, on the SAME 64 B cache line. (Measured: the
//! two field addresses differ by exactly 8.) The hardware does not see
//! "independent": every producer's `tail` store invalidates that line in the
//! consumer's core, and every consumer's `head` store invalidates it in the
//! producer's, so the two cores ping-pong one line (MESI) on ops that never
//! actually conflict. Wrapping both in `CachePadded` should make that vanish.
//!
//! ## The three scenarios, weakest-to-strongest false-sharing signal
//!
//! 1. `uncontended` — one thread, push then pop. No second core, so no
//!    inter-core traffic. This is the CONTROL: padding must NOT change it. If it
//!    does, something else moved and the other numbers are suspect. Also gives
//!    the raw per-op cost.
//! 2. `spsc` — 1 producer, 1 consumer. Exactly one writer of `tail` and one of
//!    `head`, on two cores, with nothing else in the way. CLEANEST signal;
//!    prediction: padding visibly drops per-item cost.
//! 3. `mpmc_2p2c` — 2 producers, 2 consumers. Adds producer↔producer contention
//!    on `claimed` and consumer↔consumer contention on `consumed` on top of the
//!    head/tail traffic, which partly masks the padding win.
//!
//! ## Method
//! - `iter_custom`: the closure transfers `iters` items end-to-end and returns
//!   elapsed, so criterion reports per-item transfer cost. Threads are spawned
//!   once per measurement batch, not per item, so spawn cost amortises.
//! - `black_box` on the popped value so LLVM cannot prove the drain dead.
//! - Threads are NOT pinned. Read the SHAPE (padded vs unpadded gap, lock-free vs
//!   mutex gap), not the third significant figure. Run each side several times.
//!
//! ## Note on the sample budget (stage R0 leaks!)
//! `SegQueue` is at stage R0: retired segments are never freed. One segment
//! (~536 B) is retired every 32 items, so ~17 MB leaks per million items. The
//! group below therefore runs a deliberately short measurement (10 samples,
//! 300 ms) — roughly 100 MB per SegQueue scenario. Do NOT raise
//! `measurement_time` here until reclamation lands, or the allocator pressure
//! will both distort the numbers and exhaust memory.
//!
//! Capture results in `notes/seg_queue_bench_results.md` for both the unpadded
//! and padded revisions.

use concurrent::SegQueue;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use std::collections::VecDeque;
use std::hint::{black_box, spin_loop};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// The honest lock-based baseline: unbounded, MPMC, FIFO, ~15 lines.
struct MutexQueue<T>(Mutex<VecDeque<T>>);

impl<T> MutexQueue<T> {
    fn new() -> Self {
        Self(Mutex::new(VecDeque::new()))
    }

    fn push(&self, value: T) {
        self.0.lock().unwrap().push_back(value);
    }

    fn pop(&self) -> Option<T> {
        self.0.lock().unwrap().pop_front()
    }
}

/// Keep the R0 leak bounded — see the module note.
fn configure<M: criterion::measurement::Measurement>(g: &mut BenchmarkGroup<'_, M>) {
    g.sample_size(10);
    g.warm_up_time(Duration::from_millis(100));
    g.measurement_time(Duration::from_millis(300));
}

// ---------------------------------------------------------------------------
// 1. Uncontended (control): one thread, push then immediately pop.
//    Padding must NOT move this.
// ---------------------------------------------------------------------------

fn uncontended(c: &mut Criterion) {
    let mut g = c.benchmark_group("seg_queue_uncontended");
    configure(&mut g);

    g.bench_function("seg_queue", |b| {
        let q: SegQueue<usize> = SegQueue::new();
        b.iter_custom(|iters| {
            let start = Instant::now();
            for i in 0..iters as usize {
                q.push(black_box(i));
                black_box(q.pop());
            }
            start.elapsed()
        });
    });

    g.bench_function("mutex_vecdeque", |b| {
        let q: MutexQueue<usize> = MutexQueue::new();
        b.iter_custom(|iters| {
            let start = Instant::now();
            for i in 0..iters as usize {
                q.push(black_box(i));
                black_box(q.pop());
            }
            start.elapsed()
        });
    });

    g.finish();
}

// ---------------------------------------------------------------------------
// 2. SPSC: 1 producer, 1 consumer — cleanest head/tail false-sharing signal.
// ---------------------------------------------------------------------------

fn spsc(c: &mut Criterion) {
    let mut g = c.benchmark_group("seg_queue_spsc");
    configure(&mut g);

    g.bench_function("seg_queue", |b| {
        b.iter_custom(|iters| {
            let n = iters as usize;
            let q = Arc::new(SegQueue::<usize>::new());

            let start = Instant::now();
            let prod = {
                let q = Arc::clone(&q);
                thread::spawn(move || {
                    for i in 0..n {
                        q.push(i);
                    }
                })
            };

            let mut got = 0usize;
            while got < n {
                match q.pop() {
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

    g.bench_function("mutex_vecdeque", |b| {
        b.iter_custom(|iters| {
            let n = iters as usize;
            let q = Arc::new(MutexQueue::<usize>::new());

            let start = Instant::now();
            let prod = {
                let q = Arc::clone(&q);
                thread::spawn(move || {
                    for i in 0..n {
                        q.push(i);
                    }
                })
            };

            let mut got = 0usize;
            while got < n {
                match q.pop() {
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

    g.finish();
}

// ---------------------------------------------------------------------------
// 3. MPMC: 2 producers, 2 consumers.
// ---------------------------------------------------------------------------

const NPROD: usize = 2;
const NCON: usize = 2;

fn mpmc(c: &mut Criterion) {
    let mut g = c.benchmark_group("seg_queue_mpmc_2p2c");
    configure(&mut g);

    g.bench_function("seg_queue", |b| {
        b.iter_custom(|iters| {
            let per_prod = (iters as usize / NPROD).max(1);
            let total = per_prod * NPROD;
            let q = Arc::new(SegQueue::<usize>::new());
            let consumed = Arc::new(AtomicUsize::new(0));

            let start = Instant::now();

            let producers: Vec<_> = (0..NPROD)
                .map(|_| {
                    let q = Arc::clone(&q);
                    thread::spawn(move || {
                        for i in 0..per_prod {
                            q.push(i);
                        }
                    })
                })
                .collect();

            let consumers: Vec<_> = (0..NCON)
                .map(|_| {
                    let q = Arc::clone(&q);
                    let done = Arc::clone(&consumed);
                    thread::spawn(move || loop {
                        match q.pop() {
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
            elapsed.mul_f64(iters as f64 / total as f64).max(Duration::ZERO)
        });
    });

    g.bench_function("mutex_vecdeque", |b| {
        b.iter_custom(|iters| {
            let per_prod = (iters as usize / NPROD).max(1);
            let total = per_prod * NPROD;
            let q = Arc::new(MutexQueue::<usize>::new());
            let consumed = Arc::new(AtomicUsize::new(0));

            let start = Instant::now();

            let producers: Vec<_> = (0..NPROD)
                .map(|_| {
                    let q = Arc::clone(&q);
                    thread::spawn(move || {
                        for i in 0..per_prod {
                            q.push(i);
                        }
                    })
                })
                .collect();

            let consumers: Vec<_> = (0..NCON)
                .map(|_| {
                    let q = Arc::clone(&q);
                    let done = Arc::clone(&consumed);
                    thread::spawn(move || loop {
                        match q.pop() {
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
            elapsed.mul_f64(iters as f64 / total as f64).max(Duration::ZERO)
        });
    });

    g.finish();
}

criterion_group!(benches, uncontended, spsc, mpmc);
criterion_main!(benches);
