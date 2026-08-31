#![cfg(loom)]

use concurrent::SeqLock;
use loom::sync::Arc;

type Payload = [usize; 2];

const A: Payload = [1, 1];
const B: Payload = [2, 2];
const C: Payload = [3, 3];

#[test]
fn one_writer_one_reader_never_tears() {
    loom::model(|| {
        let lock = Arc::new(SeqLock::new(A));

        let writer = {
            let lock = Arc::clone(&lock);
            loom::thread::spawn(move || lock.store(B))
        };

        let seen = lock.load();
        assert!(seen == A || seen == B, "torn read: {seen:?}");

        writer.join().unwrap();
        assert_eq!(lock.load(), B);
    });
}

/// Two writers contend for the sequence. The CAS in `store` must serialise
/// them, so the reader still only ever sees a whole payload -- and once both
/// have finished the payload is one of the two, not a mix.
#[test]
fn two_writers_serialise() {
    loom::model(|| {
        let lock = Arc::new(SeqLock::new(A));

        let w1 = {
            let lock = Arc::clone(&lock);
            loom::thread::spawn(move || lock.store(B))
        };
        let w2 = {
            let lock = Arc::clone(&lock);
            loom::thread::spawn(move || lock.store(C))
        };

        w1.join().unwrap();
        w2.join().unwrap();

        let seen = lock.load();
        assert!(seen == B || seen == C, "torn read: {seen:?}");
    });
}
