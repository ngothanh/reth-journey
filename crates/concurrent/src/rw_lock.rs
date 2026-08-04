use core::ops::{Deref, DerefMut};

mod sync {
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{AtomicU32, Ordering};
    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{AtomicU32, Ordering};

    // std `UnsafeCell` (NOT `loom::cell::UnsafeCell`) even under loom: loom's cell
    // exposes data only through `with`/`with_mut` closures so it can bound the
    // access, which is incompatible with our `Deref`/`DerefMut` guards (they hand
    // back a `&T` that outlives any closure). So loom verifies the atomic locking
    // *protocol*; data-race checking on `T` is Miri's job.
    pub(super) use core::cell::UnsafeCell;
}

use sync::{AtomicU32, Ordering, UnsafeCell};
use Ordering::Acquire;
use Ordering::Relaxed;

// Futex shim: real `atomic_wait` syscalls in production, a loom-friendly model
// under `cfg(loom)`.
#[cfg(not(loom))]
use atomic_wait::{wait as futex_wait, wake_all as futex_wake_all, wake_one as futex_wake_one};

// loom can neither type-check against nor schedule a kernel futex, so model the
// block as a yield: the surrounding acquire loop re-checks the atomic, and loom
// explores the schedule where another thread changes it. The wakes become no-ops —
// a yielding thread is already runnable in loom's scheduler.
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

pub struct RwLock<T> {
    state: AtomicU32,
    writer_wake_count: AtomicU32,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for RwLock<T> where T: Send + Sync {}
unsafe impl<T> Send for RwLock<T> where T: Send {}

pub struct ReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

pub struct WriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

const UNLOCKED: u32 = 0;
const WRITER: u32 = u32::MAX;

impl<T> RwLock<T> {
    pub fn new(data: T) -> Self {
        RwLock {
            state: AtomicU32::new(UNLOCKED),
            writer_wake_count: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn read(&self) -> ReadGuard<'_, T> {
        let mut state = self.state.load(Relaxed);
        loop {
            if state % 2 == 0 {
                assert!(state < u32::MAX - 2, "too many readers");
                if self
                    .state
                    .compare_exchange_weak(state, state + 2, Acquire, Relaxed)
                    .is_ok()
                {
                    return ReadGuard { lock: self };
                }
            } else if state % 2 == 1 {
                futex_wait(&self.state, state);
            }
            state = self.state.load(Relaxed);
        }
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        let mut state = self.state.load(Relaxed);
        loop {
            if state <= 1 {
                match self.state.compare_exchange(state, WRITER, Acquire, Relaxed) {
                    Ok(_) => {
                        return WriteGuard { lock: self };
                    }
                    Err(e) => {
                        state = e;
                        continue;
                    }
                }
            } else if state % 2 == 0 {
                match self
                    .state
                    .compare_exchange(state, state + 1, Relaxed, Relaxed)
                {
                    Ok(_) => {}
                    Err(e) => {
                        state = e;
                        continue;
                    }
                }
            }
            let w = self.writer_wake_count.load(Acquire);
            state = self.state.load(Relaxed);
            if state >= 2 {
                futex_wait(&self.writer_wake_count, w);
                state = self.state.load(Relaxed);
            }
        }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        if self.lock.state.fetch_sub(2, Ordering::Release) == 3 {
            self.lock.writer_wake_count.fetch_add(1, Ordering::Release);
            futex_wake_one(&self.lock.writer_wake_count)
        };
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(UNLOCKED, Ordering::Release);
        self.lock.writer_wake_count.fetch_add(1, Ordering::Release);
        futex_wake_one(&self.lock.writer_wake_count);
        futex_wake_all(&self.lock.state);
    }
}
