#[cfg(not(loom))]
mod tests {
    use concurrent::RwLock;
    use std::sync::Arc;
    use std::thread;

    struct Inner {
        a: i32,
        b: i32,
        c: i32,
        d: i32,
    }

    #[test]
    fn read_write_exclusively() {
        let inner = Inner {
            a: 1,
            b: 2,
            c: 3,
            d: 4,
        };

        let lock = RwLock::new(inner);
        assert_eq!(lock.read().a, 1);
        lock.write().a = 2;
        assert_eq!(lock.read().a, 2);
    }

    #[test]
    fn read_sees_initial_value() {
        let lock = RwLock::new(42);

        assert_eq!(*lock.read(), 42);
    }

    #[test]
    fn write_mutates_then_read_observes() {
        let lock = RwLock::new(42);
        *lock.write() = 45;

        assert_eq!(*lock.read(), 45);
    }

    #[test]
    fn two_read_guards_coexist() {
        let lock = RwLock::new(43);
        let r1 = lock.read();
        let r2 = lock.read();

        assert_eq!(*r1, 43);
        assert_eq!(*r2, 43);
    }

    #[test]
    fn write_then_write_after_drop() {
        let lock = RwLock::new(42);
        {
            let mut w1 = lock.write();
            *w1 = 45;
        }

        {
            let mut w2 = lock.write();
            *w2 = 43;
        }

        assert_eq!(*lock.read(), 43);
    }

    #[test]
    fn concurrent_writers_sum() {
        use std::sync::Arc;
        use std::thread;

        const WRITERS: u64 = 8;
        const PER_WRITER: u64 = if cfg!(miri) { 200 } else { 50_000 };

        let lock = Arc::new(RwLock::new(0u64));

        let writers: Vec<_> = (0..WRITERS)
            .map(|_| {
                let lock = Arc::clone(&lock);
                thread::spawn(move || {
                    for _ in 0..PER_WRITER {
                        let mut w = lock.write();
                        *w += 1;
                    }
                })
            })
            .collect();

        for w in writers {
            w.join().unwrap();
        }

        assert_eq!(*lock.read(), WRITERS * PER_WRITER);
    }

    #[test]
    fn reader_never_sees_torn_write() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::thread;

        const WRITES: u64 = if cfg!(miri) { 200 } else { 200_000 };
        const READERS: usize = 4;

        // Invariant: the two fields are ALWAYS equal to an outside observer.
        // The writer sets them one at a time (a, then b), so inside the write
        // critical section there is a window where a != b. If read/write
        // exclusion is broken, a reader can acquire during that window and
        // observe the torn state.
        let lock = Arc::new(RwLock::new((0u64, 0u64)));
        let stop = Arc::new(AtomicBool::new(false));

        let readers: Vec<_> = (0..READERS)
            .map(|_| {
                let lock = Arc::clone(&lock);
                let stop = Arc::clone(&stop);
                thread::spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        let g = lock.read();
                        assert_eq!(g.0, g.1, "reader observed a torn write: {} != {}", g.0, g.1);
                    }
                })
            })
            .collect();

        let writer = {
            let lock = Arc::clone(&lock);
            thread::spawn(move || {
                for v in 1..=WRITES {
                    let mut g = lock.write();
                    g.0 = v;
                    std::hint::spin_loop(); // widen the window where g.0 != g.1
                    g.1 = v;
                }
            })
        };

        writer.join().unwrap();
        stop.store(true, Ordering::Relaxed);
        for r in readers {
            r.join().unwrap();
        }
    }

    #[test]
    fn mixed_stress() {
        use std::sync::atomic::{AtomicBool, Ordering};

        const WRITERS: usize = 4;
        const READERS: usize = 4;
        const PUSHES_PER_WRITER: u64 = if cfg!(miri) { 50 } else { 20_000 };

        let lock = Arc::new(RwLock::new(Vec::<u64>::new()));
        let stop = Arc::new(AtomicBool::new(false));

        let readers: Vec<_> = (0..READERS)
            .map(|_| {
                let lock = Arc::clone(&lock);
                let stop = Arc::clone(&stop);
                thread::spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        let g = lock.read();
                        for (i, x) in g.iter().enumerate() {
                            assert_eq!(*x, i as u64, "reader saw an inconsistent snapshot");
                        }
                    }
                })
            })
            .collect();

        let writers: Vec<_> = (0..WRITERS)
            .map(|_| {
                let lock = Arc::clone(&lock);
                thread::spawn(move || {
                    for _ in 0..PUSHES_PER_WRITER {
                        let mut g = lock.write();
                        let next = g.len() as u64;
                        g.push(next);
                    }
                })
            })
            .collect();

        for w in writers {
            w.join().unwrap();
        }
        stop.store(true, Ordering::Relaxed);
        for r in readers {
            r.join().unwrap();
        }

        let g = lock.read();
        assert_eq!(g.len() as u64, WRITERS as u64 * PUSHES_PER_WRITER);
        for (i, x) in g.iter().enumerate() {
            assert_eq!(*x, i as u64);
        }
    }

    #[test]
    fn many_readers_complete() {
        let lock = Arc::new(RwLock::new(0u64));

        const READERS: usize = 4000;
        let handles: Vec<_> = (0..READERS)
            .map(|_| {
                let l = Arc::clone(&lock);
                thread::spawn(move || {
                    l.read();
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    }

    // Timing/scheduling test: meaningless under Miri's cooperative scheduler.
    #[cfg_attr(miri, ignore)]
    #[test]
    fn no_writer_starvation() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::mpsc;
        use std::time::Duration;

        const READERS: usize = 8;
        const TIMEOUT: Duration = Duration::from_secs(10);

        // A steady stream of readers keeps `state` above zero almost continuously.
        // Under a reader-preferring lock the writer would never find a gap and would
        // starve; the writer-priority flag must choke off new readers so the writer
        // gets in. We assert it acquires within a generous timeout.
        let lock = Arc::new(RwLock::new(0u64));
        let stop = Arc::new(AtomicBool::new(false));

        let readers: Vec<_> = (0..READERS)
            .map(|_| {
                let lock = Arc::clone(&lock);
                let stop = Arc::clone(&stop);
                thread::spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        let g = lock.read();
                        // Hold briefly so the lock is genuinely contended and the
                        // writer is forced to wait (and set its "waiting" flag).
                        for _ in 0..50 {
                            std::hint::spin_loop();
                        }
                        drop(g);
                    }
                })
            })
            .collect();

        let (tx, rx) = mpsc::channel();
        let writer = {
            let lock = Arc::clone(&lock);
            thread::spawn(move || {
                let _w = lock.write(); // blocks until acquired
                let _ = tx.send(()); // signal the moment we get in
            })
        };

        // Did the writer acquire before the timeout?
        let acquired = rx.recv_timeout(TIMEOUT);

        // Let the readers wind down, then join everything (no thread left hanging:
        // once readers stop, even a starved writer eventually gets in and exits).
        stop.store(true, Ordering::Relaxed);
        for r in readers {
            r.join().unwrap();
        }
        writer.join().unwrap();

        assert!(
            acquired.is_ok(),
            "writer starved: did not acquire within {TIMEOUT:?}"
        );
    }

    #[test]
    fn guard_released_on_panic() {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let lock = RwLock::new(0u64);

        // Panic while holding the write lock. As the stack unwinds, the WriteGuard's
        // Drop runs and must release the lock. (AssertUnwindSafe: after the panic we
        // rely only on the lock being released, not on the data's invariants.)
        // NOTE: an "explicit panic" message on stderr here is expected — the test passes.
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut w = lock.write();
            *w = 1;
            panic!("boom while holding the write lock");
        }));
        assert!(result.is_err(), "closure should have panicked");

        // If Drop had NOT released the lock, this write() would deadlock forever.
        {
            let mut w = lock.write();
            *w = 2;
        }
        assert_eq!(*lock.read(), 2);
    }
}


// Run with:  RUSTFLAGS="--cfg loom" cargo test -p concurrent --lib loom
// loom exhaustively explores thread interleavings of the atomic protocol. Keep the
// thread count and per-thread work tiny — loom's state space is combinatorial.
#[cfg(loom)]
mod loom_tests {
    use super::RwLock;
    use loom::sync::Arc;
    use loom::thread;

    // Writer/writer mutual exclusion: if any interleaving let two writers into the
    // critical section at once, their increments would interleave and the sum would
    // come out below 2. loom checks *every* schedule.
    #[test]
    fn two_writers_are_exclusive() {
        loom::model(|| {
            let lock = Arc::new(RwLock::new(0u32));

            let other = {
                let lock = Arc::clone(&lock);
                thread::spawn(move || {
                    *lock.write() += 1;
                })
            };
            *lock.write() += 1;
            other.join().unwrap();

            assert_eq!(*lock.read(), 2);
        });
    }

    // Reader/writer: across every interleaving the reader observes a valid value
    // (never a torn one), and neither thread is left parked — i.e. no lost wakeup /
    // deadlock in the flag + writer_wake_count handshake.
    #[test]
    fn reader_and_writer_make_progress() {
        loom::model(|| {
            let lock = Arc::new(RwLock::new(0u32));

            let writer = {
                let lock = Arc::clone(&lock);
                thread::spawn(move || {
                    *lock.write() = 42;
                })
            };

            let seen = *lock.read();
            assert!(seen == 0 || seen == 42, "reader saw an invalid value: {seen}");

            writer.join().unwrap();
            assert_eq!(*lock.read(), 42);
        });
    }
}
