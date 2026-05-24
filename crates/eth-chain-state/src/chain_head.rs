//! `ChainHead` — SeqLock-protected canonical chain tip.
//!
//! **Single-writer**: `store` must be called from one thread only (the engine
//! thread in reth). Multiple concurrent writers will corrupt the seq counter's
//! odd/even invariant — there is no runtime guard.
//!
//! **Many-reader**: `load` is lock-free; any number of threads may call it
//! concurrently with each other and with the single writer.
//!
//! Designed for read-heavy / rare-write traffic. Reader retries are bounded
//! but a write storm can still hurt read latency.

use concurrent::CachePadded;
use eth_primitives::B256;

mod sync {
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{fence, AtomicU64, Ordering};

    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{fence, AtomicU64, Ordering};

    /// Spin hint that also yields to loom's scheduler under `--cfg loom`.
    /// Production: cheap CPU pipeline hint (PAUSE / YIELD).
    /// Loom: `thread::yield_now()` so loom can explore other interleavings.
    #[inline]
    pub(super) fn spin_hint() {
        #[cfg(not(loom))]
        core::hint::spin_loop();
        #[cfg(loom)]
        loom::thread::yield_now();
    }
}
use self::sync::{fence, spin_hint, AtomicU64, Ordering};

pub struct ChainHead {
    inner: CachePadded<ChainHeadInner>,
    sequence: CachePadded<AtomicU64>,
}

struct ChainHeadInner {
    hash0: AtomicU64,
    hash1: AtomicU64,
    hash2: AtomicU64,
    hash3: AtomicU64,
    number: AtomicU64,
}

impl ChainHead {
    pub fn new(hash: B256, number: u64) -> Self {
        let bytes = &hash.0;
        let h0 = u64::from_ne_bytes(bytes[0..8].try_into().unwrap());
        let h1 = u64::from_ne_bytes(bytes[8..16].try_into().unwrap());
        let h2 = u64::from_ne_bytes(bytes[16..24].try_into().unwrap());
        let h3 = u64::from_ne_bytes(bytes[24..32].try_into().unwrap());

        Self {
            sequence: CachePadded::new(AtomicU64::new(0)), // even → stable initially
            inner: CachePadded::new(ChainHeadInner {
                hash0: AtomicU64::new(h0),
                hash1: AtomicU64::new(h1),
                hash2: AtomicU64::new(h2),
                hash3: AtomicU64::new(h3),
                number: AtomicU64::new(number),
            }),
        }
    }
    pub fn store(&self, head: (B256, u64)) {
        let (hash, number) = head;
        let bytes = &hash.0;
        let h0 = u64::from_ne_bytes(bytes[0..8].try_into().unwrap());
        let h1 = u64::from_ne_bytes(bytes[8..16].try_into().unwrap());
        let h2 = u64::from_ne_bytes(bytes[16..24].try_into().unwrap());
        let h3 = u64::from_ne_bytes(bytes[24..32].try_into().unwrap());

        self.sequence.fetch_add(1, Ordering::Release);   // even → odd
        // Release fence: pairs with the reader's `fence(Acquire)`.
        // When the reader's Relaxed payload load reads from one of the
        // Relaxed stores below, the two fences synchronize-with each other,
        // making the prior seq bump happen-before the reader's s2 load.
        // Without this fence, Relaxed-Relaxed has no cross-thread ordering
        // and loom finds a torn (hash, number).
        fence(Ordering::Release);
        self.inner.hash0.store(h0, Ordering::Relaxed);
        self.inner.hash1.store(h1, Ordering::Relaxed);
        self.inner.hash2.store(h2, Ordering::Relaxed);
        self.inner.hash3.store(h3, Ordering::Relaxed);
        self.inner.number.store(number, Ordering::Relaxed);
        self.sequence.fetch_add(1, Ordering::Release);   // odd → even
    }

    pub fn load(&self) -> (B256, u64) {
        loop {
            let s1 = self.sequence.load(Ordering::Acquire);
            if s1 & 1 != 0 {
                spin_hint();
                continue;
            }

            let h0 = self.inner.hash0.load(Ordering::Relaxed);
            let h1 = self.inner.hash1.load(Ordering::Relaxed);
            let h2 = self.inner.hash2.load(Ordering::Relaxed);
            let h3 = self.inner.hash3.load(Ordering::Relaxed);
            let n = self.inner.number.load(Ordering::Relaxed);

            // Pin the payload reads above to happen-before the s2 read below.
            // Acquire-on-load alone doesn't push prior loads forward — only
            // a fence does. Without this, loom finds a torn (hash, number).
            fence(Ordering::Acquire);

            let s2 = self.sequence.load(Ordering::Relaxed);

            if s1 == s2 {
                let mut bytes = [0u8; 32];
                bytes[0..8].copy_from_slice(&h0.to_ne_bytes());
                bytes[8..16].copy_from_slice(&h1.to_ne_bytes());
                bytes[16..24].copy_from_slice(&h2.to_ne_bytes());
                bytes[24..32].copy_from_slice(&h3.to_ne_bytes());
                return (B256::from(bytes), n);
            }
            spin_hint()
        }
    }
}

#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn no_torn_read() {
        loom::model(|| {
            // Initial: hash = [0; 32], number = 0.
            let head = Arc::new(ChainHead::new(B256::from([0u8; 32]), 0));

            let writer = head.clone();
            let writer_thread = thread::spawn(move || {
                // One distinct write — keep the model tiny.
                writer.store((B256::from([0xAA; 32]), 1));
            });

            let reader = head.clone();
            let reader_thread = thread::spawn(move || {
                let (h, n) = reader.load();

                // Invariant: the snapshot is one of the two stored pairs,
                // never a mix. A torn hash would have both 0x00 and 0xAA
                // bytes; a torn pair would have all-AA bytes with n=0
                // (or all-zero bytes with n=1).
                let all_zero = h.0.iter().all(|&b| b == 0);
                let all_aa = h.0.iter().all(|&b| b == 0xAA);

                assert!(all_zero || all_aa, "torn hash bytes: {:?}", h.0);
                if all_zero {
                    assert_eq!(n, 0, "torn pair: zero hash with number={n}");
                }
                if all_aa {
                    assert_eq!(n, 1, "torn pair: AA hash with number={n}");
                }
            });

            writer_thread.join().unwrap();
            reader_thread.join().unwrap();
        });
    }
}
