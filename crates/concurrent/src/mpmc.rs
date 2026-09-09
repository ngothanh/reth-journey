use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

pub struct MpmcRing<T> {
    data: Box<[UnsafeCell<MaybeUninit<T>>]>,
    head: AtomicUsize,
    tail: AtomicUsize,
    mask: usize,
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
                .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
                .collect(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            mask: capacity - 1,
        }
    }

    pub fn try_push(&self, value: T) -> Result<(), T> {
        let writer = self.tail.load(Relaxed);
        let reader = self.head.load(Relaxed);
        if writer - reader == self.capacity() {
            return Err(value);
        }

        let reserved = writer & self.mask;
        match self
            .tail
            .compare_exchange(writer, writer + 1, Relaxed, Relaxed)
        {
            Ok(_) => unsafe {
                self.data[reserved].get().write(MaybeUninit::new(value));
                Ok(())
            },
            Err(_) => Err(value),
        }
    }

    pub fn try_pop(&self) -> Option<T> {
        let writer = self.tail.load(Relaxed);
        let reader = self.head.load(Relaxed);
        if writer == reader {
            return None;
        }

        let slot = reader & self.mask;
        match self
            .head
            .compare_exchange(reader, reader + 1, Relaxed, Relaxed)
        {
            Ok(_) => unsafe { Some((*self.data[slot].get()).assume_init_read()) },
            Err(_) => None,
        }
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
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
