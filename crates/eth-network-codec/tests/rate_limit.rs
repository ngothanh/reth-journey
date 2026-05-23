use eth_network_codec::{RateLimitedStream, TokenBucket};
use futures_core::Stream;
use futures_util::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::Instant;

// `start_paused` runs on virtual time — the bucket's `sleep`s fast-forward,
// so 1000 peers finish in milliseconds and the timing is exact. The default
// current-thread runtime is required for paused-time auto-advance.
#[tokio::test(start_paused = true)]
async fn rate_limited_under_load() {
    const PEERS: usize = 1000;
    const CAPACITY: usize = 100;
    const REFILL_PER_SEC: f64 = 10.0;
    const N: usize = 200; // CAPACITY free + 100 rate-limited

    // Each peer gets its own bucket + stream and pulls N items.
    let mut handles = Vec::with_capacity(PEERS);
    for _ in 0..PEERS {
        handles.push(tokio::task::spawn(async move {
            let bucket = TokenBucket::new(CAPACITY as u64, REFILL_PER_SEC);
            let mut stream = RateLimitedStream::new(TestStream::new(), bucket);
            let start = Instant::now();
            for _ in 0..N {
                stream.next().await;
            }
            start.elapsed()
        }));
    }

    // Bucket starts full → first CAPACITY items free, the rest paced at the rate.
    let expected = Duration::from_secs_f64((N - CAPACITY) as f64 / REFILL_PER_SEC);
    let tolerance = expected.mul_f64(0.15);

    for handle in handles {
        // `.await` returning at all proves the task didn't hang on a lost
        // wakeup; the assert then checks it held the configured rate.
        let elapsed = handle.await.expect("a peer task panicked or hung");
        let diff = elapsed
            .saturating_sub(expected)
            .max(expected.saturating_sub(elapsed));
        assert!(
            diff <= tolerance,
            "peer stream took {elapsed:?}, expected {expected:?} ± {tolerance:?}",
        );
    }
}

/// Always immediately ready — so the *limiter*, not the source, paces items.
struct TestStream {}

impl TestStream {
    fn new() -> Self {
        Self {}
    }
}

impl Stream for TestStream {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(()))
    }
}
