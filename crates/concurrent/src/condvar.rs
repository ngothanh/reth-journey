use crate::MutexGuard;
use atomic_wait::{wait, wake_all, wake_one};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

pub struct Condvar {
    counter: AtomicU32,
    num_waiters: AtomicUsize,
}

impl Condvar {
    fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
            num_waiters: AtomicUsize::new(0),
        }
    }

    fn notify_one(&self) {
        if self.num_waiters.load(Ordering::Relaxed) > 0 {
            self.counter.fetch_add(1, Ordering::Relaxed);
            wake_one(&self.counter);
        }
    }

    fn notify_all(&self) {
        if self.num_waiters.load(Ordering::Relaxed) > 0 {
            self.counter.fetch_add(1, Ordering::Relaxed);
            wake_all(&self.counter);
        }
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        self.num_waiters.fetch_add(1, Ordering::Relaxed);
        let initial_value = self.counter.load(Ordering::Relaxed);
        let mutex = guard.mutex;
        drop(guard);
        wait(&self.counter, initial_value);
        self.num_waiters.fetch_sub(1, Ordering::Relaxed);
        mutex.lock()
    }
}

#[cfg(test)]
mod tests {
    use crate::condvar::Condvar;
    use crate::Mutex;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_condvar() {
        let mutex = Mutex::new(0u32);
        let condvar = Condvar::new();
        let mut wakes = 0;
        thread::scope(|s| {
            s.spawn(|| {
                thread::sleep(Duration::from_secs(1));
                *mutex.lock() = 123;
                condvar.notify_one();
            });

            let mut m = mutex.lock();
            while *m < 100 {
                m = condvar.wait(m);
                wakes += 1;
            }

            assert_eq!(*m, 123);
        });

        assert!(wakes < 10);
    }
}
