//! Concurrency drill #2 — sequenced data publication (paired Release/Acquire).
//!
//! Normal run (single-thread smoke):  cargo test -p concurrent --test cell_sender
//! Loom run (the drill):
//!   RUST_BACKTRACE=1 RUSTFLAGS="--cfg loom" cargo test -p concurrent --test cell_sender -- --nocapture

#[derive(Clone, Copy, PartialEq, Debug)]
struct Payload {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

const SENTINEL: Payload = Payload {
    a: 0xAAAA,
    b: 0xBBBB,
    c: 0xCCCC,
    d: 0xDDDD,
};

// Your single-thread smoke test. Excluded under `--cfg loom` (loom atomics panic outside
// `loom::model`, so it can't run there).
#[cfg(not(loom))]
mod single_thread_tests {
    use super::{Payload, SENTINEL};
    use concurrent::Channel;

    #[test]
    fn single_thread() {
        let (sender, receiver) = Channel::<Payload>::new();
        if sender.write(SENTINEL).is_ok() {
            let x = receiver.try_recv().unwrap();
            assert_eq!(*x, SENTINEL);
        }
    }
}

// The loom test. Only exists under `--cfg loom`.
//
// YOUR JOB (the drill):
//   1. Write the assertion below — same shape as your OnceFlag loom test: a consumer that observes
//      `Some(v)` must see the WHOLE sentinel (all 4 fields), never a mix. All-or-nothing.
//   2. Run the rep: your impl already has Release/Acquire, so this passes GREEN. To get the drill,
//      temporarily set BOTH the `state.store(DONE, ...)` in `Sender::write` and the `state.load(...)`
//      in `Receiver::try_recv` to `Ordering::Relaxed`, re-run, and confirm loom FIRES the data race
//      (`Concurrent read and write accesses`). Read the trace. Then restore Release/Acquire and
//      watch it go back to GREEN. That red -> green is the muscle this drill exists to build.
#[cfg(loom)]
mod loom_tests {
    use super::{Payload, SENTINEL};
    use concurrent::Channel;
    use loom::thread;

    #[test]
    fn recv_is_all_or_nothing() {
        loom::model(|| {
            let (tx, rx) = Channel::<Payload>::new();

            // producer thread: publish the sentinel exactly once
            let producer = thread::spawn(move || {
                let _ = tx.write(SENTINEL);
            });

            // consumer (this thread): try to receive, concurrently with the producer.
            if let Some(_v) = rx.try_recv() {
                assert_eq!(*_v, SENTINEL, "saw SET but a stale/partial payload");
            }

            producer.join().unwrap();
        });
    }
}
