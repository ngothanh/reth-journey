//! Zero-allocation proof for the semaphore's two hot paths.
//!
//! WHAT THIS PROVES: after the intrusive-list rewrite, neither acquiring from
//! the counter NOR going through the wait queue (enqueue -> grant -> consume)
//! ever touches the heap allocator. This is the measurable payoff of stage 6 —
//! the old Slab/VecDeque backend allocated on the queue path; the intrusive
//! design must not. If anyone ever reintroduces an allocation (a stray clone,
//! a Box, a Vec), this test turns red.
//!
//! HOW: we install a global allocator that counts every `alloc` call in the
//! whole process, run 10,000 iterations of each path, and assert the counter
//! did not move. Own file, because #[global_allocator] is process-wide.

use concurrent::Semaphore;
use std::alloc::{GlobalAlloc, Layout, System};
use std::future::Future;
use std::pin::pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

/// Wraps the system allocator; every heap allocation bumps ALLOCS.
struct CountingAlloc;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static A: CountingAlloc = CountingAlloc;

/// ONE test running both paths back to back — two #[test] fns would run on
/// two threads in the same process, and the other test's harness output
/// (string formatting) would leak into our process-global counter.
#[test]
fn hot_paths_are_zero_alloc() {
    counter_path();
    queue_path();
}

/// Path A: permits available -> try_acquire takes from the counter, drop
/// returns to the counter. Never touches the wait queue.
fn counter_path() {
    let sem = Semaphore::new(1);

    // Warm-up (outside the measured window): first touch of anything
    // lazily initialized happens here, not inside the measurement.
    drop(sem.try_acquire().unwrap());

    let before = ALLOCS.load(Ordering::Relaxed);
    for _ in 0..10_000 {
        let p = sem.try_acquire().unwrap();
        drop(p); // Drop -> add_permits(1): lock, no waiters, counter += 1
    }
    let allocs = ALLOCS.load(Ordering::Relaxed) - before;
    assert_eq!(allocs, 0, "counter path allocated {allocs} times in 10k ops");
}

/// Path B: the intrusive-list path. Semaphore starts at 0 permits, so every
/// cycle is forced through: enqueue (Idle->Waiting) -> grant
/// (Waiting->Granted) -> consume (Granted->Done).
fn queue_path() {
    let sem = Semaphore::new(0);
    let waker = Waker::noop();               // waker whose clone/wake are free
    let mut cx = Context::from_waker(waker); // built once, reused every cycle

    queue_cycle(&sem, &mut cx); // warm-up, outside the window

    let before = ALLOCS.load(Ordering::Relaxed);
    for _ in 0..10_000 {
        queue_cycle(&sem, &mut cx);
    }
    let allocs = ALLOCS.load(Ordering::Relaxed) - before;
    assert_eq!(allocs, 0, "queue path allocated {allocs} times in 10k ops");
}

/// One full trip through the wait queue. The permit is deliberately LEAKED
/// (mem::forget) at the end — see the comment inside for why.
fn queue_cycle(sem: &Semaphore, cx: &mut Context<'_>) {
    // pin! keeps the future on the STACK. Box::pin would heap-allocate and
    // the measurement would count the Box, not the semaphore.
    let mut fut = pin!(sem.acquire());

    // Poll #1: counter is 0, so the future must enqueue its node into the
    // intrusive list and return Pending. (This is the path under test.)
    assert!(fut.as_mut().poll(cx).is_pending());

    // Release one permit: hand-off pops our node, marks it Granted, wakes
    // the noop waker. The permit goes into the NODE — the counter stays 0.
    sem.add_permits(1);

    // Poll #2: the future finds its node Granted and consumes the permit.
    match fut.as_mut().poll(cx) {
        Poll::Ready(Ok(permit)) => {
            // WHY forget, not drop:
            //   drop(permit) would run its Drop -> add_permits(1) -> queue is
            //   empty now -> counter becomes 1. Next cycle's poll #1 would
            //   then take the fast path and NEVER ENQUEUE — from iteration 2
            //   on we'd be measuring path A again, and this test would prove
            //   nothing (green, but meaningless).
            //   forget(permit) skips that Drop, so the counter stays 0 and
            //   every cycle really goes through the queue. It is safe because
            //   SemaphorePermit owns no heap — it's just a reference plus a
            //   claim on one permit, and retiring that claim is exactly what
            //   we want (we minted one with add_permits, we retire one here;
            //   net zero per cycle).
            //   NEVER do this to the FUTURE: forgetting a queued Acquire
            //   skips the Drop that unlinks its node -> the queue keeps a
            //   pointer into freed stack memory -> use-after-free.
            std::mem::forget(permit);
        }
        _ => unreachable!("permit was granted; second poll must be Ready"),
    }
    // fut drops here in state Done -> its Drop is a no-op. Clean.
}
