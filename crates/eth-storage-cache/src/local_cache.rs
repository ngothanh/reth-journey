//! `LocalAccountCache` — single-threaded, shared-mutable account cache.
//!
//! Maps each `Address` to an `Rc<RefCell<Account>>` so multiple holders can
//! share read access and any one of them can mutate via `.borrow_mut()`.
//! The cache stages changes; `commit` drains the cache and returns the
//! modified accounts for the caller to persist (or discard).
//!
//! # Lesson
//!
//! - `Rc` lets us share ownership of an `Account` across multiple holders
//!   without copying — clones bump a reference count.
//! - `RefCell` allows interior mutability through an `Rc` (which is otherwise
//!   immutable). The borrow check moves from compile-time to runtime.
//! - Double-`borrow_mut()` on the same `RefCell` panics at runtime. This is
//!   the `RefCell` contract: it enforces "exclusive write or shared read,"
//!   just like the borrow checker — but at runtime, because shared ownership
//!   prevents static checking.

use crate::Account;
use eth_primitives::Address;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct LocalAccountCache {
    map: HashMap<Address, Rc<RefCell<Account>>>,
}

impl LocalAccountCache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Return the cached account for `addr`, loading via `f` on cache miss.
    pub fn get_or_load<F>(&mut self, addr: Address, f: F) -> Rc<RefCell<Account>>
    where
        F: FnOnce(Address) -> Account,
    {
        if let Some(account) = self.map.get(&addr) {
            account.clone()
        } else {
            let account = Rc::new(RefCell::new(f(addr)));
            self.map.insert(addr, account.clone());
            account
        }
    }

    /// Drain the cache, returning all entries.
    ///
    /// Panics if any account is still held outside the cache (i.e. an
    /// outstanding `Rc` clone exists). That would indicate a holder is still
    /// modifying state mid-commit, which is a programmer error.
    pub fn commit(&mut self) -> Vec<(Address, Account)> {
        self.map
            .drain()
            .map(|(addr, rc)| {
                let cell =
                    Rc::try_unwrap(rc).expect("commit while account is still held by another Rc");
                (addr, cell.into_inner())
            })
            .collect()
    }
}

impl Default for LocalAccountCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eth_primitives::U256;

    fn addr(last_byte: u8) -> Address {
        Address::with_last_byte(last_byte)
    }

    #[test]
    fn get_or_load_caches_first_call() {
        let mut cache = LocalAccountCache::new();
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
    fn borrow_mut_then_modify() {
        let mut cache = LocalAccountCache::new();
        let entry = cache.get_or_load(addr(1), |_account| Account::default());
        entry.borrow_mut().balance = U256::from(2);
        assert_eq!(entry.borrow().balance, U256::from(2));
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn double_borrow_mut_panics() {
        let mut cache = LocalAccountCache::new();
        let entry = cache.get_or_load(addr(1), |_account| Account::default());
        let _r1 = entry.borrow_mut();
        let _r2 = entry.borrow_mut();
    }

    #[test]
    fn commit_drains_and_returns_entries() {
        let mut cache = LocalAccountCache::new();
        {
            let a = cache.get_or_load(addr(1), |_| Account::default());
            a.borrow_mut().balance = U256::from(1);
        }
        {
            let b = cache.get_or_load(addr(2), |_| Account::default());
            b.borrow_mut().balance = U256::from(2);
        }
        let mut committed = cache.commit();
        committed.sort_by_key(|(a, _)| a.0);
        assert_eq!(committed.len(), 2);
        assert_eq!(committed[0].0, addr(1));
        assert_eq!(committed[1].0, addr(2));
    }

    #[test]
    fn commit_twice_returns_empty_second_time() {
        let mut cache = LocalAccountCache::new();
        {
            let a = cache.get_or_load(addr(1), |_| Account::default());
            a.borrow_mut().balance = U256::from(1);
        }
        let _first = cache.commit();
        let _second = cache.commit();
        assert_eq!(_first.len(), 1);
        assert_eq!(_second.len(), 0);
    }

    #[test]
    fn cache_reloads_after_commit() {
        let mut cache = LocalAccountCache::new();
        let mut count = 0;

        {
            let _a = cache.get_or_load(addr(1), |_| {
                count += 1;
                Account::default()
            });
        }
        cache.commit();
        let _b = cache.get_or_load(addr(1), |_| {
            count += 1;
            Account::default()
        });
        assert_eq!(count, 2);
    }

    #[test]
    #[should_panic(expected = "commit while account is still held")]
    fn commit_while_holding_account_panics() {
        let mut cache = LocalAccountCache::new();
        let mut count = 0;

        let _a = cache.get_or_load(addr(1), |_| {
            count += 1;
            Account::default()
        });

        cache.commit();
    }
}
