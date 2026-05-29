use std::sync::atomic::AtomicU32;
use std::sync::Arc;

mod sync {
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::Ordering;

    #[cfg(loom)]
    pub(super) use loom::sync::atomic::Ordering;
}
use sync::Ordering;

struct Inner {
    state: AtomicU32,
}

const EMPTY: u32 = 0;
const PARKED: u32 = 1;
const NOTIFIED: u32 = 2;

pub struct Parker {
    inner: Arc<Inner>,
}

#[derive(Clone)]
pub struct Unparker {
    inner: Arc<Inner>,
}

impl Parker {
    pub fn new() -> Self {
        let inner = Arc::new(Inner::new());
        Self { inner }
    }

    pub fn unparker(&self) -> Unparker {
        Unparker {
            inner: self.inner.clone(),
        }
    }

    pub fn park(&self) {
        match self
            .inner
            .state
            .compare_exchange(EMPTY, PARKED, Ordering::Acquire, Ordering::Relaxed)
        {
            Err(NOTIFIED) => {
                self.inner.state.store(EMPTY, Ordering::Release); // See NOTIFIED then no park, continue to work
            }
            Ok(_) => loop {
                atomic_wait::wait(&self.inner.state, PARKED);
                match self.inner.state.compare_exchange(
                    NOTIFIED,
                    EMPTY,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return,
                    Err(_) => continue,
                }
            },
            _ => unreachable!(),
        }
    }
}

impl Unparker {
    pub fn unpark(&self) {
        match self.inner.state.swap(NOTIFIED, Ordering::Acquire) {
            EMPTY => return,
            NOTIFIED => return,
            PARKED => atomic_wait::wake_one(&self.inner.state),
            _ => unreachable!(),
        }
    }
}

impl Inner {
    fn new() -> Self {
        Self {
            state: AtomicU32::new(EMPTY),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use crate::Parker;
    use std::time::{Duration, Instant};

    #[test]
    fn parker_smoke_unpark_then_park_returns_immediately() {
        let instant = Instant::now();
        let parker = Parker::new();
        let unparker = parker.unparker();

        unparker.unpark();
        parker.park();
        assert!(instant.elapsed().as_millis() < 100);
    }

    #[test]
    fn parker_smoke_park_then_unpark_unblocks() {
        let instant = Instant::now();
        let parker = Parker::new();
        let unparker = parker.unparker();

        parker.park();
        unparker.unpark();
        assert!(instant.elapsed().as_millis() < 10);
    }

    #[test]
    fn parker_park_actually_blocks_until_unpark() {
        let parker = Parker::new();
        let unparker = parker.unparker();

        let handle = thread::spawn(move || {
            let start = Instant::now();
            parker.park();
            start.elapsed()
        });
        thread::sleep(Duration::from_millis(50));
        unparker.unpark();
        let park_elapsed = handle.join().unwrap();
        assert!(park_elapsed >= Duration::from_millis(40),
                "park returned in {}ms — it didn't actually block", park_elapsed.as_millis());
        assert!(park_elapsed < Duration::from_millis(200), "park hung");
    }
}

#[cfg(loom)]
mod loom_tests {
    use crate::Parker;
    use std::thread;

    #[test]
    fn parker_loom_lost_wakeup_safe() {
        let parker = Parker::new();
        let unparker = parker.unparker();
        let t1 = thread::spawn(move || unparker.unpark());
        let t2 = thread::spawn(move || {
            parker.park();
        });
        t1.join().unwrap();
        t2.join().unwrap();
    }
}
