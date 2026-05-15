use crate::{Account, StateCache, StateCacheError};
use eth_primitives::{Address, Bytes, B256, U256};
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct ShardedCache<const N: usize> {
    shards: [RwLock<ShardedCacheInner>; N],
    codes: RwLock<HashMap<B256, Bytes>>,
    block_hashes: RwLock<HashMap<u64, B256>>,
}

struct ShardedCacheInner {
    accounts: HashMap<Address, Account>,
    storage: HashMap<(Address, U256), U256>,
}

impl<const N: usize> StateCache for ShardedCache<N> {
    type Error = StateCacheError;

    fn basic(&mut self, address: Address) -> Result<Option<Account>, Self::Error> {
        Ok(self
            .shard_for(&address)
            .read()
            .accounts
            .get(&address)
            .cloned())
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytes, Self::Error> {
        self.codes
            .read()
            .get(&code_hash)
            .cloned()
            .ok_or(StateCacheError::CodeNotFound(code_hash))
    }

    fn storage(&mut self, address: Address, key: U256) -> Result<U256, Self::Error> {
        self.shard_for(&address)
            .read()
            .storage
            .get(&(address, key))
            .copied()
            .ok_or(StateCacheError::StorageNotFound(address, key))
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.block_hashes
            .read()
            .get(&number)
            .cloned()
            .ok_or(StateCacheError::BlockHashNotFound(number))
    }
}

impl<const N: usize> ShardedCache<N> {
    pub fn new() -> Self {
        Self {
            shards: core::array::from_fn(|_| {
                RwLock::new(ShardedCacheInner {
                    accounts: HashMap::new(),
                    storage: Default::default(),
                })
            }),
            block_hashes: RwLock::new(Default::default()),
            codes: RwLock::new(Default::default()),
        }
    }

    /// Insert or replace an account.
    pub fn insert(&self, addr: Address, account: Account) {
        self.shard_for(&addr).write().accounts.insert(addr, account);
    }

    /// Number of accounts in shard `idx`. Test/diagnostic helper.
    pub fn shard_len(&self, idx: usize) -> usize {
        self.shards[idx].read().accounts.len()
    }

    /// Total accounts across all shards.
    pub fn len(&self) -> usize {
        self.shards.iter().map(|s| s.read().accounts.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn shard_for(&self, addr: &Address) -> &RwLock<ShardedCacheInner> {
        let idx = addr.0[0] as usize % N;
        &self.shards[idx]
    }
}

impl<const N: usize> Default for ShardedCache<N> {
    fn default() -> Self {
        ShardedCache::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eth_primitives::FixedBytes;
    use std::sync::Arc;
    use std::thread;

    fn addr_with_first_byte(first: u8, last: u8) -> Address {
        let mut bytes = [0u8; 20];
        bytes[0] = first;
        bytes[19] = last;
        FixedBytes(bytes)
    }

    fn account_with_balance(balance: u64) -> Account {
        Account {
            balance: U256::from(balance),
            ..Account::default()
        }
    }

    #[test]
    fn insert_and_basic_round_trip() {
        let mut cache: ShardedCache<16> = ShardedCache::new();
        let a = addr_with_first_byte(0x05, 0x01);

        cache.insert(a, account_with_balance(100));
        let got = cache.basic(a).unwrap().expect("account present");
        assert_eq!(got.balance, U256::from(100u64));
    }

    #[test]
    fn basic_returns_none_for_missing() {
        let mut cache: ShardedCache<16> = ShardedCache::new();
        let missing = addr_with_first_byte(0xff, 0xff);
        assert!(cache.basic(missing).unwrap().is_none());
    }

    #[test]
    fn distinct_first_bytes_route_to_distinct_shards() {
        let cache: ShardedCache<16> = ShardedCache::new();
        cache.insert(addr_with_first_byte(0x00, 0x01), account_with_balance(1));
        cache.insert(addr_with_first_byte(0x01, 0x01), account_with_balance(2));
        cache.insert(addr_with_first_byte(0x02, 0x01), account_with_balance(3));

        assert_eq!(cache.shard_len(0), 1);
        assert_eq!(cache.shard_len(1), 1);
        assert_eq!(cache.shard_len(2), 1);
    }

    #[test]
    fn first_byte_mod_n_collisions_share_a_shard() {
        let cache: ShardedCache<16> = ShardedCache::new();
        // 0x00, 0x10, 0x20 all collide on shard 0 (mod 16).
        cache.insert(addr_with_first_byte(0x00, 0x01), account_with_balance(1));
        cache.insert(addr_with_first_byte(0x10, 0x01), account_with_balance(2));
        cache.insert(addr_with_first_byte(0x20, 0x01), account_with_balance(3));

        assert_eq!(cache.shard_len(0), 3);
        for i in 1..16 {
            assert_eq!(cache.shard_len(i), 0);
        }
    }

    #[test]
    fn unsupported_methods_return_not_found() {
        let mut cache: ShardedCache<16> = ShardedCache::new();
        let addr = addr_with_first_byte(0x05, 0x01);
        let hash = B256::default();

        assert!(matches!(
            cache.code_by_hash(hash),
            Err(StateCacheError::CodeNotFound(_))
        ));
        assert!(matches!(
            cache.storage(addr, U256::ZERO),
            Err(StateCacheError::StorageNotFound(_, _))
        ));
        assert!(matches!(
            cache.block_hash(42),
            Err(StateCacheError::BlockHashNotFound(_))
        ));
    }

    #[test]
    fn concurrent_writers_to_different_shards() {
        // The whole point of sharding: writes to different shards don't
        // contend. 4 threads, distinct first-byte ranges, all writes land.
        let cache: Arc<ShardedCache<16>> = Arc::new(ShardedCache::new());

        let handles: Vec<_> = (0..4u8)
            .map(|i| {
                let cache = Arc::clone(&cache);
                thread::spawn(move || {
                    for j in 0..100u8 {
                        let addr = addr_with_first_byte(i, j);
                        cache.insert(addr, account_with_balance(u64::from(j)));
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(cache.len(), 400);
        for i in 0..4 {
            assert_eq!(cache.shard_len(i), 100);
        }
    }

    #[test]
    fn cache_is_send_and_sync() {
        // Auto-trait propagation: parking_lot::RwLock<HashMap<...>> is
        // Send + Sync, so the wrapper is too.
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ShardedCache<16>>();
        assert_sync::<ShardedCache<16>>();
    }
}
