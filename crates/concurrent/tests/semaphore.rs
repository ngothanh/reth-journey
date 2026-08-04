#[cfg(not(loom))]
mod tests {
    use concurrent::Semaphore;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn semaphore_try_acquire() {
        let semaphore = Semaphore::new(2);
        {
            semaphore.try_acquire().unwrap();
            semaphore.try_acquire().unwrap();
        }

        assert!(semaphore.try_acquire().is_some());
    }

    #[test]
    fn semaphore_acquire_succeeds() {
        let semaphore = Arc::new(Semaphore::new(1));
        let p1 = semaphore.acquire();

        let s1 = semaphore.clone();
        let handle = thread::spawn(move || {
            s1.acquire();
        });
        drop(p1);
        handle.join().unwrap();
    }
}

#[cfg(loom)]
mod loom {
    use concurrent::Semaphore;
    use loom::sync::atomic::{AtomicUsize, Ordering};
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn loom_release_publishes_prior_writes() {
        loom::model(|| {
            let s = Arc::new(Semaphore::new(0));
            let payload = Arc::new(AtomicUsize::new(0));
            let p1 = payload.clone();
            let s1 = s.clone();
            let s2 = s.clone();
            let t1 = loom::thread::spawn(move || {
                p1.store(42, Ordering::Relaxed);
                s1.add(1);
            });
            let t2 = loom::thread::spawn(move || {
                s2.acquire();
                assert_eq!(payload.load(Ordering::Relaxed), 42);
            });
            t1.join().unwrap();
            t2.join().unwrap();
        })
    }

    #[test]
    fn loom_add_does_not_loose_permits() {
        loom::model(|| {
            let s = Arc::new(Semaphore::new(0));
            let s1 = s.clone();
            let s2 = s.clone();
            let h1 = thread::spawn(move || {
                s1.add(1);
            });
            let h2 = thread::spawn(move || s2.add(1));

            h1.join().unwrap();
            h2.join().unwrap();
            let p1 = s.try_acquire();
            let p2 = s.try_acquire();
            assert!(p1.is_some());
            assert!(p2.is_some());
            assert!(s.try_acquire().is_none());
        })
    }
}
