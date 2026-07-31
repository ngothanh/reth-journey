use crate::eviction::{EvictionPolicy, NoOpEviction};
use crate::{Account, StateCache, StateCacheError};
use eth_primitives::{Address, Bytes, B256, U256};
use parking_lot::RwLock;
use std::collections::HashMap;
use tracing::instrument;

pub struct ShardedCache<const N: usize, E: EvictionPolicy<Address>> {
    shards: [RwLock<ShardedCacheInner<E>>; N],
    max_per_shard: usize,
    codes: RwLock<HashMap<B256, Bytes>>,
    block_hashes: RwLock<HashMap<u64, B256>>,
}

struct ShardedCacheInner<E: EvictionPolicy<Address>> {
    accounts: HashMap<Address, Account>,
    storage: HashMap<(Address, U256), U256>,
    eviction_policy: E,
}

impl<const N: usize, E: EvictionPolicy<Address>> StateCache for ShardedCache<N, E> {
    type Error = StateCacheError;

    #[instrument(level = "trace", skip(self), fields(shard = (address.0[0] as usize) % N))]
    fn basic(&mut self, address: Address) -> Result<Option<Account>, Self::Error> {
        // Write lock: reads update LRU recency via `on_access`.
        // Trade-off: kills read concurrency on the shard, but keeps LRU correct.
        let mut inner = self.shard_for(&address).write();
        let got = inner.accounts.get(&address).cloned();
        if got.is_some() {
            inner.eviction_policy.on_access(&address);
        }
        Ok(got)
    }

    #[instrument(level = "trace", skip(self))]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytes, Self::Error> {
        self.codes
            .read()
            .get(&code_hash)
            .cloned()
            .ok_or(StateCacheError::CodeNotFound(code_hash))
    }

    #[instrument(level = "trace", skip(self), fields(shard = (address.0[0] as usize) % N))]
    fn storage(&mut self, address: Address, key: U256) -> Result<U256, Self::Error> {
        // Same write-lock-for-recency trade-off as `basic`.
        let mut inner = self.shard_for(&address).write();
        let got = inner.storage.get(&(address, key)).copied();
        if got.is_some() {
            inner.eviction_policy.on_access(&address);
        }
        got.ok_or(StateCacheError::StorageNotFound(address, key))
    }

    #[instrument(level = "trace", skip(self))]
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.block_hashes
            .read()
            .get(&number)
            .cloned()
            .ok_or(StateCacheError::BlockHashNotFound(number))
    }
}

impl<const N: usize, E: EvictionPolicy<Address>> ShardedCache<N, E> {
    /// Build a sharded cache with `max_per_shard` capacity per shard and a
    /// fresh eviction policy per shard (produced by `eviction_factory`).
    pub fn new(max_per_shard: usize, eviction_factory: impl Fn() -> E) -> Self {
        Self {
            shards: core::array::from_fn(|_| {
                RwLock::new(ShardedCacheInner {
                    accounts: HashMap::new(),
                    storage: HashMap::new(),
                    eviction_policy: eviction_factory(),
                })
            }),
            max_per_shard,
            block_hashes: RwLock::new(HashMap::new()),
            codes: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, addr: Address, account: Account) {
        let mut inner = self.shard_for(&addr).write();
        inner.accounts.insert(addr, account);
        inner.eviction_policy.on_insert(&addr);
        while inner.accounts.len() > self.max_per_shard {
            if let Some(victim) = inner.eviction_policy.evict_one() {
                inner.accounts.remove(&victim);
            } else {
                break;
            }
        }
    }

    pub fn shard_len(&self, idx: usize) -> usize {
        self.shards[idx].read().accounts.len()
    }

    pub fn len(&self) -> usize {
        self.shards.iter().map(|s| s.read().accounts.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn shard_for(&self, addr: &Address) -> &RwLock<ShardedCacheInner<E>> {
        let idx = addr.0[0] as usize % N;
        &self.shards[idx]
    }
}

impl<const N: usize> Default for ShardedCache<N, NoOpEviction> {
    /// Unbounded, no-eviction cache. Useful for tests and benchmarks where
    /// eviction would be noise.
    fn default() -> Self {
        Self::new(usize::MAX, NoOpEviction::default)
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
        let mut cache: ShardedCache<16, NoOpEviction> = ShardedCache::default();
        let a = addr_with_first_byte(0x05, 0x01);

        cache.insert(a, account_with_balance(100));
        let got = cache.basic(a).unwrap().expect("account present");
        assert_eq!(got.balance, U256::from(100u64));
    }

    #[test]
    fn basic_returns_none_for_missing() {
        let mut cache: ShardedCache<16, NoOpEviction> = ShardedCache::default();
        let missing = addr_with_first_byte(0xff, 0xff);
        assert!(cache.basic(missing).unwrap().is_none());
    }

    #[test]
    fn distinct_first_bytes_route_to_distinct_shards() {
        let cache: ShardedCache<16, NoOpEviction> = ShardedCache::default();
        cache.insert(addr_with_first_byte(0x00, 0x01), account_with_balance(1));
        cache.insert(addr_with_first_byte(0x01, 0x01), account_with_balance(2));
        cache.insert(addr_with_first_byte(0x02, 0x01), account_with_balance(3));

        assert_eq!(cache.shard_len(0), 1);
        assert_eq!(cache.shard_len(1), 1);
        assert_eq!(cache.shard_len(2), 1);
    }

    #[test]
    fn first_byte_mod_n_collisions_share_a_shard() {
        let cache: ShardedCache<16, NoOpEviction> = ShardedCache::default();
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
        let mut cache: ShardedCache<16, NoOpEviction> = ShardedCache::default();
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
        let cache: Arc<ShardedCache<16, NoOpEviction>> = Arc::new(ShardedCache::default());

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
        assert_send::<ShardedCache<16, NoOpEviction>>();
        assert_sync::<ShardedCache<16, NoOpEviction>>();
    }

    #[test]
    fn lru_eviction_when_over_capacity() {
        use crate::eviction::LruEviction;

        // 1 shard, capacity 2. Insert 3 → oldest gets evicted.
        let mut cache: ShardedCache<1, LruEviction<Address>> =
            ShardedCache::new(2, LruEviction::new);

        let a1 = addr_with_first_byte(0x00, 0x01);
        let a2 = addr_with_first_byte(0x00, 0x02);
        let a3 = addr_with_first_byte(0x00, 0x03);

        cache.insert(a1, account_with_balance(1));
        cache.insert(a2, account_with_balance(2));
        cache.insert(a3, account_with_balance(3));

        assert!(cache.basic(a1).unwrap().is_none(), "a1 should be evicted");
        assert!(cache.basic(a2).unwrap().is_some());
        assert!(cache.basic(a3).unwrap().is_some());
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn lru_access_promotes_to_most_recent() {
        use crate::eviction::LruEviction;

        // 1 shard, capacity 2. Insert a1, a2; access a1; insert a3 → a2 evicted (a1 was just used).
        let mut cache: ShardedCache<1, LruEviction<Address>> =
            ShardedCache::new(2, LruEviction::new);

        let a1 = addr_with_first_byte(0x00, 0x01);
        let a2 = addr_with_first_byte(0x00, 0x02);
        let a3 = addr_with_first_byte(0x00, 0x03);

        cache.insert(a1, account_with_balance(1));
        cache.insert(a2, account_with_balance(2));
        let _ = cache.basic(a1).unwrap(); // promotes a1 to MRU
        cache.insert(a3, account_with_balance(3));

        assert!(
            cache.basic(a1).unwrap().is_some(),
            "a1 was just accessed, should survive"
        );
        assert!(cache.basic(a2).unwrap().is_none(), "a2 should be evicted");
        assert!(cache.basic(a3).unwrap().is_some());
    }
}
