#[cfg(loom)]
mod loom_tests {
    use concurrent::OnceFlag;
    use loom::sync::Arc;

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
    #[test]
    fn test() {
        loom::model(|| {
            let flag = Arc::new(OnceFlag::new());
            let writer_flag = Arc::clone(&flag);

            let writer = loom::thread::spawn(move || {
                let _ = writer_flag.set(SENTINEL);
            });

            if let Some(v) = flag.get() {
                // (4) the reader (main thread)
                assert_eq!(*v, SENTINEL, "saw SET but a stale/partial payload");
            }

            writer.join().unwrap();
        })
    }
}
