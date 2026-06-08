#[cfg(loom)]
mod loom_tests {
    use concurrent::AtomicCell;
    use loom::sync::Arc;

    /// Loom model: a writer and a reader race on the spinlock slow path. Under
    /// every interleaving the reader must observe a *whole* value — either the
    /// old `[0; 16]` or the new `[1; 16]`, never a mix of bytes from both. All
    /// 16 bytes are equal on purpose: a torn read (some bytes from each store)
    /// shows up as a value that is neither all-0 nor all-1.
    ///
    /// `[u8; 16]` is 16 bytes, so it can never hit the 8-byte `AtomicU64` fast
    /// path — this exercises the `AtomicBool` spinlock + `UnsafeCell` slow path.
    #[test]
    fn loom_atomic_cell_slow_path_no_torn_write() {
        loom::model(|| {
            let cell = Arc::new(AtomicCell::new([0u8; 16]));
            let writer_cell = Arc::clone(&cell);

            let writer = loom::thread::spawn(move || {
                writer_cell.store([1u8; 16]);
            });

            let observed = cell.load();
            assert!(
                observed == [0u8; 16] || observed == [1u8; 16],
                "torn read observed: {observed:?}"
            );

            writer.join().unwrap();
        });
    }
}
