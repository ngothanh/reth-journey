use core::mem::MaybeUninit;

mod sync {
    #[cfg(not(loom))]
    pub(super) use core::cell::UnsafeCell;
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{AtomicUsize, Ordering};

    #[cfg(loom)]
    pub(super) use loom::cell::UnsafeCell;
    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{AtomicUsize, Ordering};
}

use crate::CachePadded;
use sync::{AtomicUsize, Ordering, UnsafeCell};

pub struct MpmcRing<T> {
    data: Box<[Cell<T>]>,
    head: CachePadded<AtomicUsize>,
    tail: CachePadded<AtomicUsize>,
    mask: usize,
}

struct Cell<T> {
    seq: AtomicUsize,
    payload: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send> Send for MpmcRing<T> {}

unsafe impl<T: Send> Sync for MpmcRing<T> {}

impl<T> MpmcRing<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be non-zero");
        assert!(
            capacity.is_power_of_two(),
            "capacity must be a power of two"
        );
        Self {
            data: (0..capacity)
                .map(|i| Cell {
                    seq: AtomicUsize::new(i),
                    payload: UnsafeCell::new(MaybeUninit::uninit()),
                })
                .collect(),
            head: CachePadded::new(AtomicUsize::new(0)),
            tail: CachePadded::new(AtomicUsize::new(0)),
            mask: capacity - 1,
        }
    }

    pub fn try_push(&self, value: T) -> Result<(), T> {
        let mut pos = self.tail.load(Ordering::Relaxed);
        loop {
            let i = pos & self.mask;
            let seq = self.data[i].seq.load(Ordering::Acquire);
            let diff = seq.wrapping_sub(pos) as isize;
            if diff > 0 {
                pos = self.tail.load(Ordering::Relaxed);
                continue;
            }
            if diff < 0 {
                return Err(value);
            }

            match self.tail.compare_exchange_weak(
                pos,
                pos + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    #[cfg(not(loom))]
                    unsafe {
                        self.data[i].payload.get().write(MaybeUninit::new(value));
                    }
                    #[cfg(loom)]
                    self.data[i]
                        .payload
                        .with_mut(|p| unsafe { *p = MaybeUninit::new(value) });

                    self.data[i].seq.store(pos + 1, Ordering::Release);
                    return Ok(());
                }
                Err(cur) => pos = cur,
            }
        }
    }

    pub fn try_pop(&self) -> Option<T> {
        let mut pos = self.head.load(Ordering::Relaxed);

        loop {
            let i = pos & self.mask;
            let seq = self.data[i].seq.load(Ordering::Acquire);
            let diff = seq.wrapping_sub(pos) as isize - 1;
            if diff > 0 {
                pos = self.head.load(Ordering::Relaxed);
                continue;
            }
            if diff < 0 {
                return None;
            }

            match self.head.compare_exchange_weak(
                pos,
                pos + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => unsafe {
                    #[cfg(not(loom))]
                    let res = Some((*self.data[i].payload.get()).assume_init_read());

                    #[cfg(loom)]
                    let res = Some(
                        self.data[i]
                            .payload
                            .with(|p| unsafe { (*p).assume_init_read() }),
                    );

                    self.data[i]
                        .seq
                        .store(pos + self.capacity(), Ordering::Release);
                    return res;
                },
                Err(cur) => pos = cur,
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }
}

impl<T> Drop for MpmcRing<T> {
    fn drop(&mut self) {
        while let Some(x) = self.try_pop() {
            drop(x);
        }
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use crate::mpmc::MpmcRing;

    #[test]
    fn no_contention() {
        let ring: MpmcRing<usize> = MpmcRing::with_capacity(2);

        assert_eq!(ring.capacity(), 2);
        assert!(ring.try_pop().is_none());

        assert!(ring.try_push(1).is_ok());
        assert!(ring.try_push(2).is_ok());
        assert!(ring.try_push(3).is_err());

        assert_eq!(ring.try_pop().unwrap(), 1);
        assert_eq!(ring.try_pop().unwrap(), 2);
    }

    #[test]
    #[should_panic(expected = "power of two")]
    fn panics_on_non_power_of_two() {
        let _ = MpmcRing::<usize>::with_capacity(3);
    }

    #[test]
    #[should_panic(expected = "non-zero")]
    fn panics_on_zero_capacity() {
        let _ = MpmcRing::<usize>::with_capacity(0);
    }

    #[test]
    fn accepts_power_of_two_capacities() {
        for cap in [1usize, 2, 4, 8, 1024] {
            let ring = MpmcRing::<usize>::with_capacity(cap);
            assert_eq!(ring.capacity(), cap);
        }
    }
}
