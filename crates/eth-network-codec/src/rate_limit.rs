use futures_core::Stream;
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{sleep_until, Instant, Sleep};

pub struct TokenBucket {
    capacity: u64,
    current_token_count: u64,
    refill_period_ns: u64,
    last_refill: Instant,
    delay: Option<Pin<Box<Sleep>>>,
}

impl TokenBucket {
    pub fn new(capacity: u64, refill_per_sec: f64) -> Self {
        Self {
            capacity,
            current_token_count: capacity,
            refill_period_ns: (1_000_000_000f64 / refill_per_sec) as u64,
            last_refill: Instant::now(),
            delay: None,
        }
    }

    pub fn poll_acquire(&mut self, cx: &mut Context) -> Poll<()> {
        loop {
            let now = Instant::now();
            let elapsed_ns = elapsed_since(&now, &self.last_refill);
            let tokens_to_add = elapsed_ns / self.refill_period_ns;
            self.last_refill += Duration::from_nanos(tokens_to_add * self.refill_period_ns);
            self.current_token_count =
                (self.current_token_count + tokens_to_add).min(self.capacity);

            if self.current_token_count >= 1 {
                self.current_token_count -= 1;
                return Poll::Ready(());
            }

            let deadline = self.last_refill + Duration::from_nanos(self.refill_period_ns);
            if let Some(sleep) = self.delay.as_mut() {
                sleep.as_mut().reset(deadline);
            } else {
                self.delay = Some(Box::pin(sleep_until(deadline)));
            }

            let sleep = self.delay.as_mut().unwrap();
            match sleep.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(_) => {}
            }
        }
    }
}

fn elapsed_since(now: &Instant, last_refill: &Instant) -> u64 {
    now.duration_since(*last_refill).as_nanos() as u64
}

pin_project! {
    pub struct RateLimitedStream<S: Stream> {
        #[pin]
        stream: S,
        bucket: TokenBucket,
        buffer: Option<S::Item>,
    }
}

impl<S: Stream> RateLimitedStream<S> {
    pub fn new(stream: S, token_bucket: TokenBucket) -> Self {
        Self {
            stream,
            bucket: token_bucket,
            buffer: None,
        }
    }
}

impl<S: Stream> Stream for RateLimitedStream<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        let item = match this.buffer.take() {
            Some(item) => item,
            None => match this.stream.poll_next(cx) {
                Poll::Ready(Some(item)) => item,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            },
        };

        match this.bucket.poll_acquire(cx) {
            Poll::Ready(()) => Poll::Ready(Some(item)),
            Poll::Pending => {
                *this.buffer = Some(item);
                Poll::Pending
            }
        }
    }
}
