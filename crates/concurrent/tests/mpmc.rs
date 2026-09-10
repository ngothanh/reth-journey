#[cfg(loom)]
mod tests {
    use concurrent::MpmcRing;
    use loom::sync::Arc;

    #[test]
    fn one_producer_one_consumer() {
        loom::model(|| {
            let ring = Arc::new(MpmcRing::with_capacity(2));

            let clone = ring.clone();
            let handle = loom::thread::spawn(move || {
                assert!(clone.try_push(1).is_ok());
                assert!(clone.try_push(2).is_ok());
            });

            let mut res = Vec::new();
            if let Some(v) = ring.try_pop() {
                res.push(v);
            };
            if let Some(v) = ring.try_pop() {
                res.push(v);
            };
            handle.join().unwrap();
            while let Some(v) = ring.try_pop() {
                res.push(v);
            }

            assert_eq!(res, vec![1, 2]);
        });
    }

    #[test]
    fn two_producer_one_consumer() {
        loom::model(|| {
            let ring = Arc::new(MpmcRing::with_capacity(2));

            let one = ring.clone();
            let two = ring.clone();
            let handle1 = loom::thread::spawn(move || {
                assert!(one.try_push(1).is_ok());
            });
            let handle2 = loom::thread::spawn(move || {
                assert!(two.try_push(2).is_ok());
            });
            handle1.join().unwrap();
            handle2.join().unwrap();
            let mut res = Vec::new();
            while let Some(v) = ring.try_pop() {
                res.push(v);
            }

            res.sort();
            assert_eq!(res, vec![1, 2]);
        });
    }

    // Consumer<->consumer axis: two full slots, two consumers race to drain.
    // Invariant: together they recover exactly {1, 2} — no duplicate (both
    // claimed the same slot), no loss (a spurious `None` while data was present).
    #[test]
    fn two_consumer_one_producer() {
        loom::model(|| {
            let ring = Arc::new(MpmcRing::with_capacity(2));
            // Seed both slots sequentially before the race.
            assert!(ring.try_push(1).is_ok());
            assert!(ring.try_push(2).is_ok());

            let one = ring.clone();
            let two = ring.clone();
            let handle1 = loom::thread::spawn(move || one.try_pop());
            let handle2 = loom::thread::spawn(move || two.try_pop());

            let mut res: Vec<usize> = [handle1.join().unwrap(), handle2.join().unwrap()]
                .into_iter()
                .flatten()
                .collect();

            res.sort();
            assert_eq!(res, vec![1, 2]);
        });
    }
}
