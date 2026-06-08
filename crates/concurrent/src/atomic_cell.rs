//! Lock-free cell for `Copy` types up to 8 bytes (fast path) with spinlock
//! fallback for everything else.
//!
//! # Fast-path safety
//!
//! `T` must be a no-padding type when it would hit the fast path
//! (`size_of::<T>() == 8 && align_of::<T>() == 8`). For example, `u64`,
//! `i64`, `*const X` are fine; `#[repr(C, align(8))] struct S { a: u32,
//! b: u16 }` passes the gate but has 2 bytes of padding — reading those
//! bytes through `AtomicU64` is UB. A future revision should require
//! `T: bytemuck::Pod` to make padding-containing T a compile error.
mod sync {
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    #[cfg(not(loom))]
    pub(super) use core::cell::UnsafeCell;

    // Loom build: AtomicU64 is unused (the fast path is compiled out), so it is
    // not re-exported here.
    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{AtomicBool, Ordering};

    #[cfg(loom)]
    pub(super) use loom::cell::UnsafeCell;

    // Spin hint. Under loom a bare `spin_loop` makes the model "exceed maximum
    // branches": loom has no progress guarantee for a spinning thread, so it
    // re-runs the spinner forever and never schedules the lock holder's release.
    // `yield_now` hands control back to loom's scheduler so it runs the holder.
    #[cfg(not(loom))]
    pub(super) fn spin_hint() {
        core::hint::spin_loop();
    }
    #[cfg(loom)]
    pub(super) fn spin_hint() {
        loom::thread::yield_now();
    }
}

#[cfg(not(loom))]
use core::{mem, ptr};

use sync::{spin_hint, AtomicBool, Ordering, UnsafeCell};

#[cfg(not(loom))]
use sync::AtomicU64;

pub struct AtomicCell<T: Copy> {
    value: UnsafeCell<T>,
    lock: AtomicBool,
}

impl<T: Copy> AtomicCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            lock: AtomicBool::new(false),
        }
    }

    pub fn store(&self, val: T) {
        // Fast path is compiled out under loom: loom's `UnsafeCell` has no raw
        // `.get()` pointer, and loom can't model the `AtomicU64` transmute trick
        // anyway, so everything routes through the spinlock slow path there.
        #[cfg(not(loom))]
        if const { size_of::<T>() == 8 && align_of::<T>() == 8 } {
            //SAFETY: Already checked for size and alignment of 8 bytes
            let atomic: &AtomicU64 = unsafe { &*(self.value.get() as *const AtomicU64) };

            //SAFETY: Already checked for size and alignment of 8 bytes
            let bits = unsafe { mem::transmute_copy::<T, u64>(&val) };
            atomic.store(bits, Ordering::Release);
            return;
        }

        loop {
            if self
                .lock
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
            spin_hint();
        }

        //SAFETY: Only one thread holds the lock, so this write is exclusive.
        #[cfg(not(loom))]
        unsafe {
            *self.value.get() = val;
        }
        #[cfg(loom)]
        self.value.with_mut(|p| unsafe { *p = val });

        self.lock.store(false, Ordering::Release);
    }

    pub fn load(&self) -> T {
        // See `store`: the fast path is compiled out under loom.
        #[cfg(not(loom))]
        if const { size_of::<T>() == 8 && align_of::<T>() == 8 } {
            // SAFETY: gated on size_of::<T>() == 8 && align_of::<T>() == 8, so the
            // UnsafeCell<T> storage is layout-compatible with AtomicU64.
            let atomic: &AtomicU64 = unsafe { &*(self.value.get() as *const AtomicU64) };
            let bits = atomic.load(Ordering::Acquire);

            // SAFETY: gated on size_of::<T>() == 8, so copying 8 bytes from &val
            // stays in-bounds and produces a fully-initialized u64.
            return unsafe { ptr::read(&bits as *const _ as *const T) };
        }

        loop {
            if self
                .lock
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
            spin_hint();
        }

        //SAFETY: Only one thread holds the lock, so this read is exclusive.
        #[cfg(not(loom))]
        let val = unsafe { *self.value.get() };
        #[cfg(loom)]
        let val = self.value.with(|p| unsafe { *p });

        self.lock.store(false, Ordering::Release);
        val
    }
}

unsafe impl<T: Copy + Send> Sync for AtomicCell<T> {}

#[cfg(test)]
mod test {
    use crate::atomic_cell::AtomicCell;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn assert_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<AtomicCell<u64>>();
        assert_sync::<AtomicCell<[u8; 16]>>();
    }

    #[test]
    fn concurrent_store_load_does_not_tear() {
        const ITERATIONS: u32 = 10_000;
        let cell = Arc::new(AtomicCell::new(0u32));
        let writer = {
            let cell = Arc::clone(&cell);
            thread::spawn(move || {
                for i in 0..ITERATIONS {
                    cell.store(i);
                }
            })
        };

        for _ in 0..ITERATIONS {
            let v = cell.load();
            assert!(v < ITERATIONS, "observed torn or invalid value: {v}");
        }

        writer.join().unwrap();
    }

    #[test]
    fn fast_path_u64_round_trip() {
        let cell = AtomicCell::new(0u64);
        cell.store(42);
        assert_eq!(cell.load(), 42);

        cell.store(u64::MAX);
        assert_eq!(cell.load(), u64::MAX);

        cell.store(0);
        assert_eq!(cell.load(), 0);
    }

    #[test]
    fn slow_path_array_round_trip() {
        let cell = AtomicCell::new([0u8; 16]);
        cell.store([7u8; 16]);
        assert_eq!(cell.load(), [7u8; 16]);

        // Distinct bytes — would catch a wrong-offset copy or partial write that
        // an all-same-byte test would silently pass.
        let pattern = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        cell.store(pattern);
        assert_eq!(cell.load(), pattern);
    }

    // Slow-path tearing detector: writer cycles values where all 16 bytes are equal.
    // Any torn read (some bytes from value N, others from N±1) shows mixed bytes,
    // which the all-equal assertion catches. Without the spinlock holding load+store
    // exclusive, this test would fail on the first contended read.
    #[test]
    fn slow_path_no_tearing_under_contention() {
        const WRITER_ITERATIONS: u8 = 255;
        const READER_ITERATIONS: u32 = 50_000;

        let cell = Arc::new(AtomicCell::new([0u8; 16]));
        let writer = {
            let cell = Arc::clone(&cell);
            thread::spawn(move || {
                for _ in 0..100 {
                    for i in 0..WRITER_ITERATIONS {
                        cell.store([i; 16]);
                    }
                }
            })
        };

        for _ in 0..READER_ITERATIONS {
            let v = cell.load();
            let first = v[0];
            assert!(v.iter().all(|&b| b == first), "torn read observed: {v:?}");
        }

        writer.join().unwrap();
    }
}
