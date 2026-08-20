use crate::wait_list::WaiterState;
use crate::wake_list::WakeList;
use crate::{Mutex, WaitList, Waiter};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::ptr::NonNull;
use WaiterState::{Done, Granted, Idle, Waiting};

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
}

unsafe impl Send for Acquire<'_> {}

impl Drop for Acquire<'_> {
    fn drop(&mut self) {
        match self.node.state() {
            Waiting => {
                let mut state = self.semaphore.state.lock();
                state.waiters.unlink(NonNull::from(&self.node))
            }
            Granted => {
                let mut wakers = WakeList::new();
                {
                    let mut state = self.semaphore.state.lock();
                    match state.waiters.pop_front() {
                        Some(w) => unsafe {
                            (*w.as_ptr()).set_state(Granted);
                            if let Some(x) = (*w.as_ptr()).take_waker() {
                                wakers.push(x);
                            }
                        },
                        None => state.permits = state.permits.checked_add(1).expect("overflow"),
                    }
                }
                wakers.wake_all();
            }
            _ => {}
        }
    }
}

impl<'a> Future for Acquire<'a> {
    type Output = Result<SemaphorePermit<'a>, AcquireError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let node = NonNull::from(&this.node);
        let mut state = this.semaphore.state.lock();

        match this.node.state() {
            Granted => {
                this.node.set_state(Done);
                Poll::Ready(Ok(SemaphorePermit {
                    semaphore: this.semaphore,
                }))
            }
            Idle => {
                if state.closed {
                    this.node.set_state(Done);
                    Poll::Ready(Err(AcquireError {}))
                } else if state.permits > 0 {
                    state.permits -= 1;
                    this.node.set_state(Done);
                    Poll::Ready(Ok(SemaphorePermit {
                        semaphore: this.semaphore,
                    }))
                } else {
                    this.node.update_waker(cx.waker());
                    unsafe { this.node.set_state(Waiting) };
                    state.waiters.push_back(node);
                    Poll::Pending
                }
            }
            Waiting => {
                if state.closed {
                    this.node.set_state(Done);
                    state.waiters.unlink(node);
                    Poll::Ready(Err(AcquireError {}))
                } else {
                    this.node.update_waker(cx.waker());
                    Poll::Pending
                }
            }
            Done => unreachable!("polled after completion"),
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
        let mut to_wake = WakeList::new();
        let mut left = num_permits;
        loop {
            {
                let mut state = self.state.lock();
                while left > 0 && to_wake.can_push() {
                    if let Some(w) = state.waiters.pop_front() {
                        left -= 1;
                        unsafe {
                            (*w.as_ptr()).set_state(Granted);
                            if let Some(w) = (*w.as_ptr()).take_waker() {
                                to_wake.push(w);
                            }
                        }
                    } else {
                        state.permits = state.permits.checked_add(left).expect("semaphore overflow");
                        left = 0;
                        break;
                    }
                }
            }
            to_wake.wake_all();
            if left == 0 {
                break;
            }
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
