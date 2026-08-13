mod tests {
    use concurrent::Semaphore;
    use std::sync::Arc;
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
}
