//! Adaptive 3-stage backoff ladder for lock-free retry loops.
//!
//! Mirror of `crossbeam-utils::Backoff`. Per-thread (`!Sync` by virtue of `Cell<u32>`); each waiter
//! constructs its own. The ladder is `spin_loop` → `yield_now` → caller-parks; `Backoff` itself
//! never blocks — `is_completed()` is the signal for the caller to escalate to its own [[parker]].
//!
//! # Thresholds
//!
//! - `SPIN_LIMIT = 6` → burst caps at `1 << 6 = 64` spin-hint iterations.
//! - `YIELD_LIMIT = 10` → past this, `is_completed()` flips and the caller should park.
//!
//! These were derived from the empirical "transient vs structural contention" split: a transient
//! CAS race resolves in ~10-100 ns (well inside the burst budget on x86); a structural wait (lock
//! held >1 ms, queue empty under producer lag) needs the scheduler's help, which yield then park
//! deliver.
//!
//! ## The burst budget is NOT architecture-neutral (measured)
//!
//! `SPIN_LIMIT = 6` is inherited from `crossbeam-utils`, where it was tuned against x86's `pause`
//! (~1 ns). `core::hint::spin_loop()` lowers per target, and on aarch64 it emits `isb SY` — a
//! pipeline flush, measured at **12.2 ns** on Apple Silicon (see
//! `notes/backoff_bench_results.md`). So the same constant buys very different budgets:
//!
//! | | per hint | burst at `SPIN_LIMIT` | full ladder before first yield |
//! |---|---|---|---|
//! | x86_64 (`pause`) | ~1 ns | ~64 ns | ~127 ns |
//! | aarch64 (`isb`) | **12.2 ns** | **784 ns** | **~1.55 µs** |
//!
//! The wasted-CPU side of this is defensible: `yield_now()` measured **4.6 µs**, so spinning up to
//! ~1.55 µs before paying for a yield is a sane ~1:3 ratio. The cost that is *not* obvious is
//! **check granularity**: at the capped burst the caller only re-tests its condition every 784 ns
//! on aarch64 versus every ~64 ns on x86. A wait satisfied 10 ns into the final burst goes
//! unnoticed for the remaining ~774 ns.
//!
//! That is harmless for waiters whose condition resolves in tens of ns (they finish at step 0-2,
//! ~85 ns, and never reach the large bursts). It is NOT harmless for a latency-sensitive waiter
//! whose condition resolves somewhere in the 100 ns - 1 µs band: it can eat most of a microsecond
//! of pure added latency. Such a waiter needs a per-target `SPIN_LIMIT` (≈2-3 on aarch64 to match
//! x86's burst in nanoseconds, not in iteration count), not this shared constant.
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
//!    cost when the holder is co-located, and the yield step should be gated on a NUMA-distance
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
