mod tests {
    use concurrent::{Semaphore, TryAcquireError};
    use std::future::Future;
    use std::pin::pin;
    use std::sync::Arc;
    use std::task::{Context, Poll, Waker};
    use std::time::Duration;

    #[test]
    fn try_acquire_permits() {
        let semaphore = Semaphore::new(1);
        let p1 = semaphore.try_acquire();
        let p2 = semaphore.try_acquire();
        assert!(p1.is_ok());
        assert!(p2.is_err());
    }

    #[test]
    fn try_acquire_with_no_permit() {
        let semaphore = Semaphore::new(0);
        assert!(semaphore.try_acquire().is_err());
    }

    #[test]
    fn try_acquire_drop() {
        let semaphore = Semaphore::new(1);
        let permit = semaphore.try_acquire().ok().unwrap();
        drop(permit);
        assert!(semaphore.try_acquire().is_ok());
    }

    #[tokio::test]
    async fn acquire() {
        let semaphore = Semaphore::new(1);
        let result = semaphore.acquire().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn waiter_blocks_until_permit_released() {
        let sem = Arc::new(Semaphore::new(1));
        let permit_a = sem.acquire().await.unwrap();
        assert_eq!(sem.available_permits(), 0, "A got the permit. Remaining: 0");

        let sem_b = Arc::clone(&sem);
        let b = tokio::spawn(async move {
            let _permit_b = sem_b.acquire().await.unwrap();
        });

        tokio::task::yield_now().await;
        assert!(
            !b.is_finished(),
            "B needs to wait till permit returned by A"
        );

        drop(permit_a);

        tokio::time::timeout(Duration::from_secs(1), b)
            .await
            .expect("lost wakeup: B cannot get the permit")
            .expect("task B panicked");
    }

    #[tokio::test]
    async fn add_permits() {
        let semaphore = Arc::new(Semaphore::new(0));
        let s1 = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _ = s1.acquire().await;
        });
        tokio::task::yield_now().await;
        assert!(!handle.is_finished());
        semaphore.add_permits(2);
        tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("lost wakeup")
            .unwrap();
    }

    #[tokio::test]
    async fn add_permits_wakes_all_waiters() {
        let semaphore = Arc::new(Semaphore::new(0));

        let mut handles = Vec::new();
        for _ in 0..3 {
            let s = semaphore.clone();
            handles.push(tokio::spawn(async move {
                s.acquire().await.unwrap();
            }));
        }

        tokio::task::yield_now().await;
        assert_eq!(semaphore.available_permits(), 0, "No permits available");

        semaphore.add_permits(3);

        for h in handles {
            tokio::time::timeout(Duration::from_secs(1), h)
                .await
                .expect("lost wakeup")
                .unwrap();
        }

        assert_eq!(
            semaphore.available_permits(),
            3,
            "3 permits must be returned"
        );
    }

    #[test]
    fn granted_permit_is_not_stealable() {
        let semaphore = Semaphore::new(0);
        let mut acquire = pin!(semaphore.acquire());
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);

        assert!(acquire.as_mut().poll(&mut context).is_pending());

        semaphore.add_permits(1);
        assert!(
            semaphore.try_acquire().is_err(),
            "permit was assigned to acquire, cannot be steal"
        );
    }

    #[test]
    fn cancel_after_grant_return_permit() {
        let semaphore = Semaphore::new(0);
        let mut acquire = Box::pin(semaphore.acquire());
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);

        assert!(acquire.as_mut().poll(&mut context).is_pending());

        semaphore.add_permits(1);
        drop(acquire);

        assert_eq!(semaphore.available_permits(), 1);
    }

    #[test]
    fn cancel_grant_flow_to_next_waiter() {
        let semaphore = Semaphore::new(0);
        let waker = Waker::noop();
        let mut ctx = Context::from_waker(&waker);

        let mut a1 = Box::pin(semaphore.acquire());
        let mut a2 = pin!(semaphore.acquire());

        assert!(a1.as_mut().poll(&mut ctx).is_pending());
        assert!(a2.as_mut().poll(&mut ctx).is_pending());

        semaphore.add_permits(1);
        drop(a1);
        assert!(a2.as_mut().poll(&mut ctx).is_ready());
    }

    #[test]
    fn try_acquire_after_close_is_closed() {
        // permit vẫn còn, nhưng closed thắng permits
        let semaphore = Semaphore::new(1);
        semaphore.close();
        assert!(matches!(
            semaphore.try_acquire(),
            Err(TryAcquireError::Closed)
        ));
    }

    #[test]
    fn fresh_acquire_after_close_errors() {
        let semaphore = Semaphore::new(1);
        semaphore.close();

        let waker = Waker::noop();
        let mut ctx = Context::from_waker(&waker);
        let mut acquire = pin!(semaphore.acquire());

        assert!(matches!(
            acquire.as_mut().poll(&mut ctx),
            Poll::Ready(Err(_))
        ));
    }

    #[test]
    fn waiting_acquire_errors_on_close() {
        let semaphore = Semaphore::new(0);
        let waker = Waker::noop();
        let mut ctx = Context::from_waker(&waker);
        let mut acquire = pin!(semaphore.acquire());

        assert!(acquire.as_mut().poll(&mut ctx).is_pending()); // A xếp hàng
        semaphore.close();                                     // wake A với lỗi
        assert!(matches!(
            acquire.as_mut().poll(&mut ctx),
            Poll::Ready(Err(_))
        ));
    }
}
