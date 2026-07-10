use core::mem::MaybeUninit;

#[cfg(loom)]
use loom::cell::UnsafeCell;
#[cfg(loom)]
use loom::sync::atomic::{AtomicU8, Ordering};

#[cfg(not(loom))]
use core::cell::UnsafeCell;
#[cfg(not(loom))]
use core::sync::atomic::{AtomicU8, Ordering};

const EMPTY: u8 = 0;
const WRITING: u8 = 1;

const SET: u8 = 2;

pub struct AlreadySet;

pub struct OnceFlag<T> {
    state: AtomicU8,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> OnceFlag<T> {
    pub fn new() -> Self {
        OnceFlag {
            state: AtomicU8::new(0),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
    pub fn set(&self, value: T) -> Result<(), AlreadySet> {
        if self
            .state
            .compare_exchange(EMPTY, WRITING, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            return Err(AlreadySet);
        }

        //SAFETY: The one who can flip the state is the single write here.
        unsafe {
            self.write_value(value);
        }
        self.state.store(SET, Ordering::Release);
        Ok(())
    }

    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) == SET {
            return Some(unsafe { self.read_value() });
        }

        None
    }

    unsafe fn write_value(&self, v: T) {
        #[cfg(not(loom))]
        unsafe {
            (*self.value.get()).write(v);
        }
        #[cfg(loom)]
        self.value.with_mut(|p| unsafe {
            (*p).write(v);
        });
    }
    unsafe fn read_value(&self) -> &T {
        #[cfg(not(loom))]
        {
            unsafe { (*self.value.get()).assume_init_ref() }
        }
        #[cfg(loom)]
        {
            self.value.with(|p| unsafe { (*p).assume_init_ref() })
        }
    }
}

unsafe impl<T: Send> Send for OnceFlag<T> {}
unsafe impl<T: Send + Sync> Sync for OnceFlag<T> {}
