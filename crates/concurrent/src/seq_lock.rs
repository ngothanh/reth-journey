use crate::Pod;
use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use std::mem::MaybeUninit;
use std::sync::atomic::fence;

pub struct SeqLock<T> {
    data: UnsafeCell<T>,
    state: AtomicUsize,
}

unsafe impl<T: Send> Sync for SeqLock<T> {}

impl<T: Pod> SeqLock<T> {
    pub fn new(data: T) -> Self {
        SeqLock {
            data: UnsafeCell::new(data),
            state: AtomicUsize::new(0),
        }
    }

    pub fn store(&self, value: T) {
        Self::require();
        let words = size_of::<T>() / size_of::<usize>();

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
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(now) => cur = now,
            }
        }

        fence(Ordering::Release);
        let dst = self.data.get() as *mut usize;
        let src = &value as *const T as *const usize;
        for i in 0..words {
            let word = unsafe { src.add(i).read() };
            let slot = unsafe { &*(dst.add(i) as *const AtomicUsize) };
            slot.store(word, Ordering::Relaxed);
        }
        self.state.fetch_add(1, Ordering::Release);
    }

    fn require() {
        const {
            assert!(size_of::<T>() % size_of::<usize>() == 0);
            assert!(align_of::<T>() >= align_of::<usize>());
        }
    }

    pub fn load(&self) -> T {
        Self::require();
        let words = size_of::<T>() / size_of::<usize>();
        loop {
            let first = self.state.load(Ordering::Acquire);
            if first % 2 == 1 {
                spin_loop();
                continue;
            }
            let mut out = MaybeUninit::<T>::uninit();
            let dst = out.as_mut_ptr() as *mut usize;
            let src = self.data.get() as *const usize;

            for i in 0..words {
                let slot = unsafe { &*(src.add(i) as *const AtomicUsize) };
                let word = slot.load(Ordering::Relaxed);
                unsafe {
                    dst.add(i).write(word);
                }
            }
            fence(Ordering::Acquire);
            let second = self.state.load(Ordering::Relaxed);
            if first == second {
                return unsafe { out.assume_init() };
            }
        }
    }
}

#[cfg(test)]
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
