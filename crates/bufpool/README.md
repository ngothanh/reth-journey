# eth-storage-cache

In-memory account state cache for Ethereum execution. Mirrors revm's
`Database` trait shape with multiple concurrency implementations.

Phase 3's `storage-trie` crate will layer this over an MDBX-backed page
provider; today this is the pure in-memory primitive.

## Types

### Page primitives

- **`Page`** — 4 KiB heap-allocated buffer (`Box<[u8; 4096]>`) with `Deref` /
  `DerefMut` to the inner array and `Drop` instrumented via `tracing::trace!`
  for drop-order observation.
- **`PageBox<T>`** — `Box`-like smart pointer over a `Page` for
  single-allocation, deserialize-in-place storage of `T`. Today `T: Sized`;
  full `T: ?Sized` lands in W4 (Layout-based unsafe).
- **`PageAllocator`** — singly-linked free-list allocator over `Page`s.
  Recycles freed pages instead of round-tripping the global allocator.

### State data structures

- **`Account`** — `{ nonce, balance, code_hash, code }` mirroring
  `revm_primitives::AccountInfo`.
- **`EMPTY_CODE_HASH`** — `keccak256(&[])` constant; EOA marker.

### `StateCache` trait

A revm-style read interface (`basic`, `code_by_hash`, `storage`, `block_hash`)
with five concrete implementations:

| Impl | Storage | Concurrency | Notes |
|------|---------|-------------|-------|
| `LocalAccountCache` | `HashMap<Address, Rc<RefCell<Account>>>` | single-threaded | Drains on commit |
| `SharedAccountCache` | `HashMap<Address, Arc<RwLock<Account>>>` | multi-threaded | parking_lot RwLock per entry |
| `MutexCache` | `parking_lot::Mutex<{ accounts, code, storage, block_hashes }>` | multi-threaded | One coarse lock |
| `RwLockCache` | `parking_lot::RwLock<{ ... }>` | multi-threaded | Read-write split |
| `ShardedCache<const N, E>` | `[parking_lot::RwLock<{ ... }>; N]` | multi-threaded | First-byte-mod-N routing + pluggable eviction |

### Eviction policies

- **`EvictionPolicy<K>`** — trait with `on_access` / `on_insert` / `evict_one` / `forget`.
- **`LruEviction<K>`** — wraps `lru::LruCache`; doubly-linked-list LRU.
- **`NoOpEviction`** — null impl; cache grows unbounded.

## Quick start

```rust
use eth_storage_cache::{Account, ShardedCache, LruEviction};
use eth_primitives::{Address, U256};

// Production shape: 16 shards, 1000 entries each, LRU eviction.
let cache: ShardedCache<16, LruEviction<Address>> =
    ShardedCache::new(1000, || LruEviction::with_capacity(1001));

let addr = Address::with_last_byte(1);
cache.insert(addr, Account {
    balance: U256::from(1_000_000_u64),
    ..Account::default()
});
```

## Benchmarks

`cargo bench -p eth-storage-cache` runs criterion benchmarks comparing the
four `StateCache` impls under concurrent insert load (1, 2, 4, 8 threads).

Summary (OPS_PER_THREAD = 10,000, 8-core x86_64):

| Threads | Mutex | RwLock | Sharded16 | Sharded64 |
|---|---|---|---|---|
| 1 | 755 µs | 789 µs | 701 µs | 670 µs |
| 8 | 12.2 ms | 12.0 ms | 8.43 ms | 6.22 ms |

**Key takeaways**:
- Mutex and RwLock are equivalent for pure-write workloads (both serialize).
- Sharding restores parallelism. `Sharded64` is ~2× faster than `Mutex` at 8 threads.
- Shard count should exceed expected thread count by 4-8× to avoid collisions.

See `benches/cache_contention.rs` and `benches/RESULTS.md` for details.

## Tests

`cargo test -p eth-storage-cache` — 50+ unit tests including thread-safety,
LRU eviction semantics, and `Send`/`Sync` auto-trait propagation checks.

## Tracing

All `StateCache` methods are instrumented with `tracing::trace!` spans. Set
`RUST_LOG=eth_storage_cache=trace` in your binary to observe cache hits,
misses, and shard routing.

## Design notes

- `parking_lot` is used everywhere instead of `std::sync` for the no-poisoning
  ergonomics and faster fast-paths. Matches reth's convention.
- All caches expose `StateCache` with the revm-style `&mut self` shape.
  `ShardedCache` uses interior mutability via `RwLock` to make this work
  through `Arc<ShardedCache>` from multiple threads.
- `ShardedCache::basic` and `::storage` use `.write()` (not `.read()`) to call
  `on_access` on the eviction policy. Trade-off: kills read concurrency on
  the shard, keeps LRU correct. Production alternatives (approximate LRU,
  concurrent LRU like `moka`) are deferred to Phase 3.

## Inheritance

- **Inherits from**: `eth-primitives` (Address, B256, U256, Bytes, FixedBytes).
- **Used by**: future `exec-vm` (Phase 4 — `Database` impl for EVM execution),
  `storage-trie` (Phase 3 — in-memory layer above MDBX), `consensus-engine`
  (Phase 5).
