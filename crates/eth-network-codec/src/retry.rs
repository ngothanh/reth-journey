use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct RetryFuture<F, MakeF> {
    current: Pin<Box<F>>,
    make_future: MakeF,
    attempts_remaining: u32,
}

impl<F, MakeF> RetryFuture<F, MakeF>
where
    MakeF: FnMut() -> F,
{
    pub fn new(max_attempts: u32, mut make_future: MakeF) -> Self {
        assert!(max_attempts > 0, "max_attempts must be > 0");
        let current = Box::pin(make_future());
        Self {
            current,
            make_future,
            attempts_remaining: max_attempts - 1,
        }
    }
}

impl<F, MakeF, T, E> Future for RetryFuture<F, MakeF>
where
    F: Future<Output = Result<T, E>>,
    MakeF: FnMut() -> F + Unpin,
{
    type Output = Result<T, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.current.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(result)) => return Poll::Ready(Ok(result)),
                Poll::Ready(Err(e)) => {
                    if self.attempts_remaining == 0 {
                        return Poll::Ready(Err(e));
                    }
                    self.attempts_remaining -= 1;
                    let next = (self.make_future)();
                    self.current.set(next);
                }
            }
        }
    }
}
