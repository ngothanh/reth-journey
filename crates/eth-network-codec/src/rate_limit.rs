use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{sleep, Instant, Sleep};

struct TokenBucket {
    capacity: usize,
    current_token_count: f64,
    refill_per_sec: f64,
    last_refill: Instant,
    delay: Option<Pin<Box<Sleep>>>,
}

impl TokenBucket {
    pub fn new(capacity: usize, refill_per_sec: f64) -> Self {
        Self {
            capacity,
            current_token_count: capacity as f64,
            refill_per_sec,
            last_refill: Instant::now(),
            delay: None,
        }
    }

    pub fn poll_acquire(&mut self, cx: &mut Context) -> Poll<()> {
        loop {
            let now = Instant::now();
            let earned = elapsed_since(&now, &self.last_refill) * self.refill_per_sec;
            self.current_token_count += earned;
            self.last_refill = now;
            self.current_token_count = self.current_token_count.min(self.capacity as f64);
            if self.current_token_count >= 1.0 {
                self.current_token_count -= 1.0;
                self.delay = None;
                return Poll::Ready(());
            }

            if self.delay.is_none() {
                let remaining = 1.0 - self.current_token_count;
                let secs = remaining / self.refill_per_sec as f64;
                let wait = Duration::from_secs_f64(secs);
                self.delay = Some(Box::pin(sleep(wait)));
            }

            let delay = self.delay.as_mut().unwrap();
            match delay.as_mut().poll(cx) {
                Poll::Ready(_) => {
                    self.delay = None;
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

fn elapsed_since(now: &Instant, last_refill: &Instant) -> f64 {
    now.duration_since(*last_refill).as_secs_f64()
}
