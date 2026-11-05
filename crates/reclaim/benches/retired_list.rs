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
//! ## L18: eight heads — prediction (written before the run, 2026-11-05)
//!
//! | threads | ns per push, one thread's view |
//! |---:|---:|
//! | 1 | 3 (≈ 1 ns more than one head, to pick the shard) |
//! | 8 | 100 (less contention, but not much less) |
//!
//! ## L18: measured (2026-11-05, three runs, both groups in each run)
//!
//! Whole-machine ns per push; one thread's view is that × threads.
//!
//! | threads | one head | sharded | sharded, one thread's view | sharded, pushes/s total |
//! |---:|---:|---:|---:|---:|
//! | 1 | 1.8 – 2.4 | 1.9 – 2.3 | ≈ 2 | ≈ 500 M |
//! | 2 | 6.6 – 7.2 | 5.0 – 5.2 | ≈ 10 | ≈ 195 M |
//! | 4 | 15.7 – 16.1 | 9.0 – 9.1 | ≈ 36 | ≈ 110 M |
//! | 8 | 100 – 122 | 21 – 29 | ≈ 170 – 230 | ≈ 35 – 47 M |
//!
//! Against the prediction:
//!
//! - 1 thread, 3 ns: the shard costs nothing measurable. One shift and one mask
//!   are under a nanosecond, inside the run-to-run noise.
//! - 8 threads, 100 ns as one thread sees it: about 2× too optimistic. Sharding
//!   helps (≈ 4–5× at 8 threads), but much less than "8 heads, 8 threads"
//!   suggests.
//! - The open question: with 8 threads on 8 shards, the machine still does
//!   ≈ 10× fewer pushes per second than ONE thread on one head.
//!
//! The answer: all 8 `head`s (8 bytes each) sat in one 128-byte cache line.
//! The exchanges stopped failing on each other, but the line still travelled
//! between cores on almost every push: false sharing.
//!
//! ## L18: each shard on its own 128-byte line (2026-11-05, three runs)
//!
//! No prediction was written for this run.
//!
//! | threads | one head | sharded, unpadded | sharded, padded | padded, one thread's view | padded, pushes/s total |
//! |---:|---:|---:|---:|---:|---:|
//! | 1 | 1.7 – 1.9 | 1.9 – 2.3 | 1.9 | ≈ 2 | ≈ 525 M |
//! | 2 | 5.9 – 6.8 | 5.0 – 5.2 | 2.3 – 2.5 | ≈ 5 | ≈ 415 M |
//! | 4 | 15.6 – 15.9 | 9.0 – 9.1 | 2.4 – 2.7 | ≈ 10 | ≈ 390 M |
//! | 8 | 118 – 131 | 21 – 29 | 5.9 – 8.9 | ≈ 48 – 71 | ≈ 110 – 170 M |
//!
//! - Padding is worth 3–4× on top of sharding; at 8 threads, padded shards are
//!   ≈ 15–20× faster than one head.
//! - Still not solved: more threads do not give more pushes in total. 2 and 4
//!   threads stay just below one thread; 8 fall to about a third. The pile is
//!   no longer the obvious limit, but this bench does not say what is.
//!
//! ## L18: the control — one pile per thread (2026-11-05, three runs)
//!
//! Each thread pushes into a pile nobody else touches, each on its own line.
//! Nothing is shared, so this is the best any shared pile could reach here.
//! (`concurrent_utils::CachePadded` replaced the local `Padded` before this
//! run; the sharded numbers did not move.)
//!
//! | threads | one head | sharded, padded | one pile per thread | pushes/s total, control |
//! |---:|---:|---:|---:|---:|
//! | 1 | 1.7 – 2.1 | 1.8 – 2.1 | 2.0 – 2.3 | ≈ 480 M |
//! | 2 | 6.3 – 7.2 | 2.4 – 2.5 | 1.2 – 1.6 | ≈ 750 M |
//! | 4 | 15.9 – 16.0 | 2.4 – 2.6 | 1.2 | ≈ 830 M |
//! | 8 | 118 – 133 | 5.7 – 6.5 | 0.7 – 0.8 | ≈ 1.3 G |
//!
//! What it says, for each of the three guesses after the padded run:
//!
//! - **The machine itself does not scale well.** Even with nothing shared,
//!   4 threads push no more than 2, and 8 threads give under 3× one thread.
//!   Some of the gap is not the pile: allocating every node on the main
//!   thread (its lines start in another core's cache) and the 4 efficiency
//!   cores both cost something that no pile design removes.
//! - **Sharing still costs, and most at 8 threads.** Sharded is ≈ 2× below
//!   the control at 2 and 4 threads, and ≈ 8× below it at 8. With 8 threads on
//!   8 shards, two threads are often in the same shard at once, and each
//!   thread pushes ≈ 8 neighbouring nodes in a row into the same shard.
//! - So the next step (L19, a run of ~20 objects pushed with ONE exchange)
//!   attacks the part that is left: fewer exchanges on a shared `head` means
//!   fewer moments when two threads want the same line.
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
use reclaim::bench_internals::{RetiredList, ShardedList};
use concurrent_utils::CachePadded;
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

/// A pile the bench can push into and empty: one head, or eight.
trait Pile: Sync {
    fn new() -> Self;
    /// `thread` is the pushing thread's number, `0..threads`. Only the
    /// one-pile-per-thread control uses it.
    ///
    /// # Safety
    /// `node` is fresh, pushed once, and not used after.
    unsafe fn push(&self, thread: usize, node: *mut Node);
    fn free_everything(&self);
}

impl Pile for RetiredList {
    fn new() -> Self {
        RetiredList::new()
    }
    unsafe fn push(&self, _thread: usize, node: *mut Node) {
        // SAFETY: the caller keeps `push`'s promises.
        unsafe { RetiredList::push(self, node) }
    }
    fn free_everything(&self) {
        // SAFETY: this thread took the whole chain; no reader exists.
        unsafe { RetiredList::free_all(self.take_all()) }
    }
}

impl Pile for ShardedList {
    fn new() -> Self {
        ShardedList::new()
    }
    unsafe fn push(&self, _thread: usize, node: *mut Node) {
        // SAFETY: the caller keeps `push`'s promises.
        unsafe { ShardedList::push(self, node) }
    }
    fn free_everything(&self) {
        for chain in self.take_all() {
            // SAFETY: this thread took every chain; no reader exists.
            unsafe { RetiredList::free_all(chain) }
        }
    }
}

/// The control: every thread has a pile of its own, on its own cache line.
/// Nothing is shared between pushing threads, so this is the best any shared
/// pile could do on this machine. Not a design: a pile nobody else can take
/// from cannot be emptied by the thread that runs a round.
struct OnePilePerThread([CachePadded<RetiredList>; 8]);

impl Pile for OnePilePerThread {
    fn new() -> Self {
        OnePilePerThread(std::array::from_fn(|_| CachePadded::new(RetiredList::new())))
    }
    unsafe fn push(&self, thread: usize, node: *mut Node) {
        // SAFETY: the caller keeps `push`'s promises.
        unsafe { self.0[thread].push(node) }
    }
    fn free_everything(&self) {
        for pile in &self.0 {
            // SAFETY: this thread took the whole chain; no reader exists.
            unsafe { RetiredList::free_all(pile.take_all()) }
        }
    }
}

/// `threads` threads push `iters` nodes in total into one `P`, all at once.
/// Returns the time per push for the whole machine, scaled to `iters`.
fn timed_pushes<P: Pile>(threads: usize, iters: u64) -> Duration {
    let per_thread = (iters as usize).div_ceil(threads);
    let pile = P::new();
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
        for (thread, batch) in batches.into_iter().enumerate() {
            let (pile, barrier) = (&pile, &barrier);
            scope.spawn(move || {
                barrier.wait();
                for SendPtr(node) in batch {
                    // SAFETY: a fresh node, pushed once, not used after.
                    unsafe { pile.push(thread, node) };
                }
            });
        }
        barrier.wait();
        Instant::now()
        // The scope joins every pusher before it returns.
    });
    let elapsed = start.elapsed();

    // Untimed: free everything.
    pile.free_everything();

    elapsed.mul_f64(iters as f64 / (per_thread * threads) as f64)
}

fn bench_pile<P: Pile>(c: &mut Criterion, name: &str) {
    let mut group = c.benchmark_group(format!("retired_list/{name}"));
    group.measurement_time(Duration::from_secs(5));
    group.throughput(Throughput::Elements(1));
    for threads in [1usize, 2, 4, 8] {
        group.bench_with_input(BenchmarkId::from_parameter(threads), &threads, |b, &threads| {
            b.iter_custom(|iters| timed_pushes::<P>(threads, iters));
        });
    }
    group.finish();
}

fn one_head(c: &mut Criterion) {
    bench_pile::<RetiredList>(c, "one_head");
}

fn sharded(c: &mut Criterion) {
    bench_pile::<ShardedList>(c, "sharded");
}

fn one_pile_per_thread(c: &mut Criterion) {
    bench_pile::<OnePilePerThread>(c, "one_pile_per_thread");
}

criterion_group!(benches, one_head, sharded, one_pile_per_thread);
criterion_main!(benches);
