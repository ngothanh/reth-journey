use atomic_wait::{wait, wake_one};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering};

pub struct Mutex<T> {
    state: AtomicU32,
    data: UnsafeCell<T>,
}

const UNLOCK: u32 = 0;
const LOCK_NO_WAIT: u32 = 1;

const LOCK_WAITING: u32 = 2;

unsafe impl<T> Sync for Mutex<T> where T: Send {}

pub struct MutexGuard<'a, T> {
    pub mutex: &'a Mutex<T>,
}

unsafe impl<T> Send for MutexGuard<'_, T> where T: Send {}
unsafe impl<T> Sync for MutexGuard<'_, T> where T: Sync {}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        if self.mutex.state.swap(UNLOCK, Ordering::Release) == LOCK_WAITING {
            wake_one(&self.mutex.state);
        }
    }
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Mutex {
            state: AtomicU32::new(UNLOCK),
            data: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        if self
            .state
            .compare_exchange(UNLOCK, LOCK_NO_WAIT, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.state.swap(LOCK_WAITING, Ordering::Acquire) != 0 {
                wait(&self.state, LOCK_WAITING);
            }
        }

        MutexGuard { mutex: self }
    }
}
