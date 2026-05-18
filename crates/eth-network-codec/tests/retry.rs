use eth_network_codec::RetryFuture;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn succeeds_first_try() {
    let result = RetryFuture::new(3, || async { Ok::<_, ()>(42_u32) }).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42_u32);
}

#[tokio::test]
async fn retries_until_success() {
    let attempts = Arc::new(AtomicU32::new(0));
    let result = RetryFuture::new(5, || async {
        let a = attempts.clone();
        let i = a.fetch_add(1, Ordering::Relaxed) + 1;
        if i < 3 {
            Err::<u32, _>("not yet")
        } else {
            Ok(12)
        }
    })
    .await;
    assert_eq!(result.unwrap(), 12);
    assert_eq!(attempts.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn gives_up_after_exhausting_attempts() {
    let attempts = Arc::new(AtomicU32::new(0));
    let result = RetryFuture::new(5, || async {
        let _ = attempts.fetch_add(1, Ordering::Relaxed) + 1;
        Err::<u32, _>("never work")
    })
    .await;
    assert!(result.is_err());
    assert_eq!(attempts.load(Ordering::Relaxed), 5);
}

#[tokio::test]
async fn handles_pending_inner_future() {
    let attempts = Arc::new(AtomicU32::new(0));
    let result = RetryFuture::new(5, || async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        let n = attempts.fetch_add(1, Ordering::Relaxed) + 1;
        if n < 3 {
            Err::<u32, _>("more")
        } else {
            Ok(99)
        }
    })
    .await;

    assert!(result.is_ok());
    assert_eq!(attempts.load(Ordering::Relaxed), 3);
}
