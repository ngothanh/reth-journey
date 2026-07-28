use atomic_wait::{wait, wake_all, wake_one};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::Acquire;
use std::sync::atomic::{AtomicU32, Ordering};
use Ordering::Relaxed;

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

    pub fn read(&self) -> ReadGuard<T> {
        let mut state = self.state.load(Relaxed);
        loop {
            if state % 2 == 0 {
                assert!(state < u32::MAX - 2, "too many readers");
                if self
                    .state
                    .compare_exchange_weak(state, state + 2, Ordering::Acquire, Relaxed)
                    .is_ok()
                {
                    return ReadGuard { lock: self };
                }
            } else if state % 2 == 1 {
                wait(&self.state, state);
            }
            state = self.state.load(Relaxed);
        }
    }

    pub fn write(&self) -> WriteGuard<T> {
        let mut state = self.state.load(Relaxed);
        loop {
            if state <= 1 {
                match self
                    .state
                    .compare_exchange(state, WRITER, Acquire, Relaxed)
                {
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
                wait(&self.writer_wake_count, w);
                state = self.state.load(Relaxed);
            }
        }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        if self.lock.state.fetch_sub(2, Ordering::Release) == 3 {
            self.lock.writer_wake_count.fetch_add(1, Ordering::Release);
            wake_one(&self.lock.writer_wake_count)
        };
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(UNLOCKED, Ordering::Release);
        self.lock.writer_wake_count.fetch_add(1, Ordering::Release);
        wake_one(&self.lock.writer_wake_count);
        wake_all(&self.lock.state);
    }
}
