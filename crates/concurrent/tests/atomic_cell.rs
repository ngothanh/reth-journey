// Fast-path / Pod acceptance tests. Compiled as a *downstream* crate (an
// integration test sees only `concurrent`'s public API), so these double as
// proof that `AtomicCell` and `Pod` are usable from outside the crate.
//
// These are also the bodies miri runs for R5: `cargo +nightly miri test
// --test atomic_cell` exercises the `u64` and `*const u64` fast paths and must
// report zero UB.
//
// NOTE: the plan checklist names these methods `store_fast`/`load_fast`; the
// shipped API unifies them into `store`/`load`, which pick the fast path via a
// `const` size/align check now gated on `T: Pod`. Tests use the real API.
#[cfg(not(loom))]
mod fast_path_tests {
    use concurrent::AtomicCell;
    use core::ptr;

    // R3: `u64` is size 8 / align 8, so it takes the `AtomicU64` fast path and
    // round-trips. The `T: Pod` bound is what makes that transmute sound.
    #[test]
    fn atomic_cell_u64_fast_path_compiles_and_works() {
        let cell = AtomicCell::new(42u64);
        cell.store(7);
        assert_eq!(cell.load(), 7);
    }

    // R2: raw pointers are `Pod` (size 8 / align 8 → fast path). Null round-trips
    // with no provenance concerns; a real address round-trips byte-for-byte.
    #[test]
    fn atomic_cell_pod_pointer_compiles() {
        let cell = AtomicCell::new(ptr::null::<u64>());
        assert!(cell.load().is_null());

        let x = 9u64;
        let p: *const u64 = &x;
        cell.store(p);
        assert_eq!(cell.load() as usize, p as usize);
    }

    // R2: the `[T: Pod; N]` blanket impl makes `[u8; 8]` `Pod`. (align 1, so this
    // actually rides the slow path — the point is that the type is *accepted*.)
    #[test]
    fn atomic_cell_pod_array_compiles() {
        let cell = AtomicCell::new([0u8; 8]);
        cell.store([1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(cell.load(), [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    // R1: a downstream type implementing the re-exported `concurrent::Pod` can
    // use the fast path. `#[repr(transparent)]` over `u64` → size 8, align 8, no
    // padding, every bit pattern valid, so the unsafe impl upholds the contract.
    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Debug)]
    struct MyU64Wrapper(u64);

    // SAFETY: transparent newtype over `u64` — no padding, every 8-byte pattern
    // is a valid value. Meets `Pod`'s contract.
    unsafe impl concurrent::Pod for MyU64Wrapper {}

    #[test]
    fn atomic_cell_pod_downstream_impl_via_reexport() {
        let cell = AtomicCell::new(MyU64Wrapper(1));
        cell.store(MyU64Wrapper(99));
        assert_eq!(cell.load(), MyU64Wrapper(99));
    }
}

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
