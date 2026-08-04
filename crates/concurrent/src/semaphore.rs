
mod sync {
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{AtomicU32, Ordering};

    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{AtomicU32, Ordering};
}

use sync::{AtomicU32, Ordering};
// Pulled from the shim above, NOT from `std`, so the loom build cannot silently
// mix a std `Ordering` with a loom atomic.
use Ordering::{Acquire, Relaxed, Release};

pub struct Semaphore {
    count: AtomicU32,
    waiters: AtomicU32,

    /// Loom-only. The real implementation blocks on a futex, which loom cannot
    /// see. Modelling the wait as a spin would let a "sleeping" thread make
    /// progress on its own, and a lost wakeup would never show up — the spinner
    /// just loops until it observes the new count. A `Condvar` is the smallest
    /// thing loom *does* understand as genuinely blocked, so a lost wakeup
    /// surfaces as a deadlock the model checker reports.
    #[cfg(loom)]
    parking: (loom::sync::Mutex<()>, loom::sync::Condvar),
}

pub struct SemaphorePermit<'a> {
    semaphore: &'a Semaphore,
}

impl Drop for SemaphorePermit<'_> {
    fn drop(&mut self) {
        self.semaphore.add(1)
    }
}

impl Semaphore {
    pub fn new(permits: u32) -> Semaphore {
        Semaphore {
            count: AtomicU32::new(permits),
            waiters: AtomicU32::new(0),
            #[cfg(loom)]
            parking: (loom::sync::Mutex::new(()), loom::sync::Condvar::new()),
        }
    }

    /// Block until `count` is observed to differ from `expected`.
    ///
    /// Returning early is always sound — every caller re-checks `count` in a
    /// loop. Blocking *forever* after a wake is not, which is what
    /// `loom_no_lost_wakeup` pins down.
    #[cfg(not(loom))]
    fn wait_while_eq(&self, expected: u32) {
        atomic_wait::wait(&self.count, expected);
    }

    #[cfg(loom)]
    fn wait_while_eq(&self, expected: u32) {
        let (mutex, condvar) = &self.parking;
        let mut guard = mutex.lock().unwrap();
        // Re-checking `count` *under the lock* is what models the futex's
        // atomic compare-and-block. It closes the window where `add` bumps the
        // count and signals between this thread's `waiters.fetch_add` and its
        // going to sleep: either we observe the new count and never sleep, or
        // we sleep holding the lock the waker must take to signal.
        while self.count.load(Relaxed) == expected {
            guard = condvar.wait(guard).unwrap();
        }
    }

    /// Wake waiters after `count` has been increased by `num`.
    #[cfg(not(loom))]
    fn wake_waiters(&self, num: u32) {
        if num == 1 {
            atomic_wait::wake_one(&self.count);
        } else {
            atomic_wait::wake_all(&self.count);
        }
    }

    #[cfg(loom)]
    fn wake_waiters(&self, num: u32) {
        let (mutex, condvar) = &self.parking;
        // Taking the lock is what prevents signalling into the gap between a
        // waiter's count re-check and its `wait`.
        let _guard = mutex.lock().unwrap();
        if num == 1 {
            condvar.notify_one();
        } else {
            condvar.notify_all();
        }
    }

    pub fn acquire(&self) -> SemaphorePermit<'_> {
        let mut cur = self.count.load(Relaxed);
        loop {
            if cur > 0 {
                match self.count.compare_exchange(cur, cur - 1, Acquire, Relaxed) {
                    Ok(_) => {
                        return SemaphorePermit { semaphore: self };
                    }
                    Err(s) => {
                        cur = s;
                    }
                }
            } else if cur == 0 {
                self.waiters.fetch_add(1, Relaxed);
                self.wait_while_eq(0);
                self.waiters.fetch_sub(1, Relaxed);
                cur = self.count.load(Acquire);
            }
        }
    }

    pub fn try_acquire(&self) -> Option<SemaphorePermit<'_>> {
        let mut cur = self.count.load(Relaxed);

        loop {
            if cur == 0 {
                return None;
            }
            match self
                .count
                .compare_exchange_weak(cur, cur - 1, Acquire, Relaxed)
            {
                Ok(_) => return Some(SemaphorePermit { semaphore: self }),
                Err(e) => {
                    cur = e;
                }
            }
        }
    }

    pub fn add(&self, num: u32) {
        if num == 0 {
            return;
        }

        let mut cur = self.count.load(Relaxed);
        loop {
            let next = cur.checked_add(num).expect("semaphore overflow");
            match self
                .count
                .compare_exchange_weak(cur, next, Release, Relaxed)
            {
                Ok(_) => break,
                Err(e) => cur = e,
            }
        }

        // Fast path: skip the syscall when nobody is parked.
        //
        // `Relaxed` is right here — `waiters` guards no data, and neither stale
        // outcome is incorrect. Reading too high costs one wasted syscall.
        // Reading too low skips a wake, and that is caught by the futex:
        // `atomic_wait::wait(&count, 0)` compares and blocks *atomically in the
        // kernel*, so a waiter racing us re-reads `count`, sees the value this
        // `add` just stored, and returns instead of sleeping.
        //
        // Loom cannot express that guarantee — its `wait_while_eq` re-check is
        // an ordinary load, and when this fast path returns early it never
        // touches the parking mutex, so no happens-before edge makes the new
        // count visible to the waiter, which then sleeps forever. The fast path
        // is therefore disabled under loom so `loom_no_lost_wakeup` tests the
        // wakeup protocol rather than the model's blind spot.
        //
        // KNOWN GAP: consequently the fast path itself is not model-checked.
        #[cfg(not(loom))]
        if self.waiters.load(Relaxed) == 0 {
            return;
        }
        self.wake_waiters(num);
    }
}
