use crate::Pod;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

mod sync {
    #[cfg(not(loom))]
    pub(super) use core::hint::spin_loop;
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{fence, AtomicUsize, Ordering};

    #[cfg(loom)]
    pub(super) use loom::hint::spin_loop;
    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{fence, AtomicUsize, Ordering};
}

use sync::{fence, spin_loop, AtomicUsize, Ordering};

pub struct SeqLock<T> {
    /// The payload.
    ///
    /// Outside loom this is the real thing: one `T`, written and read with
    /// word-sized relaxed atomic ops through a raw pointer.
    ///
    /// Under loom it is replaced by one `AtomicUsize` per word. Loom only
    /// models operations that go through *its* atomic types, so a payload
    /// living behind a plain `UnsafeCell` would be invisible to the model and
    /// the racy read the seqlock deliberately allows would never be explored.
    #[cfg(not(loom))]
    data: UnsafeCell<T>,
    #[cfg(loom)]
    data: Box<[AtomicUsize]>,
    #[cfg(loom)]
    _marker: core::marker::PhantomData<UnsafeCell<T>>,

    state: AtomicUsize,
}

unsafe impl<T: Send> Sync for SeqLock<T> {}

impl<T: Pod> SeqLock<T> {
    pub fn new(data: T) -> Self {
        Self::require();

        #[cfg(not(loom))]
        {
            SeqLock {
                data: UnsafeCell::new(data),
                state: AtomicUsize::new(0),
            }
        }

        #[cfg(loom)]
        {
            let src = &data as *const T as *const usize;
            let cells = (0..Self::WORDS)
                .map(|i| AtomicUsize::new(unsafe { src.add(i).read() }))
                .collect();
            SeqLock {
                data: cells,
                _marker: core::marker::PhantomData,
                state: AtomicUsize::new(0),
            }
        }
    }

    const WORDS: usize = size_of::<T>() / size_of::<usize>();

    fn require() {
        const {
            assert!(size_of::<T>() % size_of::<usize>() == 0);
            assert!(align_of::<T>() >= align_of::<usize>());
        }
    }

    /// The `i`-th payload word, as an atomic.
    ///
    /// Outside loom the cast is sound because `AtomicUsize` has the same size,
    /// alignment and layout as `usize`, and `require()` has already checked
    /// that `T` is a whole number of correctly aligned words.
    #[inline]
    fn word(&self, i: usize) -> &AtomicUsize {
        debug_assert!(i < Self::WORDS);

        #[cfg(not(loom))]
        {
            unsafe { &*(self.data.get() as *const AtomicUsize).add(i) }
        }

        #[cfg(loom)]
        {
            &self.data[i]
        }
    }

    pub fn store(&self, value: T) {
        Self::require();

        // Claim the lock: bump an even sequence to odd.
        let mut cur = self.state.load(Ordering::Relaxed);
        loop {
            if cur % 2 == 1 {
                spin_loop();
                cur = self.state.load(Ordering::Relaxed);
                continue;
            }
            match self.state.compare_exchange_weak(
                cur,
                cur + 1,
                // Acquire (read half of the RMW): claiming the slot must inherit
                // the previous writer's payload writes, published by its
                // `fetch_add(Release)`. Relaxed here reads the seq number but not
                // the payload, and two writers' words tear permanently (loom
                // `two_writers_serialise`). Failure ordering stays Relaxed: a lost
                // race inherits nothing, it just retries.
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(now) => cur = now,
            }
        }

        // Floor: nothing below may sink past the odd bump.
        fence(Ordering::Release);

        let src = &value as *const T as *const usize;
        for i in 0..Self::WORDS {
            let word = unsafe { src.add(i).read() };
            self.word(i).store(word, Ordering::Relaxed);
        }

        self.state.fetch_add(1, Ordering::Release);
    }

    pub fn load(&self) -> T {
        Self::require();

        loop {
            let first = self.state.load(Ordering::Acquire);
            if first % 2 == 1 {
                spin_loop();
                continue;
            }

            let mut out = MaybeUninit::<T>::uninit();
            let dst = out.as_mut_ptr() as *mut usize;
            for i in 0..Self::WORDS {
                let word = self.word(i).load(Ordering::Relaxed);
                unsafe { dst.add(i).write(word) };
            }

            // Roof: the payload reads may not float past the second seq read.
            fence(Ordering::Acquire);
            let second = self.state.load(Ordering::Relaxed);
            if first == second {
                return unsafe { out.assume_init() };
            }
            spin_loop();
        }
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use crate::arc::Arc;
    use crate::seq_lock::SeqLock;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    #[test]
    fn store_load() {
        let lock = SeqLock::new(10u64);
        assert_eq!(lock.load(), 10);
        lock.store(20);
        assert_eq!(lock.load(), 20);
    }

    #[test]
    fn store_load_multi_thread() {
        const WRITES: u64 = 200_000;
        const READERS: usize = 3;

        let lock = Arc::new(SeqLock::new([0u64; 4]));
        let done = Arc::new(AtomicBool::new(false));

        let mut handles = Vec::new();

        for _ in 0..READERS {
            let lock = lock.clone();
            let done = done.clone();
            handles.push(thread::spawn(move || {
                let mut reads = 0u64;
                while !done.load(Ordering::Relaxed) {
                    let v = lock.load();
                    assert!(
                        v[0] == v[1] && v[1] == v[2] && v[2] == v[3],
                        "torn read after {reads} reads: {v:?}"
                    );
                    reads += 1;
                }
            }));
        }

        let writer = {
            let lock = lock.clone();
            let done = done.clone();
            thread::spawn(move || {
                for i in 1..=WRITES {
                    lock.store([i; 4]);
                }
                done.store(true, Ordering::Relaxed);
            })
        };

        writer.join().unwrap();
        for h in handles {
            h.join().unwrap();
        }
    }
}
