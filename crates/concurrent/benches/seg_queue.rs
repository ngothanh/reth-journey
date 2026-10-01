//! Throughput of the unbounded lock-free `SegQueue`, against a lock-based
//! baseline, across three consumer drain policies.
//!
//! ## The three questions this bench answers
//!
//! **1. Is lock-free worth it?** `Mutex<VecDeque>` is the honest baseline: also
//! unbounded, also MPMC, also FIFO, ~15 lines. If `SegQueue` does not beat it,
//! all the CAS reasoning bought nothing.
//!
//! **2. Where does false sharing actually cost time?** Two candidate pairs of
//! "logically disjoint but physically adjacent" atomics, and the measurements show
//! the obvious one is the wrong one:
//!
//! - `SegQueue::{head, tail}` — 8 B apart, same line, but only written when a
//!   segment boundary is crossed, once per `SEG_LEN` (32) ops. COLD. Padding them
//!   measured free and worth ~nothing (10.7 ns of false sharing ÷ 32 = 0.33 ns/op,
//!   below the noise floor — a calculation that takes 30 seconds and would have
//!   skipped the experiment entirely).
//! - `Segment::{claimed, consumed}` — also 8 B apart, but written on EVERY push
//!   and pop. HOT. Padding it took spsc 45.8 → 30.2 ns (−34%).
//!
//! "Adjacent + logically disjoint" is only half the test; the other half is write
//! frequency. Padding is not free either: separating the counters made the
//! single-threaded path touch two cache lines per push+pop instead of one and
//! allocate a larger `Segment` every 32 items, moving the uncontended control
//! 8.7 → 15.2 ns (+74%).
//!
//! **3. How much does the CALLER's drain policy matter?** A great deal — more than
//! anything inside the queue. `pop` returns `Option`, so the empty-wait policy is
//! deliberately the caller's, and that policy turned out to dominate the whole
//! benchmark (spsc −56% from backing off instead of hot-spinning). So it is swept
//! as an explicit variable rather than left as a hidden constant: the unit of
//! measurement is the **pair** (queue, drain policy). There is no such thing as
//! "the throughput of a queue".
//!
//! | policy | on `pop() == None` | CPU while waiting | item latency on arrival |
//! |---|---|---|---|
//! | `Spin` | `spin_loop()` (≈12.2 ns/ISB here) | burns a core | ~ns — lowest |
//! | `Backoff` | `Backoff::snooze()` — bursts, then `yield_now` | adaptive | ns→µs |
//! | `Yield` | `thread::yield_now()` (≈4.6 µs here) | gives the core back | ~µs — highest |
//!
//! A fourth policy — **park, woken by the producer** — is deliberately absent: it
//! needs a notification layer (`Parker`/`WaitList` → `channel`), which is a
//! separate node. It would not be strictly better either: parking adds µs-scale
//! wake latency to an item that arrives while the consumer sleeps, ~1000× the
//! ns-scale catch of a dedicated spinner. Which is why latency-critical consumers
//! get their own core to spin on rather than a clever notification.
//!
//! ## Method
//! - `iter_custom`: the closure transfers `iters` items end-to-end and returns
//!   elapsed, so criterion reports per-item transfer cost. Threads are spawned
//!   once per measurement batch, not per item, so spawn cost amortises.
//! - `black_box` on the popped value so LLVM cannot prove the drain dead.
//! - Threads are NOT pinned. Read the SHAPE, not the third digit. The *unchanged*
//!   baseline arms have been observed to drift 3–5% between runs with `p = 0.00`:
//!   anything under ~5% here is noise, not a result.
//!
//! ## Note on the sample budget (stage R0 leaks!)
//! `SegQueue` is at stage R0: retired segments are never freed, ~536 B per 32
//! items (~17 MB per million). The groups below therefore run deliberately short
//! measurements. Do NOT raise `measurement_time` until reclamation lands, or the
//! allocator pressure will both distort the numbers and exhaust memory.
//!
//! Results and the reasoning behind each round live in
//! `notes/seg_queue_bench_results.md`.

use concurrent::{Backoff, SegQueue};
use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use std::collections::VecDeque;
use std::hint::{black_box, spin_loop};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Minimal shape shared by both queues, so one generic body benches both and
/// adding a scenario or a policy does not duplicate it.
trait Queue<T>: Send + Sync {
    fn create() -> Self;
    fn push(&self, value: T);
    fn pop(&self) -> Option<T>;
}

impl<T: Send> Queue<T> for SegQueue<T> {
    fn create() -> Self {
        SegQueue::new()
    }
    fn push(&self, value: T) {
        SegQueue::push(self, value);
    }
    fn pop(&self) -> Option<T> {
        SegQueue::pop(self)
    }
}

/// The honest lock-based baseline: unbounded, MPMC, FIFO, ~15 lines.
struct MutexQueue<T>(Mutex<VecDeque<T>>);

impl<T: Send> Queue<T> for MutexQueue<T> {
    fn create() -> Self {
        Self(Mutex::new(VecDeque::new()))
    }
    fn push(&self, value: T) {
        self.0.lock().unwrap().push_back(value);
    }
    fn pop(&self) -> Option<T> {
        self.0.lock().unwrap().pop_front()
    }
}

/// What a consumer does when `pop()` reports the queue empty. See the module doc:
/// this is the caller's decision, and it dominates the measurement.
#[derive(Copy, Clone)]
enum Policy {
    Spin,
    Backoff,
    Yield,
}

impl Policy {
    fn name(self) -> &'static str {
        match self {
            Policy::Spin => "spin",
            Policy::Backoff => "backoff",
            Policy::Yield => "yield",
        }
    }
}

/// Per-consumer waiter. `Backoff` is per-thread (`!Sync`), so each consumer builds
/// its own; the other two policies are stateless.
struct Waiter {
    policy: Policy,
    backoff: Backoff,
}

impl Waiter {
    fn new(policy: Policy) -> Self {
        Self {
            policy,
            backoff: Backoff::new(),
        }
    }

    fn wait(&self) {
        match self.policy {
            Policy::Spin => spin_loop(),
            Policy::Backoff => self.backoff.snooze(),
            Policy::Yield => thread::yield_now(),
        }
    }
}

/// Keep the R0 leak bounded — see the module note.
fn configure(g: &mut BenchmarkGroup<'_, WallTime>) {
    g.sample_size(10);
    g.warm_up_time(Duration::from_millis(100));
    g.measurement_time(Duration::from_millis(250));
}

// ---------------------------------------------------------------------------
// 1. Uncontended (control): one thread, push then immediately pop. Never
//    observes an empty queue, so no drain policy applies. This must NOT move
//    when only concurrency behaviour changes.
// ---------------------------------------------------------------------------

fn uncontended(c: &mut Criterion) {
    let mut g = c.benchmark_group("seg_queue_uncontended");
    configure(&mut g);
    bench_uncontended::<SegQueue<usize>>(&mut g, "seg_queue");
    bench_uncontended::<MutexQueue<usize>>(&mut g, "mutex_vecdeque");
    g.finish();
}

fn bench_uncontended<Q: Queue<usize>>(g: &mut BenchmarkGroup<'_, WallTime>, name: &str) {
    g.bench_function(name, |b| {
        let q = Q::create();
        b.iter_custom(|iters| {
            let start = Instant::now();
            for i in 0..iters as usize {
                q.push(black_box(i));
                black_box(q.pop());
            }
            start.elapsed()
        });
    });
}

// ---------------------------------------------------------------------------
// 2. SPSC: 1 producer, 1 consumer — the cleanest cross-core handoff signal,
//    swept across drain policies.
// ---------------------------------------------------------------------------

fn spsc(c: &mut Criterion) {
    for policy in [Policy::Spin, Policy::Backoff, Policy::Yield] {
        let mut g = c.benchmark_group(format!("seg_queue_spsc_{}", policy.name()));
        configure(&mut g);
        bench_spsc::<SegQueue<usize>>(&mut g, "seg_queue", policy);
        bench_spsc::<MutexQueue<usize>>(&mut g, "mutex_vecdeque", policy);
        g.finish();
    }
}

fn bench_spsc<Q: Queue<usize> + 'static>(
    g: &mut BenchmarkGroup<'_, WallTime>,
    name: &str,
    policy: Policy,
) {
    g.bench_function(name, |b| {
        b.iter_custom(|iters| {
            let n = iters as usize;
            let q = Arc::new(Q::create());

            let start = Instant::now();
            let prod = {
                let q = Arc::clone(&q);
                thread::spawn(move || {
                    for i in 0..n {
                        q.push(i);
                    }
                })
            };

            let waiter = Waiter::new(policy);
            let mut got = 0usize;
            while got < n {
                match q.pop() {
                    Some(v) => {
                        black_box(v);
                        got += 1;
                    }
                    None => waiter.wait(),
                }
            }
            prod.join().unwrap();
            start.elapsed()
        });
    });
}

// ---------------------------------------------------------------------------
// 3. MPMC. 2p2c holds the policy fixed (backoff) as the representative config;
//    4p4c sweeps all three, because that is the config where the policy
//    inverted the verdict against the mutex.
// ---------------------------------------------------------------------------

fn mpmc(c: &mut Criterion) {
    mpmc_config(c, 2, 2, Policy::Backoff);
    for policy in [Policy::Spin, Policy::Backoff, Policy::Yield] {
        mpmc_config(c, 4, 4, policy);
    }
}

fn mpmc_config(c: &mut Criterion, nprod: usize, ncon: usize, policy: Policy) {
    let mut g = c.benchmark_group(format!("seg_queue_mpmc_{nprod}p{ncon}c_{}", policy.name()));
    configure(&mut g);
    bench_mpmc::<SegQueue<usize>>(&mut g, "seg_queue", nprod, ncon, policy);
    bench_mpmc::<MutexQueue<usize>>(&mut g, "mutex_vecdeque", nprod, ncon, policy);
    g.finish();
}

fn bench_mpmc<Q: Queue<usize> + 'static>(
    g: &mut BenchmarkGroup<'_, WallTime>,
    name: &str,
    nprod: usize,
    ncon: usize,
    policy: Policy,
) {
    g.bench_function(name, |b| {
        b.iter_custom(|iters| {
            let per_prod = (iters as usize / nprod).max(1);
            let total = per_prod * nprod;
            let q = Arc::new(Q::create());
            let consumed = Arc::new(AtomicUsize::new(0));

            let start = Instant::now();

            let producers: Vec<_> = (0..nprod)
                .map(|_| {
                    let q = Arc::clone(&q);
                    thread::spawn(move || {
                        for i in 0..per_prod {
                            q.push(i);
                        }
                    })
                })
                .collect();

            let consumers: Vec<_> = (0..ncon)
                .map(|_| {
                    let q = Arc::clone(&q);
                    let done = Arc::clone(&consumed);
                    thread::spawn(move || {
                        let waiter = Waiter::new(policy);
                        loop {
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
                                    waiter.wait();
                                }
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
            elapsed
                .mul_f64(iters as f64 / total as f64)
                .max(Duration::ZERO)
        });
    });
}

criterion_group!(benches, uncontended, spsc, mpmc);
criterion_main!(benches);
