use core::hash::Hash;
use lru::LruCache;
use std::num::NonZeroUsize;

pub trait EvictionPolicy<K> {
    fn on_access(&mut self, key: &K);

    fn on_insert(&mut self, key: &K);

    fn evict_one(&mut self) -> Option<K>;

    fn forget(&mut self, key: &K);
}

pub struct LruEviction<K: Hash + Eq> {
    order: LruCache<K, ()>,
}

impl<K: Hash + Eq + Clone> EvictionPolicy<K> for LruEviction<K> {
    fn on_access(&mut self, key: &K) {
        self.order.get(key);
    }

    fn on_insert(&mut self, key: &K) {
        self.order.put(key.clone(), ());
    }

    fn evict_one(&mut self) -> Option<K> {
        self.order.pop_lru().map(|(key, _)| key)
    }

    fn forget(&mut self, key: &K) {
        self.order.pop(key);
    }
}

impl<K: Hash + Eq> LruEviction<K> {
    /// Build an LRU tracker with default capacity (1024).
    /// For larger caches, use [`Self::with_capacity`] sized at `max_per_shard + 1`
    /// or higher to prevent the inner `LruCache` from auto-evicting before
    /// the containing cache's eviction logic runs.
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            order: LruCache::new(NonZeroUsize::new(cap).expect("capacity must be > 0")),
        }
    }
}

impl<K: Hash + Eq> Default for LruEviction<K> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct NoOpEviction;

impl<K: Hash + Eq> EvictionPolicy<K> for NoOpEviction {
    fn on_access(&mut self, _key: &K) {}

    fn on_insert(&mut self, _key: &K) {}

    fn evict_one(&mut self) -> Option<K> {
        None
    }

    fn forget(&mut self, _key: &K) {}
}
