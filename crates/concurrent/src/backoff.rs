//! Adaptive 3-stage backoff ladder for lock-free retry loops.
//!
//! Mirror of `crossbeam-utils::Backoff`. Per-thread (`!Sync` by virtue of `Cell<u32>`); each waiter
//! constructs its own. The ladder is `spin_loop` → `yield_now` → caller-parks; `Backoff` itself
//! never blocks — `is_completed()` is the signal for the caller to escalate to its own [[parker]].
//!
//! # Thresholds
//!
//! - `SPIN_LIMIT = 6` → burst caps at `1 << 6 = 64` PAUSE iterations (~64 ns on modern x86/aarch64).
//! - `YIELD_LIMIT = 10` → past this, `is_completed()` flips and the caller should park.
//!
//! These were derived from the empirical "transient vs structural contention" split: a transient
//! CAS race resolves in ~10-100 ns (well inside the 64-PAUSE budget); a structural wait (lock
//! held >1 ms, queue empty under producer lag) needs the scheduler's help, which yield then park
//! deliver.
//!
//! # 5-year failure mode
//!
//! The thresholds above assume **a multi-core SMT host where `spin_loop()` lowers to PAUSE/YIELD**
//! and the scheduler can re-route the holder to a different core within ~µs. Three shifts that
//! invalidate the ladder:
//!
//! 1. **Single-core deployment** (embedded, WASM, some container limits) — spin is *always* wrong:
//!    the holder cannot run while the waiter spins, so every retry is wasted CPU. `SPIN_LIMIT`
//!    should drop to 0; the ladder collapses to "yield immediately, then park."
//! 2. **256+ core machines with NUMA** — `yield_now()` may reschedule the waiter onto a far node,
//!    making the next CAS retry cost ~100 ns (cross-socket cache miss) instead of ~10 ns. The
//!    spin band should *widen* (SPIN_LIMIT≥8, burst up to 256 PAUSEs) to amortize the cross-socket
//!    cost when the holder is co-located, and the yield rung should be gated on a NUMA-distance
//!    hint to avoid the far-node trap.
//! 3. **RISC-V `Zihintpause` adoption** — if `spin_loop()` lowers to a no-op on a target without
//!    Zihintpause, every PAUSE in the burst is ~1 cycle of pure pipeline pressure with no SMT
//!    yield benefit. The burst would need to be tuned target-by-target via `cfg_target_feature`,
//!    same pattern as `CachePadded`'s 64/128-byte split.
use core::cell::Cell;
use core::hint;
use std::thread::yield_now;

pub struct Backoff {
    step: Cell<u32>,
}

const SPIN_LIMIT: u32 = 6;
const YIELD_LIMIT: u32 = 10;

impl Backoff {
    pub fn new() -> Self {
        Self { step: Cell::new(0) }
    }

    pub fn spin(&self) {
        for _ in 0..1 << self.step.get().min(SPIN_LIMIT) {
            hint::spin_loop();
        }
        if self.step.get() <= SPIN_LIMIT {
            self.step.set(self.step.get() + 1);
        }
    }

    pub fn snooze(&self) {
        let current_step = self.step.get();
        if self.step.get() <= SPIN_LIMIT {
            for _ in 0..1 << current_step {
                hint::spin_loop();
            }
        } else {
            yield_now();
        }
        if current_step <= YIELD_LIMIT {
            self.step.set(current_step + 1);
        }
    }

    pub fn is_completed(&self) -> bool {
        self.step.get() > YIELD_LIMIT
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_climbs_to_completion_within_eleven_calls() {
        let backoff = Backoff::new();
        assert!(!backoff.is_completed());

        for _ in 0..=YIELD_LIMIT {
            assert!(!backoff.is_completed());
            backoff.snooze();
        }

        assert!(backoff.is_completed());
    }

    #[test]
    fn spin_stays_in_spin_band() {
        let backoff = Backoff::new();

        for _ in 0..1000 {
            backoff.spin();
        }

        assert!(!backoff.is_completed());
    }
}
