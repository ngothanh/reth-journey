//! Concurrency drill #3 — two-flag mutual exclusion.
//!
//! Three variants of the same protocol, differing **only** in memory ordering, so the
//! loom model and the bench can run against each in turn:
//!
//! | Variant | Set | Peek | Between |
//! |---|---|---|---|
//! | [`DekkerAcqRel`] | `Release` | `Acquire` | nothing |
//! | [`DekkerFence`] | `Release` | `Acquire` | `fence(SeqCst)` |
//! | [`DekkerSeqCst`] | `SeqCst` | `SeqCst` | nothing |
//!
//! The lock is *only* two `AtomicBool`s — no counter, no `fetch_add`, no
//! `compare_exchange`. Each flag has exactly one writer, which is what makes plain
//! stores sufficient and an atomic read-modify-write unnecessary.

mod sync {
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{fence, AtomicBool, Ordering};

    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{fence, AtomicBool, Ordering};

    /// How many retries loom explores before pruning the branch.
    ///
    /// In the model each thread enters the critical section once, so a schedule where
    /// one side has already retried this many times is one where the other side simply
    /// has not been scheduled yet. Those schedules say nothing about mutual exclusion,
    /// and there are infinitely many of them.
    #[cfg(loom)]
    const LOOM_RETRY_BUDGET: u32 = 3;

    /// Backoff hint for the retry path.
    ///
    /// Two loom problems, one function. A bare `spin_loop` gives loom no preemption
    /// point, so it re-runs the spinner forever and never schedules the other thread's
    /// release — the `atomic_cell.rs` problem, fixed by yielding. But yielding alone
    /// makes every retry a fresh branch point, so the model tree is unbounded and loom
    /// gives up with "exceeded maximum number of branches". `skip_branch` prunes the
    /// path once the budget is spent: loom stops adding branch points, the schedule
    /// becomes deterministic, and this execution runs to completion instead of
    /// exploding. It prunes rather than fails, so a legal-but-unlucky schedule is
    /// abandoned quietly rather than reported as a bug.
    #[cfg(loom)]
    pub(super) fn spin_hint(attempts: &mut u32) {
        *attempts += 1;
        if *attempts == LOOM_RETRY_BUDGET {
            loom::skip_branch();
        }
        loom::thread::yield_now();
    }

    #[cfg(not(loom))]
    pub(super) fn spin_hint(_attempts: &mut u32) {
        core::hint::spin_loop();
    }
}

use sync::{fence, spin_hint, AtomicBool, Ordering};

/// Lets the loom model and the bench run against every variant without duplication.
pub trait TwoFlagLock: Send + Sync + 'static {
    fn new() -> Self;

    /// `side` is 0 or 1 — which of the two threads is calling. Blocks until this side
    /// may enter the critical section.
    fn enter(&self, side: usize);

    /// Must be called by the side that entered.
    fn exit(&self, side: usize);
}

// ---------------------------------------------------------------------------
// Variant 1 — Release on the set, Acquire on the peek. The prediction under test.
// ---------------------------------------------------------------------------

pub struct DekkerAcqRel {
    flag: [AtomicBool; 2],
}

impl TwoFlagLock for DekkerAcqRel {
    fn new() -> Self {
        Self {
            flag: [AtomicBool::new(false), AtomicBool::new(false)],
        }
    }

    fn enter(&self, side: usize) {
        let my_flag = &self.flag[side];
        let their_flag = &self.flag[1 - side];
        let mut attempts = 0u32;

        loop {
            my_flag.store(true, Ordering::Release);
            if !their_flag.load(Ordering::Acquire) {
                break;
            }
            // Contended: drop the claim so the other side can progress, then retry.
            // Holding the flag while spinning would deadlock — that gap is what real
            // Dekker's `turn` variable fills.
            my_flag.store(false, Ordering::Release);
            spin_hint(&mut attempts);
        }
    }

    fn exit(&self, side: usize) {
        self.flag[side].store(false, Ordering::Release);
    }
}

// ---------------------------------------------------------------------------
// Variant 2 — drill step 4. Same orderings, plus a fence between set and peek.
// ---------------------------------------------------------------------------

pub struct DekkerFence {
    flag: [AtomicBool; 2],
}

impl TwoFlagLock for DekkerFence {
    fn new() -> Self {
        Self {
            flag: [AtomicBool::new(false), AtomicBool::new(false)],
        }
    }

    fn enter(&self, side: usize) {
        let my_flag = &self.flag[side];
        let their_flag = &self.flag[1 - side];
        let mut attempts = 0u32;

        loop {
            my_flag.store(true, Ordering::Release);
            fence(Ordering::SeqCst);
            if !their_flag.load(Ordering::Acquire) {
                break;
            }
            my_flag.store(false, Ordering::Release);
            spin_hint(&mut attempts);
        }
    }

    fn exit(&self, side: usize) {
        self.flag[side].store(false, Ordering::Release);
    }
}

// ---------------------------------------------------------------------------
// Variant 3 — drill step 5. SeqCst store and load, no explicit fence.
// ---------------------------------------------------------------------------

pub struct DekkerSeqCst {
    flag: [AtomicBool; 2],
}

impl TwoFlagLock for DekkerSeqCst {
    fn new() -> Self {
        Self {
            flag: [AtomicBool::new(false), AtomicBool::new(false)],
        }
    }

    fn enter(&self, side: usize) {
        let my_flag = &self.flag[side];
        let their_flag = &self.flag[1 - side];
        let mut attempts = 0u32;

        loop {
            my_flag.store(true, Ordering::SeqCst);
            if !their_flag.load(Ordering::SeqCst) {
                break;
            }
            // Contended: drop the claim so the other side can progress, then retry.
            // Holding the flag while spinning would deadlock — that gap is what real
            // Dekker's `turn` variable fills.
            my_flag.store(false, Ordering::SeqCst);
            spin_hint(&mut attempts);
        }
    }

    fn exit(&self, side: usize) {
        self.flag[side].store(false, Ordering::SeqCst);
    }
}

// ---------------------------------------------------------------------------
// Loom model — the drill's instrument.
// ---------------------------------------------------------------------------
//
//   RUSTFLAGS="--cfg loom" cargo test -p concurrent --lib loom_model
//
// ONE critical-section entry per thread. Loom explores every legal interleaving, so the
// state space grows combinatorially in iterations; one entry each is enough to exhibit
// two threads inside, and keeps the model in milliseconds rather than minutes.
//
// The retry path is bounded by `sync::LOOM_RETRY_BUDGET` — without it the model tree is
// unbounded and loom aborts with "exceeded maximum number of branches".
//
// Expected results:
//   acq_rel  — FAILS. Causality violation; this is the drill.
//   fence    — PASSES (~7 s). The fix, model-checked rather than merely stress-tested.
//   seq_cst  — FAILS, but this is a loom limitation, NOT a bug in the algorithm. Loom
//              gives `fence(SeqCst)` a global clock (`rt/thread.rs::seq_cst_fence`, whose
//              only caller is `rt/atomic.rs::fence_seqcst`), but SeqCst *loads and stores*
//              never join it — they get a weaker per-location rule. Dekker's correctness
//              needs the total order across two *different* locations, which loom does not
//              build for plain SeqCst ops. Verify that variant with the real-threads
//              stress test instead.
#[cfg(loom)]
mod loom_model {
    use super::{DekkerAcqRel, DekkerFence, DekkerSeqCst, TwoFlagLock};
    use loom::cell::UnsafeCell;
    use loom::sync::Arc;

    struct Shared<L> {
        lock: L,
        /// Deliberately NOT atomic. An `AtomicUsize::fetch_add` never loses an increment
        /// no matter how many threads are inside, so the test could not fail.
        counter: UnsafeCell<usize>,
    }

    // The claim under test: `lock` serialises access to `counter`. For `DekkerAcqRel`
    // this is false, which is exactly what loom is here to demonstrate.
    unsafe impl<L: Send> Sync for Shared<L> {}

    fn two_threads_one_entry_each<L: TwoFlagLock>() {
        loom::model(|| {
            let shared = Arc::new(Shared {
                lock: L::new(),
                counter: UnsafeCell::new(0usize),
            });
            let other = shared.clone();

            let handle = loom::thread::spawn(move || {
                other.lock.enter(1);
                other.counter.with_mut(|c| unsafe { *c += 1 });
                other.lock.exit(1);
            });

            shared.lock.enter(0);
            shared.counter.with_mut(|c| unsafe { *c += 1 });
            shared.lock.exit(0);

            handle.join().unwrap();

            shared.counter.with(|c| {
                assert_eq!(
                    unsafe { *c },
                    2,
                    "lost an increment — both threads were inside the critical section"
                );
            });
        });
    }

    #[test]
    fn loom_model_acq_rel() {
        two_threads_one_entry_each::<DekkerAcqRel>();
    }

    #[test]
    fn loom_model_fence() {
        two_threads_one_entry_each::<DekkerFence>();
    }

    #[test]
    fn loom_model_seq_cst() {
        two_threads_one_entry_each::<DekkerSeqCst>();
    }
}

// ---------------------------------------------------------------------------
// Real-threads stress. Complements loom rather than replacing it: this shows how loud
// the bug is on real hardware; loom proves whether a fix is actually sound.
// ---------------------------------------------------------------------------
#[cfg(all(test, not(loom)))]
mod tests {
    use super::{DekkerAcqRel, DekkerFence, DekkerSeqCst, TwoFlagLock};
    use core::cell::UnsafeCell;
    use std::sync::Arc;
    use std::thread;

    const ITERS: usize = 1_000_000;

    struct Shared<L> {
        lock: L,
        counter: UnsafeCell<usize>,
    }

    unsafe impl<L: Send> Sync for Shared<L> {}

    /// Returns the final counter. Expected `2 * ITERS` if mutual exclusion holds.
    fn stress<L: TwoFlagLock>() -> usize {
        let shared = Arc::new(Shared {
            lock: L::new(),
            counter: UnsafeCell::new(0usize),
        });
        let other = shared.clone();

        let handle = thread::spawn(move || {
            for _ in 0..ITERS {
                other.lock.enter(1);
                unsafe { *other.counter.get() += 1 };
                other.lock.exit(1);
            }
        });

        for _ in 0..ITERS {
            shared.lock.enter(0);
            unsafe { *shared.counter.get() += 1 };
            shared.lock.exit(0);
        }
        handle.join().unwrap();

        unsafe { *shared.counter.get() }
    }

    /// Documents the broken baseline rather than asserting correctness: this variant
    /// loses increments, and the point of the drill is that it does. Measured
    /// ~1.43–1.56M of 2,000,000 on aarch64 (M-series).
    #[test]
    fn acq_rel_loses_increments() {
        let got = stress::<DekkerAcqRel>();
        let lost = 2 * ITERS - got;
        println!(
            "DekkerAcqRel: {got} / {} — {lost} lost ({:.1}%)",
            2 * ITERS,
            100.0 * lost as f64 / (2 * ITERS) as f64
        );
        assert!(
            got <= 2 * ITERS,
            "counter cannot exceed the number of increments performed"
        );
    }

    #[test]
    fn fence_preserves_every_increment() {
        assert_eq!(stress::<DekkerFence>(), 2 * ITERS);
    }

    #[test]
    fn seq_cst_preserves_every_increment() {
        assert_eq!(stress::<DekkerSeqCst>(), 2 * ITERS);
    }
}
