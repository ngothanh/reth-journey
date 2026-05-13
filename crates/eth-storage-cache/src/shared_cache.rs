//! `SharedAccountCache` — multi-threaded, shared-mutable account cache.
//!
//! The multi-threaded counterpart of [`LocalAccountCache`](crate::LocalAccountCache).
//! Identical API; only the synchronization primitives change:
//!
//! - `Rc<T>` → `Arc<T>` (atomic ref count, `Send + Sync`).
//! - `RefCell<T>` → `parking_lot::RwLock<T>` (real lock, deadlocks rather than
//!   panicking on misuse; no poisoning like `std::sync::RwLock`).
//! - `.borrow()` / `.borrow_mut()` → `.read()` / `.write()` (no `.unwrap()`).
//!
//! See `notes/05_smart_pointers.md` for the full diff and trade-offs.

extern crate alloc;

use crate::Account;
use alloc::sync::Arc;
use eth_primitives::Address;
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct SharedAccountCache {
    map: HashMap<Address, Arc<RwLock<Account>>>,
}

impl SharedAccountCache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Return the cached account for `addr`, loading via `f` on cache miss.
    pub fn get_or_load<F>(&mut self, addr: Address, f: F) -> Arc<RwLock<Account>>
    where
        F: FnOnce(Address) -> Account,
    {
        if let Some(account) = self.map.get(&addr) {
            Arc::clone(account)
        } else {
            let account = Arc::new(RwLock::new(f(addr)));
            self.map.insert(addr, Arc::clone(&account));
            account
        }
    }

    /// Drain the cache, returning all entries.
    ///
    /// Panics if any account is still held outside the cache (i.e. an
    /// outstanding `Arc` clone exists). Same semantics as the local variant.
    pub fn commit(&mut self) -> Vec<(Address, Account)> {
        self.map
            .drain()
            .map(|(addr, arc)| {
                let lock = Arc::try_unwrap(arc)
                    .expect("commit while account is still held by another Arc");
                (addr, lock.into_inner())
            })
            .collect()
    }
}

impl Default for SharedAccountCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eth_primitives::U256;
    use std::sync::Mutex;
    use std::thread;

    fn addr(last_byte: u8) -> Address {
        Address::with_last_byte(last_byte)
    }

    #[test]
    fn get_or_load_caches_first_call() {
        let mut cache = SharedAccountCache::new();
        let mut count = 0;

        let _a = cache.get_or_load(addr(1), |_| {
            count += 1;
            Account::default()
        });
        let _b = cache.get_or_load(addr(1), |_| {
            count += 1;
            Account::default()
        });
        assert_eq!(count, 1);
    }

    #[test]
    fn write_then_read() {
        let mut cache = SharedAccountCache::new();
        let entry = cache.get_or_load(addr(1), |_| Account::default());
        entry.write().balance = U256::from(2);
        assert_eq!(entry.read().balance, U256::from(2));
    }

    #[test]
    fn second_write_blocks_when_first_held() {
        // parking_lot's RwLock cannot panic on double-write like RefCell does.
        // Instead, it blocks. Use `try_write` to observe the blocking without
        // hanging the test.
        let mut cache = SharedAccountCache::new();
        let entry = cache.get_or_load(addr(1), |_| Account::default());
        let _first = entry.write();
        let second = entry.try_write();
        assert!(
            second.is_none(),
            "second write should fail while the first is held"
        );
    }

    #[test]
    fn shared_across_threads() {
        // The whole reason this cache exists: shareable across threads.
        // Wrap the cache itself in Mutex so threads can serialize access to it,
        // then each thread loads a distinct address.
        let cache = Arc::new(Mutex::new(SharedAccountCache::new()));

        let handles: Vec<_> = (1..=4)
            .map(|i| {
                let cache = Arc::clone(&cache);
                thread::spawn(move || {
                    let mut guard = cache.lock().unwrap();
                    let entry = guard.get_or_load(addr(i), |_| Account::default());
                    entry.write().balance = U256::from(u64::from(i));
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let mut guard = cache.lock().unwrap();
        let committed = guard.commit();
        assert_eq!(committed.len(), 4);
    }

    #[test]
    fn commit_drains_and_returns_entries() {
        let mut cache = SharedAccountCache::new();
        {
            let a = cache.get_or_load(addr(1), |_| Account::default());
            a.write().balance = U256::from(1);
        }
        {
            let b = cache.get_or_load(addr(2), |_| Account::default());
            b.write().balance = U256::from(2);
        }
        let mut committed = cache.commit();
        committed.sort_by_key(|(a, _)| a.0);
        assert_eq!(committed.len(), 2);
        assert_eq!(committed[0].0, addr(1));
        assert_eq!(committed[1].0, addr(2));
    }

    #[test]
    fn commit_twice_returns_empty_second_time() {
        let mut cache = SharedAccountCache::new();
        cache.get_or_load(addr(1), |_| Account::default());
        let first = cache.commit();
        let second = cache.commit();
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 0);
    }

    #[test]
    fn cache_reloads_after_commit() {
        let mut cache = SharedAccountCache::new();
        let mut count = 0;

        cache.get_or_load(addr(1), |_| {
            count += 1;
            Account::default()
        });
        cache.commit();
        cache.get_or_load(addr(1), |_| {
            count += 1;
            Account::default()
        });
        assert_eq!(count, 2);
    }

    #[test]
    #[should_panic(expected = "commit while account is still held")]
    fn commit_while_holding_account_panics() {
        let mut cache = SharedAccountCache::new();
        let _outstanding = cache.get_or_load(addr(1), |_| Account::default());
        cache.commit();
    }
}
