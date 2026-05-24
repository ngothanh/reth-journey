//! `ChainHead` — SeqLock-protected canonical chain tip.
//!
//! Stub file. See plan/W004.md Tuesday for the build exercise.
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
use core::sync::atomic::AtomicU64;
use eth_primitives::B256;
use std::sync::atomic::Ordering;

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

        self.sequence.fetch_add(1, Ordering::Release);
        self.inner.hash0.store(h0, Ordering::Relaxed);
        self.inner.hash1.store(h1, Ordering::Relaxed);
        self.inner.hash2.store(h2, Ordering::Relaxed);
        self.inner.hash3.store(h3, Ordering::Relaxed);
        self.inner.number.store(number, Ordering::Relaxed);
        self.sequence.fetch_add(1, Ordering::Release);
    }

    pub fn load(&self) -> (B256, u64) {
        loop {
            let i = self.sequence.load(Ordering::Acquire);
            if i & 1 != 0 {
                core::hint::spin_loop();
                continue;
            }
            break;
        }
        //TODO write loom test to detect error
        let h0 = self.inner.hash0.load(Ordering::Relaxed);
        let h1 = self.inner.hash1.load(Ordering::Relaxed);
        let h2 = self.inner.hash2.load(Ordering::Relaxed);
        let h3 = self.inner.hash3.load(Ordering::Relaxed);
        let n = self.inner.number.load(Ordering::Relaxed);

        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(&h0.to_ne_bytes());
        bytes[8..16].copy_from_slice(&h1.to_ne_bytes());
        bytes[16..24].copy_from_slice(&h2.to_ne_bytes());
        bytes[24..32].copy_from_slice(&h3.to_ne_bytes());
        (B256::from(bytes), n)
    }
}
