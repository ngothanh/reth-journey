use crate::{Mutex, MutexGuard, WaitList, Waiter};
use core::future::Future;
use core::pin::Pin;
use core::task::Waker;
use core::task::{Context, Poll};
use std::ptr::NonNull;

pub struct Semaphore {
    state: Mutex<State>,
}
unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

struct State {
    waiters: WaitList,
    permits: usize,
    closed: bool,
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
    node: Waiter,
    queued: bool,
}

unsafe impl Send for Acquire<'_> {}

impl Drop for Acquire<'_> {
    fn drop(&mut self) {
        let mut to_wake = Vec::new();
        {
            let mut state = self.semaphore.state.lock();
            if self.queued {
                if self.node.granted {
                    release_lock(state, 1, &mut to_wake);
                } else {
                    state.waiters.unlink(NonNull::from(&self.node));
                }
            }
        }

        for w in to_wake {
            w.wake()
        }
    }
}

impl<'a> Future for Acquire<'a> {
    type Output = Result<SemaphorePermit<'a>, AcquireError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let node = NonNull::from(&this.node);
        let mut state = this.semaphore.state.lock();
        match this.queued {
            true => {
                if this.node.granted {
                    this.queued = false;
                    Poll::Ready(Ok(SemaphorePermit {
                        semaphore: this.semaphore,
                    }))
                } else if state.closed {
                    this.queued = false;
                    state.waiters.unlink(node);
                    Poll::Ready(Err(AcquireError {}))
                } else {
                    this.node.update_waker(cx.waker());
                    Poll::Pending
                }
            }
            false => {
                if state.closed {
                    Poll::Ready(Err(AcquireError {}))
                } else {
                    if state.permits > 0 {
                        state.permits -= 1;
                        return Poll::Ready(Ok(SemaphorePermit {
                            semaphore: this.semaphore,
                        }));
                    }
                    this.node.update_waker(cx.waker());
                    state.waiters.push_back(node);
                    this.queued = true;
                    Poll::Pending
                }
            }
        }
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
                waiters: WaitList::new(),
                permits: num_permits,
                closed: false,
            }),
        }
    }

    pub fn close(&self) {
        let mut to_wake = Vec::new();
        {
            let mut state = self.state.lock();
            state.closed = true;
            state.waiters.take_all_wakers(&mut to_wake);
        }

        for w in to_wake {
            w.wake();
        }
    }

    pub fn acquire(&self) -> Acquire<'_> {
        Acquire {
            semaphore: self,
            node: Waiter::new(),
            queued: false,
        }
    }

    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, TryAcquireError> {
        let mut state = self.state.lock();
        if state.closed {
            return Err(TryAcquireError::Closed);
        }
        if state.permits > 0 {
            state.permits -= 1;
            return Ok(SemaphorePermit { semaphore: self });
        }

        Err(TryAcquireError::NoPermits)
    }

    pub fn add_permits(&self, num_permits: usize) {
        let mut to_wake = Vec::new();
        {
            let state = self.state.lock();
            release_lock(state, num_permits, &mut to_wake);
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

fn release_lock(mut state: MutexGuard<State>, num_permits: usize, to_wake: &mut Vec<Waker>) {
    let mut left = num_permits;
    while left > 0 {
        if let Some(w) = state.waiters.pop_front() {
            left -= 1;
            unsafe {
                (*w.as_ptr()).granted = true;
                if let Some(w) = (*w.as_ptr()).take_waker() {
                    to_wake.push(w);
                }
            }
        } else {
            break;
        }
    }
    state.permits = state.permits.checked_add(left).expect("semaphore overflow");
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
        let semaphore = Semaphore::new(0);

        let w1 = WatchedWaker::new();
        let w2 = WatchedWaker::new();

        let waker1 = Waker::from(w1.clone());
        let waker2 = Waker::from(w2.clone());

        let mut ctx = Context::from_waker(&waker1);
        let mut ctx2 = Context::from_waker(&waker2);

        let mut fut = pin!(semaphore.acquire());

        assert!(fut.as_mut().poll(&mut ctx).is_pending());
        assert!(fut.as_mut().poll(&mut ctx2).is_pending());

        semaphore.add_permits(1);

        assert!(!w1.woken());
        assert!(w2.woken());
    }
}
