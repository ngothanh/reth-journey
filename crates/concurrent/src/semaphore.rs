use atomic_wait::wait;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct Semaphore {
    count: AtomicU32,
}

pub struct SemaphorePermit<'a> {
    semaphore: &'a Semaphore,
}

impl Semaphore {
    pub fn new(permits: u32) -> Semaphore {
        Semaphore {
            count: AtomicU32::new(permits),
        }
    }

    pub fn acquire(&self) -> SemaphorePermit {
        let mut cur = self.count.load(Ordering::Relaxed);
        loop {
            if cur > 0 {
                match self.count.compare_exchange(
                    cur,
                    cur - 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        return SemaphorePermit { semaphore: self };
                    }
                    Err(s) => {
                        cur = s;
                    }
                }
            } else if cur == 0 {
                wait(&self.count, 0);
                cur = self.count.load(Ordering::Acquire);
            }
        }
    }

    pub fn try_acquire(&self) -> Option<SemaphorePermit> {
        let mut cur = self.count.load(Ordering::Relaxed);

        loop {
            if cur == 0 {
                return None;
            }
            match self.count.compare_exchange_weak(
                cur,
                cur - 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(SemaphorePermit { semaphore: self }),
                Err(e) => {
                    cur = e;
                }
            }
        }
    }

    pub fn release(&self, num: usize) {
        todo!()
    }
}
