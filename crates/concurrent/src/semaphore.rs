use crate::Mutex;
use core::future::Future;
use core::pin::Pin;
use core::task::Waker;
use core::task::{Context, Poll};
use std::collections::VecDeque;

pub struct Semaphore {
    state: Mutex<State>,
}

struct Waiter {
    waker: Waker,
}

struct State {
    waiters: VecDeque<Waiter>,
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
}

impl Drop for Acquire<'_> {
    fn drop(&mut self) {}
}

impl<'a> Future for Acquire<'a> {
    type Output = Result<SemaphorePermit<'a>, AcquireError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.semaphore.state.lock();
        if state.permits > 0 {
            state.permits -= 1;
            return Poll::Ready(Ok(SemaphorePermit {
                semaphore: self.semaphore,
            }));
        }

        state.waiters.push_back(Waiter {
            waker: cx.waker().clone(),
        });
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
                waiters: VecDeque::new(),
                permits: num_permits,
            }),
        }
    }

    pub fn acquire(&self) -> Acquire<'_> {
        Acquire { semaphore: self }
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
            for _ in 0..num_permits {
                if state.waiters.is_empty() {
                    break;
                }

                to_wake.push(state.waiters.pop_front().unwrap().waker)
            }
        }
        for waker in to_wake {
            waker.wake()
        }
    }

    pub fn available_permits(&self) -> usize {
        self.state.lock().permits
    }
}
