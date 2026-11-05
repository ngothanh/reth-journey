//! Cost of `push` on ONE pile when many threads retire at the same time (C2, L17).
//!
//! The question: is a single `head` a hot spot? Every `push` ends with a
//! compare-exchange on the same cache line, so with N threads that line has to
//! move between cores on almost every push. If this bench shows no real cost,
//! the 8 shards of L18 have no reason to exist.
//!
//! ## Prediction (written before the first run, 2026-11-05)
//!
//! | threads | ns per push, as one thread sees it | pushes per second, all threads |
//! |---:|---:|---|
//! | 1 | ≤ 10 | — |
//! | 8 | 42 | fewer than with 1 thread |
//!
//! Reasoning: one uncontended exchange is cheap; with 8 threads the threads
//! fight over `head`, so each push waits longer.
//!
//! (Note added before running: 42 ns per push on each of 8 threads is
//! 8 / 42 ns ≈ 190 M pushes/s in total, which is MORE than 1 / 10 ns = 100 M/s.
//! So the two halves of the prediction disagree with each other; the run
//! decides which half is wrong.)
//!
//! ## Measured (2026-11-05, three runs of the whole bench)
//!
//! Criterion's number is wall time per push for the whole machine; the
//! one-thread view is that × threads.
//!
//! | threads | ns per push, whole machine | ns per push, one thread's view | pushes/s, all threads |
//! |---:|---:|---:|---:|
//! | 1 | 1.9 – 2.4 | 1.9 – 2.4 | ≈ 450 M |
//! | 2 | 6.9 – 7.2 | ≈ 14 | ≈ 140 M |
//! | 4 | 15.9 – 19.1 | ≈ 70 | ≈ 55 M |
//! | 8 | 55 – 135 (noisy) | ≈ 440 – 1080 | ≈ 7 – 18 M |
//!
//! Against the prediction:
//!
//! - 1 thread, ≤ 10 ns: right. About 2 ns: the exchange hits a line already in
//!   this core's cache.
//! - 8 threads, 42 ns as one thread sees it: wrong by 10–25×. Each push needs
//!   `head`'s line for itself; the line travels to that core, and every failed
//!   exchange sends it travelling again for nothing.
//! - fewer pushes in total with 8 threads: right, and by far more than
//!   expected: 25–60× fewer than ONE thread. Two threads are already 3× slower
//!   in total than one. Adding threads to one head does not add work, it only
//!   adds traffic for one cache line.
//! - The 8-thread row moves between runs (55, 108, 135 ns): half the threads
//!   are on the efficiency cores, and where the OS puts them changes each run.
//!
//! This is the worst case: threads that do nothing but retire. A SegQueue
//! retires one segment per 32 pops, so its real pressure on `head` is far
//! lower. Whether the shards of L18 pay off has to be judged at that rate too.
//!
//! ## How it is measured
//!
//! - Only `push` is timed. Every node is allocated before the clock starts, and
//!   `take_all` + `free_all` run after it stops, so `malloc`/`free` are not in
//!   the number.
//! - All threads wait on a `Barrier`, then push their share into the same pile.
//!   The clock runs from the barrier to the last `join`.
//! - Criterion's "time" is wall time divided by the total number of pushes, so
//!   it is the cost of one push **for the whole machine**. One thread sees
//!   `threads ×` that.
//!
//! Machine: Apple M2, 4 performance cores + 4 efficiency cores. With 8 threads,
//! half run on the slow cores and `head`'s cache line crosses between the two
//! clusters.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use reclaim::bench_internals::RetiredList;
use reclaim::{Retirable, RetireLink};
use std::sync::Barrier;
use std::time::{Duration, Instant};

/// The smallest retirable object: just a link.
struct Node {
    link: RetireLink,
}

unsafe impl Retirable for Node {
    fn retire_link(&self) -> &RetireLink {
        &self.link
    }

    unsafe fn reclaim(ptr: *mut Self) {
        // SAFETY: made with `Box::new`, and `free_all` frees each node once.
        unsafe { drop(Box::from_raw(ptr)) };
    }
}

/// A raw node address that can be moved into a pushing thread.
struct SendPtr(*mut Node);
// SAFETY: only the address moves; each node is pushed by exactly one thread.
unsafe impl Send for SendPtr {}

fn one_head(c: &mut Criterion) {
    let mut group = c.benchmark_group("retired_list/one_head");
    group.measurement_time(Duration::from_secs(5));

    for threads in [1usize, 2, 4, 8] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(threads), &threads, |b, &threads| {
            b.iter_custom(|iters| {
                let per_thread = (iters as usize).div_ceil(threads);
                let list = RetiredList::new();
                let barrier = Barrier::new(threads + 1);

                // Untimed: every node exists before the clock starts.
                let batches: Vec<Vec<SendPtr>> = (0..threads)
                    .map(|_| {
                        (0..per_thread)
                            .map(|_| {
                                SendPtr(Box::into_raw(Box::new(Node {
                                    link: RetireLink::new(),
                                })))
                            })
                            .collect()
                    })
                    .collect();

                let start = std::thread::scope(|scope| {
                    for batch in batches {
                        let (list, barrier) = (&list, &barrier);
                        scope.spawn(move || {
                            barrier.wait();
                            for SendPtr(node) in batch {
                                // SAFETY: a fresh node, pushed once, not used after.
                                unsafe { list.push(node) };
                            }
                        });
                    }
                    barrier.wait();
                    Instant::now()
                    // The scope joins every pusher before it returns.
                });
                let elapsed = start.elapsed();

                // Untimed: free everything.
                // SAFETY: this thread took the whole chain; no reader exists.
                unsafe { RetiredList::free_all(list.take_all()) };

                // Report per push, for the whole machine.
                elapsed.mul_f64(iters as f64 / (per_thread * threads) as f64)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, one_head);
criterion_main!(benches);
