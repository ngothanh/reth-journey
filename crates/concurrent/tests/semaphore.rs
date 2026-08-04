#[cfg(not(loom))]
mod tests {
    use concurrent::Semaphore;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// Every permit must be *bound*. An unbound `try_acquire()` is a temporary
    /// that drops at the semicolon, handing the permit straight back — a test
    /// written that way passes against a semaphore that ignores its count.
    #[test]
    fn try_acquire_returns_none_when_exhausted() {
        let sem = Semaphore::new(2);

        let p1 = sem.try_acquire();
        let p2 = sem.try_acquire();
        assert!(p1.is_some(), "first of two permits refused");
        assert!(p2.is_some(), "second of two permits refused");

        assert!(sem.try_acquire().is_none(), "granted a third permit from two");
    }

    #[test]
    fn permit_returns_on_drop() {
        let sem = Semaphore::new(1);
        {
            let _p = sem.try_acquire().expect("the only permit was refused");
            assert!(sem.try_acquire().is_none(), "permit handed out twice");
        }
        assert!(sem.try_acquire().is_some(), "permit not returned on drop");
    }

    #[test]
    fn new_zero_never_grants() {
        let sem = Semaphore::new(0);
        assert!(sem.try_acquire().is_none(), "granted a permit from an empty semaphore");
    }

    #[test]
    fn add_restores_permits() {
        let sem = Semaphore::new(0);
        sem.add(3);

        let held: Vec<_> = (0..3).map(|_| sem.try_acquire()).collect();
        assert!(held.iter().all(|p| p.is_some()), "add(3) did not yield 3 permits");
        assert!(sem.try_acquire().is_none(), "add(3) yielded more than 3 permits");
    }

    /// Proves `acquire` genuinely *waits* rather than returning early. The flag
    /// is set before the permit is added, so observing it false after `acquire`
    /// returns would mean acquire handed out a permit that did not exist.
    #[test]
    fn acquire_blocks_until_permit_available() {
        let sem = Arc::new(Semaphore::new(0));
        let released = Arc::new(AtomicBool::new(false));

        let s = sem.clone();
        let r = released.clone();
        let adder = thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            r.store(true, SeqCst);
            s.add(1);
        });

        let _permit = sem.acquire();
        assert!(released.load(SeqCst), "acquire returned before any permit was added");

        adder.join().unwrap();
    }

    /// N threads hammering a semaphore of `permits` must never put more than
    /// `permits` of them inside the critical section at once.
    fn assert_never_exceeds(permits: u32, threads: usize, iters: usize) {
        let sem = Arc::new(Semaphore::new(permits));
        let in_flight = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let sem = sem.clone();
                let in_flight = in_flight.clone();
                thread::spawn(move || {
                    for _ in 0..iters {
                        let _permit = sem.acquire();
                        let inside = in_flight.fetch_add(1, SeqCst) + 1;
                        assert!(
                            inside <= permits as usize,
                            "{inside} holders inside a {permits}-permit semaphore"
                        );
                        // Widen the critical section. Without this the section is
                        // a few nanoseconds long and overlaps essentially never
                        // occur — verified: a semaphore handing out UNLIMITED
                        // permits still passed this test at 4x200 iterations.
                        // A stress test only finds a race it gives time to happen.
                        thread::yield_now();
                        in_flight.fetch_sub(1, SeqCst);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(in_flight.load(SeqCst), 0, "a permit leaked");
    }

    #[test]
    fn one_permit_is_mutually_exclusive() {
        assert_never_exceeds(1, 4, 200);
    }

    #[test]
    fn n_permits_cap_concurrency_at_n() {
        assert_never_exceeds(3, 8, 200);
    }

    /// `add` keeps a `checked_add` rather than collapsing to `fetch_add`, so
    /// overflow is loud instead of wrapping the count to a bogus value.
    #[test]
    #[should_panic(expected = "semaphore overflow")]
    fn add_past_u32_max_panics() {
        let sem = Semaphore::new(u32::MAX);
        sem.add(1);
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

    /// A waiter that goes to sleep must always be woken by a later `add`.
    ///
    /// There is deliberately no `assert!` here — the property is *liveness*,
    /// not a value. If any interleaving loses the wakeup, the waiting thread
    /// stays parked inside `acquire`, `join()` never returns, and loom reports
    /// the deadlock. Passing means **no** schedule leaves the waiter asleep.
    ///
    /// The race under test: the waiter observes `count == 0` and registers in
    /// `waiters`, while `add` bumps `count` and then reads `waiters` to decide
    /// whether to signal at all. Every interleaving of those four operations is
    /// explored here.
    #[test]
    fn loom_no_lost_wakeup() {
        loom::model(|| {
            let s = Arc::new(Semaphore::new(0));
            let s1 = s.clone();

            let waiter = thread::spawn(move || {
                let _permit = s1.acquire();
            });

            s.add(1);
            waiter.join().unwrap();
        })
    }

    /// One permit, two threads, blocking `acquire`: no interleaving may put both
    /// inside at once.
    ///
    /// The counter is incremented *while the permit is held*, which is what makes
    /// this test about simultaneity rather than about counting successes. Two
    /// threads each succeeding is perfectly legal if they are serialised — only
    /// overlapping is a bug.
    #[test]
    fn loom_mutual_exclusion_one_permit() {
        loom::model(|| {
            let s = Arc::new(Semaphore::new(1));
            let inside = Arc::new(AtomicUsize::new(0));

            let handles: Vec<_> = (0..2)
                .map(|_| {
                    let s = s.clone();
                    let inside = inside.clone();
                    thread::spawn(move || {
                        let _permit = s.acquire();
                        let n = inside.fetch_add(1, Ordering::SeqCst) + 1;
                        assert_eq!(n, 1, "two threads held the only permit at once");
                        inside.fetch_sub(1, Ordering::SeqCst);
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        })
    }

    /// Same invariant via the non-blocking path. A thread that is refused simply
    /// does nothing, so this exercises the `try_acquire` CAS loop against a
    /// concurrent competitor rather than the wait protocol.
    #[test]
    fn loom_try_acquire_never_oversubscribes() {
        loom::model(|| {
            let s = Arc::new(Semaphore::new(1));
            let inside = Arc::new(AtomicUsize::new(0));

            let handles: Vec<_> = (0..2)
                .map(|_| {
                    let s = s.clone();
                    let inside = inside.clone();
                    thread::spawn(move || {
                        if let Some(_permit) = s.try_acquire() {
                            let n = inside.fetch_add(1, Ordering::SeqCst) + 1;
                            assert_eq!(n, 1, "try_acquire oversubscribed a 1-permit semaphore");
                            inside.fetch_sub(1, Ordering::SeqCst);
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        })
    }
}
