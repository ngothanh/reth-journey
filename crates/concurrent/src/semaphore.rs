use std::sync::atomic::Ordering::{Relaxed, Release};
use Ordering::Acquire;

mod sync {
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{AtomicU32, Ordering};

    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{AtomicU32, Ordering};
}

use sync::{AtomicU32, Ordering};

#[cfg(not(loom))]
use atomic_wait::{wait as futex_wait, wake_all as futex_wake_all, wake_one as futex_wake_one};
#[cfg(loom)]
fn futex_wait(atomic: &AtomicU32, expected: u32) {
    if atomic.load(Relaxed) == expected {
        loom::thread::yield_now();
    }
}
#[cfg(loom)]
fn futex_wake_one(_atomic: &AtomicU32) {}
#[cfg(loom)]
fn futex_wake_all(_atomic: &AtomicU32) {}

pub struct Semaphore {
    count: AtomicU32,
    waiters: AtomicU32,
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
                self.waiters.fetch_add(1, Ordering::Release);
                futex_wait(&self.count, 0);
                self.waiters.fetch_sub(1, Ordering::Release);
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

        let waiters = self.waiters.load(Relaxed);
        if waiters == 0 {
            return;
        }
        if num == 1 {
            futex_wake_one(&self.count);
        } else {
            futex_wake_all(&self.count);
        }
    }
}
