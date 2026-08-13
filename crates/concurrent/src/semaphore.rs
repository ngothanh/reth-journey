use crate::Mutex;
use core::future::Future;
use core::pin::Pin;
use core::task::Waker;
use core::task::{Context, Poll};
use slab::Slab;

pub struct Semaphore {
    state: Mutex<State>,
}

struct Waiter {
    waker: Waker,
}

struct State {
    waiters: Slab<Waiter>,
    permits: usize,
}

pub struct SemaphorePermit<'a> {
    semaphore: &'a Semaphore,
}

impl Drop for SemaphorePermit<'_> {
    fn drop(&mut self) {
        self.semaphore.add_permits(1);
    }
}

pub struct Acquire<'a> {
    semaphore: &'a Semaphore,
    key: Option<usize>,
}

impl Drop for Acquire<'_> {
    fn drop(&mut self) {
        let mut state = self.semaphore.state.lock();
        if let Some(k) = self.key {
            state.waiters.remove(k);
        }
    }
}

impl<'a> Future for Acquire<'a> {
    type Output = Result<SemaphorePermit<'a>, AcquireError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let mut state = this.semaphore.state.lock();

        if state.permits > 0 {
            state.permits -= 1;
            if let Some(k) = this.key {
                this.key = None;
                state.waiters.remove(k);
            }
            return Poll::Ready(Ok(SemaphorePermit {
                semaphore: this.semaphore,
            }));
        }

        match this.key {
            None => {
                let i = state.waiters.insert(Waiter {
                    waker: cx.waker().clone(),
                });
                this.key = Some(i);
            }
            Some(k) => {
                if !state.waiters[k].waker.will_wake(cx.waker()) {
                    state.waiters[k].waker = cx.waker().clone();
                }
            }
        }

        Poll::Pending
    }
}

pub enum TryAcquireError {
    NoPermits,
    Closed,
}

#[derive(Debug)]
pub struct AcquireError {}

impl Semaphore {
    pub fn new(num_permits: usize) -> Self {
        Semaphore {
            state: Mutex::new(State {
                waiters: Slab::new(),
                permits: num_permits,
            }),
        }
    }

    pub fn acquire(&self) -> Acquire<'_> {
        Acquire {
            semaphore: self,
            key: None,
        }
    }

    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, TryAcquireError> {
        let mut state = self.state.lock();
        if state.permits > 0 {
            state.permits -= 1;
            return Ok(SemaphorePermit { semaphore: self });
        }

        Err(TryAcquireError::NoPermits)
    }

    pub fn add_permits(&self, num_permits: usize) {
        let mut to_wake = Vec::new();
        {
            let mut state = self.state.lock();
            state.permits = state
                .permits
                .checked_add(num_permits)
                .expect("semaphore overflow");
            let mut count = 0;
            for (_, waiter) in state.waiters.iter() {
                if count == num_permits {
                    break;
                }
                to_wake.push(waiter.waker.clone());
                count += 1;
            }
        }
        for waker in to_wake {
            waker.wake()
        }
    }

    pub fn available_permits(&self) -> usize {
        self.state.lock().permits
    }

    #[cfg(test)]
    pub fn waiter_count(&self) -> usize {
        self.state.lock().waiters.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::Semaphore;
    use std::future::Future;
    use std::pin::pin;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    #[test]
    fn repoll_does_not_duplicate_waiter() {
        let semaphore = Semaphore::new(0);
        let waker = Waker::noop();
        let mut ctx = Context::from_waker(&waker);
        let mut fut = pin!(semaphore.acquire());

        let first_poll = fut.as_mut().poll(&mut ctx);
        assert!(matches!(first_poll, Poll::Pending));
        assert_eq!(semaphore.waiter_count(), 1, "1 waiter,first poll");

        let second_poll = fut.as_mut().poll(&mut ctx);
        assert!(matches!(second_poll, Poll::Pending));
        assert_eq!(semaphore.waiter_count(), 1, "1 waiter,second poll");
    }

    struct WatchedWaker(AtomicBool);

    impl WatchedWaker {
        fn new() -> Arc<Self> {
            Arc::new(WatchedWaker(AtomicBool::new(false)))
        }

        fn woken(&self) -> bool {
            self.0.load(Ordering::Relaxed)
        }
    }

    impl Wake for WatchedWaker {
        fn wake(self: Arc<Self>) {
            self.0.store(true, Ordering::Relaxed);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.store(true, Ordering::Relaxed);
        }
    }

    #[test]
    fn poll_refresh_waker() {
        // Given
        let semaphore = Semaphore::new(0);

        let w1 = WatchedWaker::new();
        let w2 = WatchedWaker::new();

        let waker1 = Waker::from(w1.clone());
        let waker2 = Waker::from(w2.clone());

        let mut ctx = Context::from_waker(&waker1);
        let mut ctx2 = Context::from_waker(&waker2);

        // When
        let mut fut = pin!(semaphore.acquire());

        // Then
        assert!(fut.as_mut().poll(&mut ctx).is_pending());
        assert!(fut.as_mut().poll(&mut ctx2).is_pending());

        semaphore.add_permits(1);

        assert!(!w1.woken());
        assert!(w2.woken());
    }
}
