//! R6: rate-limit backoff for RPC calls.
//!
//! Built on W3's `RetryFuture`, which owns the retry *count* but has no notion of delay —
//! it re-polls immediately. Retrying a 429 with no wait is worse than not retrying, so the
//! backoff is injected here: each attempt after the first sleeps before the call is made.

use core::future::Future;
use core::time::Duration;
use eth_network_codec::RetryFuture;

const MAX_ATTEMPTS: u32 = 3;
const BASE_BACKOFF_MS: u64 = 100;

/// Exponential: attempt 0 fires immediately, then 100ms, then 200ms.
fn backoff(attempt: u32) -> Option<Duration> {
    attempt
        .checked_sub(1)
        .map(|n| Duration::from_millis(BASE_BACKOFF_MS << n))
}

/// Run `make` until it succeeds or `MAX_ATTEMPTS` is exhausted, sleeping between tries.
///
/// `make` is called once per attempt. The futures alloy hands back are lazy — nothing is
/// sent until the future is polled — so constructing before the sleep costs nothing.
pub(crate) fn with_backoff<MakeF, Fut, T, E>(
    mut make: MakeF,
) -> impl Future<Output = Result<T, E>>
where
    MakeF: FnMut() -> Fut + Unpin,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempt = 0u32;

    RetryFuture::new(MAX_ATTEMPTS, move || {
        let delay = backoff(attempt);
        attempt += 1;
        let call = make();

        async move {
            if let Some(delay) = delay {
                eprintln!("retrying in {}ms", delay.as_millis());
                tokio::time::sleep(delay).await;
            }
            call.await
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_is_exponential_and_skips_the_first_attempt() {
        assert_eq!(backoff(0), None, "first attempt must not sleep");
        assert_eq!(backoff(1), Some(Duration::from_millis(100)));
        assert_eq!(backoff(2), Some(Duration::from_millis(200)));
    }

    #[tokio::test]
    async fn succeeds_without_retrying() {
        let mut calls = 0;
        let result: Result<u32, ()> = with_backoff(|| {
            calls += 1;
            async { Ok(7) }
        })
        .await;

        assert_eq!(result, Ok(7));
    }

    #[tokio::test]
    async fn retries_up_to_the_cap_then_gives_up() {
        let attempts = std::cell::Cell::new(0u32);
        let result: Result<u32, &str> = with_backoff(|| {
            attempts.set(attempts.get() + 1);
            async { Err("429") }
        })
        .await;

        assert_eq!(result, Err("429"));
        assert_eq!(attempts.get(), MAX_ATTEMPTS, "should stop at the cap");
    }

    #[tokio::test]
    async fn recovers_on_a_later_attempt() {
        let attempts = std::cell::Cell::new(0u32);
        let result: Result<u32, &str> = with_backoff(|| {
            let n = attempts.get() + 1;
            attempts.set(n);
            async move {
                if n < 3 {
                    Err("429")
                } else {
                    Ok(n)
                }
            }
        })
        .await;

        assert_eq!(result, Ok(3));
        assert_eq!(attempts.get(), 3);
    }
}
