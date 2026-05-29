mod sync {
    // Under a normal build the atomics, Arc, and the futex wait/wake come from
    // std + the `atomic-wait` crate. Under `--cfg loom` they must ALL be routed
    // through loom so the model checker can intercept every memory operation —
    // a single std atomic that slips through makes loom blind to that access.
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{AtomicU32, Ordering};
    #[cfg(not(loom))]
    pub(super) use std::sync::Arc;

    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{AtomicU32, Ordering};
    #[cfg(loom)]
    pub(super) use loom::sync::Arc;

    #[cfg(not(loom))]
    pub(super) fn wait(atomic: &AtomicU32, expected: u32) {
        atomic_wait::wait(atomic, expected);
    }

    #[cfg(not(loom))]
    pub(super) fn wake_one(atomic: &AtomicU32) {
        atomic_wait::wake_one(atomic);
    }

    // Loom has no futex. Model the blocking wait as a spin that hands control
    // back to the scheduler: loom explores every interleaving, so the unpark
    // thread is always eventually scheduled and this load observes the state
    // change — the spin can never deadlock the model. Critically this adds NO
    // synchronization of its own (no mutex/condvar): the only ordering edge is
    // whatever `atomic`'s own load establishes, so the loom model still tests
    // the Parker's real acquire/release choices rather than masking them.
    #[cfg(loom)]
    pub(super) fn wait(atomic: &AtomicU32, expected: u32) {
        while atomic.load(Ordering::Acquire) == expected {
            loom::thread::yield_now();
        }
    }

    // No-op under loom: the spinning waiter re-reads `atomic` on every scheduler
    // turn, so the state write in `unpark` IS the wakeup — there is nothing else
    // to signal.
    #[cfg(loom)]
    pub(super) fn wake_one(_atomic: &AtomicU32) {}
}
use sync::{wait, wake_one, Arc, AtomicU32, Ordering};

struct Inner {
    state: AtomicU32,
}

const EMPTY: u32 = 0;
const PARKED: u32 = 1;
const NOTIFIED: u32 = 2;

pub struct Parker {
    inner: Arc<Inner>,
}

#[derive(Clone)]
pub struct Unparker {
    inner: Arc<Inner>,
}

impl Parker {
    pub fn new() -> Self {
        let inner = Arc::new(Inner::new());
        Self { inner }
    }

    pub fn unparker(&self) -> Unparker {
        Unparker {
            inner: self.inner.clone(),
        }
    }

    pub fn park(&self) {
        match self
            .inner
            .state
            .compare_exchange(EMPTY, PARKED, Ordering::Acquire, Ordering::Relaxed)
        {
            Err(NOTIFIED) => {
                self.inner.state.store(EMPTY, Ordering::Release); // See NOTIFIED then no park, continue to work
            }
            Ok(_) => loop {
                wait(&self.inner.state, PARKED);
                match self.inner.state.compare_exchange(
                    NOTIFIED,
                    EMPTY,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return,
                    Err(_) => continue,
                }
            },
            _ => unreachable!(),
        }
    }
}

#[cfg(all(test, not(loom)))]
impl Parker {
    /// Test-only window into the raw state word so a unit test can assert the
    /// 3-state machine's transitions directly (`EMPTY` / `PARKED` / `NOTIFIED`)
    /// instead of inferring them from timing. Not part of the public API.
    pub(crate) fn __state_for_test(&self) -> u32 {
        self.inner.state.load(Ordering::Relaxed)
    }
}

impl Unparker {
    pub fn unpark(&self) {
        match self.inner.state.swap(NOTIFIED, Ordering::Acquire) {
            EMPTY => return,
            NOTIFIED => return,
            PARKED => wake_one(&self.inner.state),
            _ => unreachable!(),
        }
    }
}

impl Inner {
    fn new() -> Self {
        Self {
            state: AtomicU32::new(EMPTY),
        }
    }
}

// These tests use real OS threads, sleeps, and the real futex; under `--cfg
// loom` the Parker's atomics become loom atomics that panic outside
// `loom::model`, so this whole module is compiled out of loom builds.
#[cfg(all(test, not(loom)))]
mod tests {
    use super::{EMPTY, NOTIFIED};
    use crate::Parker;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn parker_smoke_unpark_then_park_returns_immediately() {
        let instant = Instant::now();
        let parker = Parker::new();
        let unparker = parker.unparker();

        unparker.unpark();
        parker.park();
        assert!(instant.elapsed().as_millis() < 100);
    }

    #[test]
    fn parker_smoke_park_then_unpark_unblocks() {
        let parker = Parker::new();
        let unparker = parker.unparker();
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            unparker.unpark();
        });

        let start = Instant::now();
        parker.park(); // A block ở đây
        let elapsed = start.elapsed();

        handle.join().unwrap();
        assert!(
            elapsed >= Duration::from_millis(8),
            "park returned too quickly ({:?}), didn't actually sleep",
            elapsed
        );
        assert!(
            elapsed < Duration::from_millis(500),
            "park took too long ({:?}), possible deadlock",
            elapsed
        );
    }

    #[test]
    fn parker_park_actually_blocks_until_unpark() {
        let parker = Parker::new();
        let unparker = parker.unparker();

        let handle = thread::spawn(move || {
            let start = Instant::now();
            parker.park();
            start.elapsed()
        });
        thread::sleep(Duration::from_millis(50));
        unparker.unpark();
        let park_elapsed = handle.join().unwrap();
        assert!(
            park_elapsed >= Duration::from_millis(40),
            "park returned in {}ms — it didn't actually block",
            park_elapsed.as_millis()
        );
        assert!(park_elapsed < Duration::from_millis(200), "park hung");
    }

    #[test]
    fn parker_park_unpark_cycles_repeatable() {
        let parker = Parker::new();
        let unparker = parker.unparker();
        let handle = thread::spawn(move || {
            let mut elapsed_per_cycle = Vec::new();
            for _ in 0..10 {
                let start = Instant::now();
                parker.park();
                elapsed_per_cycle.push(start.elapsed());
            }
            elapsed_per_cycle
        });
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(20));
            unparker.unpark();
        }
        let elapsed = handle.join().unwrap();
        for (i, e) in elapsed.iter().enumerate() {
            assert!(
                *e >= Duration::from_millis(10),
                "cycle {} returned in {}ms — state machine didn't reset",
                i,
                e.as_millis()
            );
        }
    }

    #[test]
    fn parker_unpark_before_park_is_consumed_then_blocks_next_cycle() {
        let parker = Parker::new();
        let unparker = parker.unparker();

        unparker.unpark();
        parker.park();

        let (parker, unparker) = (parker, unparker);
        let handle = thread::spawn(move || {
            let start = Instant::now();
            parker.park();
            start.elapsed()
        });
        thread::sleep(Duration::from_millis(50));
        unparker.unpark();
        let elapsed = handle.join().unwrap();
        assert!(
            elapsed >= Duration::from_millis(40),
            "second park returned in {}ms — first park left state corrupted",
            elapsed.as_millis()
        );
    }

    /// Direct state-machine walk via `__state_for_test()` — no timing guesswork.
    /// Single-threaded on purpose: every operation here is non-blocking, so we
    /// can assert the exact state word after each transition. (A bare `park()`
    /// from EMPTY *would* block forever, which is why the final step only
    /// asserts we're back at the clean EMPTY start state rather than calling
    /// `park()` again.)
    #[test]
    fn parker_state_invariants() {
        let parker = Parker::new();
        let unparker = parker.unparker();

        // Fresh parker starts EMPTY.
        assert_eq!(parker.__state_for_test(), EMPTY, "new Parker should be EMPTY");

        // unpark() on a never-parked Parker: EMPTY -> NOTIFIED, no waiter to
        // wake (the `swap` sees EMPTY, so no `wake_one`/futex syscall fires).
        unparker.unpark();
        assert_eq!(
            parker.__state_for_test(),
            NOTIFIED,
            "unpark from EMPTY should leave NOTIFIED"
        );

        // unpark() from NOTIFIED is idempotent: the `swap` sees NOTIFIED and
        // returns early. A second (and this is the (park, unpark, unpark)
        // sequence's tail) unpark must NOT arm a second wake.
        unparker.unpark();
        assert_eq!(
            parker.__state_for_test(),
            NOTIFIED,
            "redundant unpark must stay NOTIFIED, not double-arm"
        );

        // park() from NOTIFIED returns instantly and consumes the token,
        // resetting to EMPTY — it must never enter the futex_wait slow path.
        let start = Instant::now();
        parker.park();
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "park from NOTIFIED blocked ({:?}) — it took the slow path",
            start.elapsed()
        );
        assert_eq!(
            parker.__state_for_test(),
            EMPTY,
            "park must consume the NOTIFIED token back to EMPTY"
        );

        // Back at the clean start state: the (park, unpark, unpark) sequence
        // left no orphaned NOTIFIED/PARKED residue and produced exactly one
        // wake's worth of effect, not two.
    }
}

// Loom model-checked tests. Compiled only under `--cfg loom`; run with:
//   RUSTFLAGS="--cfg loom" cargo test --release -p concurrent --lib loom
#[cfg(all(test, loom))]
mod loom_tests {
    use crate::Parker;
    use loom::sync::Arc;
    use std::time::{Duration, Instant};

    /// Loom explores ALL interleavings of the park-vs-unpark race; the model
    /// passing under every one of them means no schedule produces a lost wakeup
    /// (i.e. park is never left blocked after an unpark).
    #[test]
    fn parker_loom_lost_wakeup_safe() {
        loom::model(|| {
            let parker = Parker::new();
            let unparker = parker.unparker();
            let t1 = loom::thread::spawn(move || unparker.unpark());
            let t2 = loom::thread::spawn(move || parker.park());
            t1.join().unwrap();
            t2.join().unwrap();
            // t2.join() completing under EVERY interleaving = no lost wakeup.
        });
    }

    /// Negative control: proves loom is actually intercepting the atomics. A
    /// 2-thread model over the Parker's handful of atomic ops must take real
    /// wall-clock time to explore exhaustively; a sub-100ms pass means the
    /// atomics weren't routed through `loom::sync::atomic` and loom saw nothing.
    #[test]
    fn parker_loom_test_must_be_slow() {
        let start = Instant::now();
        loom::model(|| {
            let parker = Parker::new();
            let unparker = parker.unparker();
            let t = loom::thread::spawn(move || unparker.unpark());
            parker.park();
            t.join().unwrap();
        });
        assert!(
            start.elapsed() >= Duration::from_millis(100),
            "loom test passed in {}ms — atomics aren't routed to loom::sync::atomic",
            start.elapsed().as_millis()
        );
    }

    /// Proves `unpark` must publish prior writes with Release ordering. The
    /// producer writes `payload` via Relaxed *before* unpark; the consumer wakes
    /// and reads it. The read can only be guaranteed to see 42 if unpark's state
    /// write releases the producer's prior stores and park's read acquires them.
    #[test]
    fn parker_unpark_release_publishes_data() {
        loom::model(|| {
            use loom::sync::atomic::{AtomicU32, Ordering};
            let parker = Parker::new();
            let unparker = parker.unparker();
            let payload = Arc::new(AtomicU32::new(0));
            let payload_c = payload.clone();
            let t1 = loom::thread::spawn(move || {
                payload_c.store(42, Ordering::Relaxed);
                unparker.unpark();
            });
            parker.park();
            let observed = payload.load(Ordering::Relaxed);
            t1.join().unwrap();
            assert_eq!(
                observed, 42,
                "park did not synchronize with producer's prior writes"
            );
        });
    }
}
