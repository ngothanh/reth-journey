//! `MutexCache` — multi-threaded state cache behind a single coarse-grained lock.
//!
//! Whereas [`SharedAccountCache`](crate::SharedAccountCache) gives each
//! account its own `RwLock` (fine-grained), `MutexCache` puts *every* state
//! map — accounts, code, storage, block hashes — behind one `Mutex`.
//!
//! # Lesson
//!
//! - One lock, one allocation. Simpler than per-entry `Arc<RwLock<_>>`.
//! - Atomic across all four maps: anything you do under the guard sees a
//!   consistent snapshot. Fine-grained locks can't offer that.
//! - Every caller serializes on the same lock — no concurrent reads. Fine if
//!   contention is low; bad if many readers fan in.
//! - **Never hold the guard across slow work** (DB I/O, syscalls). Cache
//!   misses bubble up as `NotFound*` errors so the caller can release the
//!   lock, hit the backing store, then call `insert_*` to populate.

use crate::database::{StateCache, StateCacheError};
use crate::Account;
use eth_primitives::{Address, Bytes, B256, U256};
use parking_lot::Mutex;
use std::collections::HashMap;

pub struct MutexCache {
    inner: Mutex<MutexCacheInner>,
}

struct MutexCacheInner {
    accounts: HashMap<Address, Account>,
    code: HashMap<B256, Bytes>,
    storage: HashMap<(Address, U256), U256>,
    block_hashes: HashMap<u64, B256>,
}

impl MutexCacheInner {
    fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            code: HashMap::new(),
            storage: HashMap::new(),
            block_hashes: HashMap::new(),
        }
    }
}

impl MutexCache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MutexCacheInner::new()),
        }
    }

    pub fn insert_account(&self, address: Address, account: Account) {
        self.inner.lock().accounts.insert(address, account);
    }

    pub fn insert_code(&self, code_hash: B256, code: Bytes) {
        self.inner.lock().code.insert(code_hash, code);
    }

    pub fn insert_storage(&self, address: Address, key: U256, value: U256) {
        self.inner.lock().storage.insert((address, key), value);
    }

    pub fn insert_block_hash(&self, number: u64, hash: B256) {
        self.inner.lock().block_hashes.insert(number, hash);
    }
}

impl Default for MutexCache {
    fn default() -> Self {
        Self::new()
    }
}

impl StateCache for MutexCache {
    type Error = StateCacheError;

    fn basic(&mut self, address: Address) -> Result<Option<Account>, Self::Error> {
        Ok(self.inner.lock().accounts.get(&address).cloned())
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytes, Self::Error> {
        self.inner
            .lock()
            .code
            .get(&code_hash)
            .cloned()
            .ok_or(StateCacheError::CodeNotFound(code_hash))
    }

    fn storage(&mut self, address: Address, key: U256) -> Result<U256, Self::Error> {
        self.inner
            .lock()
            .storage
            .get(&(address, key))
            .copied()
            .ok_or(StateCacheError::StorageNotFound(address, key))
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.inner
            .lock()
            .block_hashes
            .get(&number)
            .copied()
            .ok_or(StateCacheError::BlockHashNotFound(number))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(last_byte: u8) -> Address {
        Address::with_last_byte(last_byte)
    }

    fn hash(last_byte: u8) -> B256 {
        let mut bytes = [0u8; 32];
        bytes[31] = last_byte;
        B256::from(bytes)
    }

    #[test]
    fn basic_returns_none_on_miss_and_some_on_hit() {
        let mut cache = MutexCache::new();
        assert_eq!(cache.basic(addr(1)).unwrap(), None);

        let mut acct = Account::default();
        acct.balance = U256::from(42u64);
        cache.insert_account(addr(1), acct.clone());

        assert_eq!(cache.basic(addr(1)).unwrap(), Some(acct));
    }

    #[test]
    fn code_by_hash_returns_not_found_on_miss() {
        let mut cache = MutexCache::new();
        let h = hash(7);
        match cache.code_by_hash(h) {
            Err(StateCacheError::CodeNotFound(got)) => assert_eq!(got, h),
            other => panic!("expected CodeNotFound, got {other:?}"),
        }

        cache.insert_code(h, Bytes::from_static(b"\x60\x00"));
        assert_eq!(cache.code_by_hash(h).unwrap().as_ref(), &b"\x60\x00"[..]);
    }

    #[test]
    fn storage_returns_not_found_on_miss() {
        let mut cache = MutexCache::new();
        let a = addr(1);
        let slot = U256::from(5u64);

        match cache.storage(a, slot) {
            Err(StateCacheError::StorageNotFound(got_a, got_s)) => {
                assert_eq!(got_a, a);
                assert_eq!(got_s, slot);
            }
            other => panic!("expected StorageNotFound, got {other:?}"),
        }

        cache.insert_storage(a, slot, U256::from(99u64));
        assert_eq!(cache.storage(a, slot).unwrap(), U256::from(99u64));
    }

    #[test]
    fn block_hash_returns_not_found_on_miss() {
        let mut cache = MutexCache::new();
        match cache.block_hash(100) {
            Err(StateCacheError::BlockHashNotFound(n)) => assert_eq!(n, 100),
            other => panic!("expected BlockHashNotFound, got {other:?}"),
        }

        let h = hash(1);
        cache.insert_block_hash(100, h);
        assert_eq!(cache.block_hash(100).unwrap(), h);
    }

    #[test]
    fn inserts_take_shared_self_so_arc_works() {
        // The `insert_*` helpers take `&self` (interior mutability via the
        // Mutex), which is exactly what makes the cache shareable behind an
        // `Arc` without an extra outer lock.
        extern crate alloc;
        use alloc::sync::Arc;
        use std::thread;

        let cache = Arc::new(MutexCache::new());
        let handles: Vec<_> = (1u8..=8)
            .map(|i| {
                let cache = Arc::clone(&cache);
                thread::spawn(move || {
                    cache.insert_account(addr(i), Account::default());
                    cache.insert_storage(addr(i), U256::from(i), U256::from(u64::from(i)));
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }

        // Read back via the trait — requires &mut, so unwrap the Arc.
        let mut cache = Arc::try_unwrap(cache).ok().expect("no outstanding clones");
        for i in 1u8..=8 {
            assert!(cache.basic(addr(i)).unwrap().is_some());
            assert_eq!(
                cache.storage(addr(i), U256::from(i)).unwrap(),
                U256::from(u64::from(i))
            );
        }
    }
}
