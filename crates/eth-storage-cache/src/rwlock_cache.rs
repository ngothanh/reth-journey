//! `RwLockCache` — multi-threaded state cache behind a single `RwLock`.
//!
//! Same coarse-grained shape as [`MutexCache`](crate::MutexCache), with one
//! change: the inner lock is a `parking_lot::RwLock` instead of a `Mutex`.
//! Reads take `.read()` (shared); writes take `.write()` (exclusive).
//!
//! # Lesson
//!
//! - Many concurrent readers are fine — they don't block each other.
//! - A writer still excludes everyone, including readers.
//! - The trait signature is `&mut self`, but that's the *wrapper*'s mutability,
//!   not the inner lock's. The read-path methods call `.read()` on the inner
//!   `RwLock` and don't actually mutate anything in the wrapper. The `&mut`
//!   is an inherited constraint from revm's `Database` shape; it doesn't
//!   force us to use `.write()`.
//! - vs `MutexCache`: better when reads dominate; equivalent (or slightly
//!   worse, due to bookkeeping overhead) when writes dominate.

use crate::database::{StateCache, StateCacheError};
use crate::Account;
use eth_primitives::{Address, Bytes, B256, U256};
use parking_lot::RwLock;
use std::collections::HashMap;
use tracing::instrument;

pub struct RwLockCache {
    inner: RwLock<RwLockCacheInner>,
}

struct RwLockCacheInner {
    accounts: HashMap<Address, Account>,
    storage: HashMap<(Address, U256), U256>,
    code: HashMap<B256, Bytes>,
    block_hashes: HashMap<u64, B256>,
}

impl RwLockCacheInner {
    fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            code: HashMap::new(),
            storage: HashMap::new(),
            block_hashes: HashMap::new(),
        }
    }
}

impl RwLockCache {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(RwLockCacheInner::new()),
        }
    }

    pub fn insert_account(&self, address: Address, account: Account) {
        self.inner.write().accounts.insert(address, account);
    }

    pub fn insert_code(&self, code_hash: B256, code: Bytes) {
        self.inner.write().code.insert(code_hash, code);
    }

    pub fn insert_storage(&self, address: Address, key: U256, value: U256) {
        self.inner.write().storage.insert((address, key), value);
    }

    pub fn insert_block_hash(&self, number: u64, hash: B256) {
        self.inner.write().block_hashes.insert(number, hash);
    }
}

impl Default for RwLockCache {
    fn default() -> Self {
        Self::new()
    }
}

impl StateCache for RwLockCache {
    type Error = StateCacheError;

    #[instrument(level = "trace", skip(self))]
    fn basic(&mut self, address: Address) -> Result<Option<Account>, Self::Error> {
        Ok(self.inner.read().accounts.get(&address).cloned())
    }

    #[instrument(level = "trace", skip(self))]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytes, Self::Error> {
        self.inner
            .read()
            .code
            .get(&code_hash)
            .cloned()
            .ok_or(StateCacheError::CodeNotFound(code_hash))
    }

    #[instrument(level = "trace", skip(self))]
    fn storage(&mut self, address: Address, key: U256) -> Result<U256, Self::Error> {
        self.inner
            .read()
            .storage
            .get(&(address, key))
            .copied()
            .ok_or(StateCacheError::StorageNotFound(address, key))
    }

    #[instrument(level = "trace", skip(self))]
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.inner
            .read()
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
        let mut cache = RwLockCache::new();
        assert_eq!(cache.basic(addr(1)).unwrap(), None);

        let acct = Account {
            balance: U256::from(42u64),
            ..Account::default()
        };
        cache.insert_account(addr(1), acct.clone());

        assert_eq!(cache.basic(addr(1)).unwrap(), Some(acct));
    }

    #[test]
    fn code_by_hash_returns_not_found_on_miss() {
        let mut cache = RwLockCache::new();
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
        let mut cache = RwLockCache::new();
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
        let mut cache = RwLockCache::new();
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
        // Population is done via `&self` writes (the inner RwLock provides
        // interior mutability), so the cache is shareable behind an `Arc`
        // without an outer lock.
        extern crate alloc;
        use alloc::sync::Arc;
        use std::thread;

        let cache = Arc::new(RwLockCache::new());
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

        let mut cache = Arc::try_unwrap(cache).ok().expect("no outstanding clones");
        for i in 1u8..=8 {
            assert!(cache.basic(addr(i)).unwrap().is_some());
            assert_eq!(
                cache.storage(addr(i), U256::from(i)).unwrap(),
                U256::from(u64::from(i))
            );
        }
    }

    #[test]
    fn concurrent_readers_do_not_block_each_other() {
        // The whole reason to pick RwLock over Mutex: many readers can hold
        // the lock simultaneously. We verify by taking a read guard, then
        // confirming `try_read()` succeeds while it's still held.
        let cache = RwLockCache::new();
        cache.insert_account(addr(1), Account::default());

        let g1 = cache.inner.read();
        let g2 = cache.inner.try_read();
        assert!(g2.is_some(), "second reader should not be blocked");
        drop(g1);
    }
}
