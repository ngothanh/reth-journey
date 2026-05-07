# Reth Contributor — 24-Month Daily Plan

> **Start**: 2026-04-25
> **Horizon**: 24 months, reassess at Month 24
> **Commitment**: 5h/day × 6 days/week = 30h/week
> **Schedule**: Mon-Sat work, Sunday rest + weekly ritual

**Final-phase deliverables**:
- `storage-trie` crate (Month 7-12) — reth storage + trie re-implementation
- `exec-vm` crate (Month 13-18) — revm + reth evm re-implementation
- `consensus-engine` crate (Month 19-24) — reth consensus + engine API re-implementation

**Phase 1-2 seed crates** (built during Rust mastery, NOT throwaway — extended in later phases):
- `eth-primitives` (Week 1-4) — mirrors `alloy-primitives`. Newtypes, hashing, atomic-cached hashes.
- `eth-rlp` (Week 5) — mirrors `alloy-rlp`. Encodable/Decodable traits + derive macro.
- `eth-storage-cache` (Week 2) — mirrors `revm::CacheDB` + reth in-memory state cache. Replaces `shardkv`.
- `eth-network-codec` (Week 3) — mirrors `reth-eth-wire` framing layer. Replaces `backpressure-net`.
- `eth-consensus` (Week 6-13) — mirrors `alloy-consensus`. Header, tx envelopes, EIP fee math.
- `eth-trie` (Week 10, 20) — mirrors `alloy-trie`. Nibbles, HashBuilder, proof retainer.
- `exec-vm` seed (Week 9, 17) — mirrors `revm-interpreter` subset. Same crate that ships v1.0 in Phase 4.

---

## How to Use

Check off tasks as completed. One day = one section. If you fall behind, adjust forward — don't delete the plan. Sunday ritual reviews the week and recalibrates.

**Daily 5h block structure**:
- 15 min warm-up (review notes, set intent)
- 90 min deep work 1
- 10 min break
- 90 min deep work 2
- 15 min wrap-up (commit, log, questions)

---

## Curriculum Principle: Inherited Exercises

**No throwaways.** Every exercise in Phase 1 and Phase 2 builds a real component in a workspace crate that mirrors a specific upstream module in alloy / reth / revm AND is reused in a later phase. The same `exec-vm` crate seeded in Week 9 is the one that ships v1.0 in Phase 4. The same `eth-trie` crate seeded in Week 10 is what `storage-trie` extends in Phase 3.

Each week is annotated with:
- **Mirror target** — the upstream crate/module the week's artifact mirrors (read it; don't copy it).
- **Crate / file** — what gets created or extended in this workspace.
- **Inherits from** — the prior week's artifacts this work depends on.
- **Feeds into** — the future phase/crate that consumes this work.

Workspace layout (built incrementally):

```
crates/
  eth-primitives/      Week 1-4    → mirrors alloy-primitives
  eth-storage-cache/   Week 2      → mirrors revm::CacheDB + reth in-memory state cache
  eth-network-codec/   Week 3      → mirrors reth-eth-wire framing
  eth-rlp/             Week 5      → mirrors alloy-rlp + alloy-rlp-derive
  eth-consensus/       Week 6-13   → mirrors alloy-consensus
  exec-vm/             Week 9, 17, Phase 4   → mirrors revm-interpreter + revm
  eth-trie/            Week 10, 20, Phase 3  → mirrors alloy-trie
  storage-trie/        Phase 3     → owns mmap/MDBX-backed state DB; consumes eth-trie + eth-storage-cache + eth-rlp + eth-primitives
  consensus-engine/    Phase 5     → owns engine API + fork choice; consumes everything above
```

If you ever feel an exercise is "just to learn the syntax," stop — find the matching alloy/reth/revm module instead. Concept must serve the artifact, not the other way around.

---

# PHASE 1: RUST MASTERY — fast track for experienced engineers (Month 1-3)

> Compressed for a 12-year Java/Kotlin engineer. Skip beginner syntax (you already have analogues for if/else, structs, generics, modules, testing). Drill the genuinely Rust-specific concepts: ownership, lifetimes, traits + coherence, smart pointers, async/Pin, unsafe, atomics. Use the saved time to start Alloy/revm PRs and Ethereum protocol study early — by end of Month 3 you should already be ahead of the original Phase 2 entry point.

## Month 1: Rust Core (Weeks 1-4)

### Week 1 — Ownership/borrowing/lifetimes via `eth-primitives` foundation

**Mirror target**: `alloy-primitives` (Address, B256, U256, Bytes, FixedBytes)
**Crate created**: `crates/eth-primitives/` — workspace member from Day 1.
**Inherits from**: nothing (this is the root of the chain).
**Feeds into**: every later week — eth-rlp imports its types Week 5; eth-consensus Week 6; eth-trie Week 10; storage-trie Phase 3; exec-vm Phase 4; consensus-engine Phase 5.

> Read alloy-primitives source for **shape** (signatures, type relationships) — don't copy code. Rebuild from spec + your own design choices. Mistakes ARE the lesson.

**Pre-week setup**: ✓ already done (rustup, rust-analyzer, cargo tools, repo, Rustlings cloned)

**Monday — Skim the Book chs 1-9, write nothing + workspace scaffold**
- [X] Speed-read Book ch1-3 (~30 min): hello world, variables, functions, control flow — confirm syntax only
- [X] Speed-read Book ch5-9 (~90 min): structs, enums, modules, collections, error handling — note differences from Kotlin (Result vs exceptions, sealed classes vs enums-with-data, no inheritance)
- [X] Skip all Rustlings: `intro`, `variables`, `functions`, `if`, `primitive_types`, `strings`, `vecs`, `hashmaps`, `modules`
- [X] Write `notes/01_kotlin_to_rust_delta.md`: 1-page diff between Kotlin and Rust mental models
- [X] Create workspace `Cargo.toml` (resolver = "2", `[workspace] members = ["crates/*"]`)
- [X] Create `crates/eth-primitives` with `Cargo.toml`, `src/lib.rs`, `src/error.rs` skeleton
- [ ] Read alloy-primitives top-level `lib.rs` + map the 8 types you'll build this week (Bytes, FixedBytes, Address, B256, B64, U256, Bloom, PrimitivesError)
- [ ] Commit + log

**Tuesday — Book ch4 + `FixedBytes<const N: usize>`**
- [X] Book ch4.1 (Ownership) — read twice
- [X] Book ch4.2 (References and Borrowing) — read twice
- [X] Book ch4.3 (Slices)
- [ ] Rustlings `move_semantics` (all 6)
- [ ] **Build**: `crates/eth-primitives/src/fixed_bytes.rs` — `FixedBytes<const N: usize>([u8; N])` with `Copy`, `Default`, `From<[u8; N]>`, `AsRef<[u8]>`, `AsMut<[u8]>`, `Deref<Target=[u8; N]>`, `PartialEq`, `Hash`. `repr(transparent)` so it's ABI-compatible with `[u8; N]` (same as alloy).
- [ ] Test: zero-init, equality, slice access, hash stability. Match alloy-primitives FixedBytes test cases.
- [ ] Borrow-checker drill: try to write a method `fn split(&mut self) -> (&mut [u8], &mut [u8])` and resolve it the right way (`split_at_mut`). Document the lesson in `notes/02_borrow_checker_errors.md` from real code, not contrived programs.
- [ ] Commit + log

**Wednesday — Lifetimes + `Bytes` + `BytesView<'a>`**
- [X] Book ch10.3 (Lifetimes) — read twice
- [ ] Watch Crust of Rust: Lifetime Annotations (full)
- [ ] **Build**: `crates/eth-primitives/src/bytes.rs` — `Bytes(Arc<[u8]>)` cheap-clone wrapper mirroring `alloy_primitives::Bytes`. Methods: `new()`, `from_static(&'static [u8])`, `slice(range) -> Bytes`, `len`, `is_empty`, `as_ref`.
- [ ] **Build**: `BytesView<'a>(&'a [u8])` for borrowed views — this is where lifetime annotations earn their keep. Add `Bytes::view(&self) -> BytesView<'_>`.
- [ ] Implement `From<Vec<u8>>`, `From<&'static [u8]>`, `Display` (lowercase hex with 0x prefix).
- [ ] Document lifetime elision rules in `notes/03_lifetimes.md` using the actual `Bytes::slice` and `BytesView::split_at` signatures as examples.
- [ ] Commit + log

**Thursday — Traits + `Address` + `B256` + sealed-trait pattern**
- [X] Book ch10.1 + ch10.2 (Generics, Traits)
- [ ] Rustlings `generics`, `traits` — all
- [ ] Read about orphan rule, coherence, sealed traits
- [ ] **Build**: `crates/eth-primitives/src/address.rs` — `pub type Address = FixedBytes<20>;` + impl block with `Address::from_word(B256)`, `Address::with_last_byte(u8)`, `Address::ZERO`. EIP-55 checksum encoding via `to_checksum(chain_id: Option<u64>) -> String`.
- [ ] **Build**: `crates/eth-primitives/src/aliases.rs` — `pub type B256 = FixedBytes<32>;`, `B64 = FixedBytes<8>`. Match alloy aliases exactly.
- [ ] **Build**: sealed-trait pattern for a future `Encodable` placeholder — `mod private { pub trait Sealed {} }`. impl `Sealed` for `Address`, `B256`, `Bytes`. This blocks downstream crates from extending the trait — same pattern reth uses on `BlockHashOrNumber`.
- [ ] Write 4 functions (one each for `&dyn Encodable`, `Box<dyn Encodable>`, `impl Encodable`, `<T: Encodable>`) over the sealed trait — observe what compiles and why.
- [ ] Notes in `notes/04_traits.md`: static vs dynamic dispatch tradeoffs.
- [ ] Commit + log

**Friday — Error handling + iterators via `PrimitivesError` + hex parsing**
- [X] Book ch9 + ch13.1 + ch13.2
- [ ] Rustlings `error_handling`, `options`, `iterators` — all
- [ ] Read `thiserror` and `anyhow` docs end-to-end
- [ ] **Build**: `crates/eth-primitives/src/error.rs` — `PrimitivesError` enum (`InvalidLength { expected, got }`, `InvalidHex(String)`, `InvalidChecksum`, `Overflow`) with `thiserror::Error`. Match alloy's variants where they overlap.
- [ ] **Build**: `FromStr` for `Address`, `B256`, `Bytes` — accept both `0x`-prefixed and bare hex. Iterator-driven byte-pair decoder (no `hex` crate dep — write it yourself, then compare to `const-hex`).
- [ ] Three rewrites of `parse_address`: panic, Result+thiserror, anyhow — keep Result+thiserror in the crate; document the trade in `notes/04_traits.md`.
- [ ] Watch Crust of Rust: Iterators (full).
- [ ] Implement `flatten()` from scratch — but apply it: write a `Bytes::concat(parts: impl IntoIterator<Item = impl AsRef<[u8]>>) -> Bytes` using only `Iterator` trait. This is the same shape `alloy_rlp::encode` will need next week.
- [ ] Commit + log

**Saturday — Closures + Fn/FnMut/FnOnce + `U256` + R4R**
- [X] Book ch13.1 (Closures) — focus on FnOnce/FnMut/Fn semantics (Kotlin lambdas don't distinguish)
- [ ] **Build**: `crates/eth-primitives/src/uint.rs` — `pub use ruint::aliases::U256;` + extension trait `U256Ext` adding `from_be_slice`, `to_be_bytes_trimmed_vec`, `bit_len`. Mirrors alloy-primitives' U256 surface.
- [ ] Closure exercise via real use: `Bytes::map_chunks<F: FnMut(&[u8]) -> Bytes>(&self, chunk_size, f) -> Bytes` — used Week 5 by RLP encoder for length-prefixed framing.
- [ ] Read Rust for Rustaceans ch1-2 (Foundations, Types).
- [ ] `cargo clippy --all -- -D warnings`, `cargo test`, tag `eth-primitives v0.1.0-week1`.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] "Can I explain ownership/borrowing/lifetimes using `Bytes::slice` and `FixedBytes` examples without looking up?" — if no, redo Tue/Wed.
- [ ] Inheritance check: `eth-primitives` exports Address, B256, Bytes, FixedBytes, U256, PrimitivesError. Every later week imports from this.

---

### Week 2 — Smart pointers + sync concurrency via `eth-storage-cache`

**Mirror target**: `revm::db::CacheDB` (in-memory state DB) + `reth_provider`'s in-memory layer + `revm_primitives::Database` trait shape.
**Crate created**: `crates/eth-storage-cache/` — replaces what was `shardkv` (which fed into nothing).
**Inherits from**: `eth-primitives` (Address, B256, U256, Bytes from Week 1).
**Feeds into**: `exec-vm` Phase 4 consumes this as its `Database` impl driving execution; `storage-trie` Phase 3 uses this as the in-memory layer above MDBX.

> The `StateCache` trait you build this week is the same trait `exec-vm` will require to read account state during opcode execution. Get the shape right.

**Monday — Box, Deref, Drop via `Page` primitive**
- [X] Book ch15.1-15.4
- [ ] **Build**: `crates/eth-storage-cache/src/page.rs` — `Page(Box<[u8; 4096]>)` with `Deref<Target=[u8; 4096]>`, `DerefMut`, `Drop` instrumented via `tracing::trace!` to learn drop order. This 4 KiB page is the actual primitive `storage-trie` Phase 3 reuses for its mmap-backed layout.
- [ ] Implement `MyBox<T>` exercise but apply it: write a `PageBox<T: ?Sized>` that uses `Page` as backing storage for `T` (single-allocation deserialize-in-place). This is the shape MDBX cursors use.
- [ ] Single-linked list of `Page`s as a free-list allocator (`PageAllocator`). Attempt a doubly-linked free list to feel the pain → motivates Rc/Weak Tuesday.
- [ ] Commit + log

**Tuesday — RefCell, Rc, Arc via `Account` cache**
- [X] Book ch15.5-15.6
- [ ] Watch Crust of Rust: Smart Pointers and Interior Mutability
- [ ] **Build**: `crates/eth-storage-cache/src/account.rs` — `Account { nonce: u64, balance: U256, code_hash: B256, code: Option<Bytes> }` mirroring `revm_primitives::Account`. Use `eth-primitives` types.
- [ ] **Build**: `LocalAccountCache(HashMap<Address, Rc<RefCell<Account>>>)` first — single-threaded. Add `get_or_load`, `commit`. Use `RefCell::borrow_mut` and observe the runtime panic when you double-borrow — the borrow-checker lesson moves to runtime.
- [ ] **Migrate**: clone the file to `SharedAccountCache(HashMap<Address, Arc<RwLock<Account>>>)` for the multi-threaded version. Document the line-by-line diff in `notes/05_smart_pointers.md` — when to reach for Rc vs Arc vs Box vs `&`.
- [ ] Commit + log

**Wednesday — Threads, channels, Mutex via `StateCache` trait**
- [X] Book ch16 (whole chapter)
- [ ] Watch Crust of Rust: Channels — implement bounded MPSC from scratch
- [ ] **Build**: `crates/eth-storage-cache/src/database.rs` — `StateCache` trait shaped like revm's `Database`:
  ```
  trait StateCache {
      type Error;
      fn basic(&self, addr: Address) -> Result<Option<Account>, Self::Error>;
      fn code_by_hash(&self, hash: B256) -> Result<Bytes, Self::Error>;
      fn storage(&self, addr: Address, slot: U256) -> Result<U256, Self::Error>;
      fn block_hash(&self, num: u64) -> Result<B256, Self::Error>;
  }
  ```
- [ ] Implement `MutexCache` (single mutex over the HashMap) and `RwLockCache`. Apply your bounded-MPSC channel as a write-batch queue: writes go through the channel, applied in order by a single committer thread (this is the same pattern reth uses for its `StateProviderFactory` writer).
- [ ] Read `parking_lot::Mutex` vs std — keep `parking_lot` as the dep (reth uses it).
- [ ] Commit + log

**Thursday — Send/Sync via `ShardedCache`**
- [X] Book ch16.4 (Send and Sync)
- [ ] Read `std::marker` docs carefully
- [ ] **Build**: `ShardedCache<const N: usize>` — `[parking_lot::RwLock<HashMap<Address, Account>>; N]` hash-routed by `Address::word()[0] % N`. Implement `StateCache` for it. Same sharding scheme reth's tx pool uses.
- [ ] Send/!Sync + !Send/Sync exercises grounded in the cache: prove `Rc<RefCell<Account>>` is `!Send` (compile error) and `Arc<RwLock<Account>>` is `Send + Sync`. Document the trait bounds your `StateCache` impls need.
- [ ] Commit + log

**Friday — `EvictionPolicy` + criterion benches**
- [ ] **Build**: `crates/eth-storage-cache/src/eviction.rs` — `EvictionPolicy` trait. Two impls: `LruEviction` (size-bounded), `BlockTagEviction` (pinned by block number, evicts when N blocks behind chain head — this is the eviction `BlockchainTree` actually uses).
- [ ] Wire eviction into `ShardedCache` (each shard has its own LRU).
- [ ] criterion bench: Mutex vs RwLock vs Sharded(N=16, N=64) under varied read/write ratios. Plot and commit the results.
- [ ] Read `parking_lot`, `dashmap`, `arc-swap` docs — 1-paragraph summary each in `notes/05_smart_pointers.md`. Note: reth picks `parking_lot::RwLock` over `dashmap` for the state cache; understand why (you'll see contention patterns in the bench).
- [ ] Commit + log

**Saturday — Polish + R4R + tag v0.1.0**
- [ ] thiserror `StateCacheError`, tracing spans on `basic`/`storage` calls (block_number + address fields), full concurrent tests with `loom` on a tiny subset.
- [ ] Read Rust for Rustaceans ch1-2 (Foundations, Types).
- [ ] README documents the trait, the shard scheme, the eviction policies. Tag `eth-storage-cache v0.1.0`.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] Inheritance check: `StateCache` trait mirrors revm's `Database` — Phase 4 `exec-vm` will impl this trait for free; Phase 3 `storage-trie` will impl it backed by mmap.

---

### Week 3 — Async/Pin/Future via `eth-network-codec`

**Mirror target**: `reth-eth-wire` framing layer + `tokio_util::codec::Framed`. Specifically the `P2PStream` / `EthStream` framing — message length prefix, snappy compression hook, message-id dispatch. Not the full devp2p/RLPx handshake (that's a Phase 5 problem); just the codec + stream substrate.
**Crate created**: `crates/eth-network-codec/` — replaces what was `backpressure-net`.
**Inherits from**: `eth-primitives` (B256 for message hashes, Bytes for payloads).
**Feeds into**: Phase 5 `consensus-engine` reuses the `MessageStream` for engine API JSON-RPC framing; the rate-limited stream becomes the per-peer ingress for any sync stage that consumes block bodies; Week 11 type-state pattern adds `Connection<S>` lifecycle states.

**Monday — Tokio fast track + transport scaffold**
- [ ] Read Tokio tutorial cover-to-cover: Hello → Spawning → Shared State → Channels → I/O → Framing.
- [ ] **Build**: `crates/eth-network-codec/src/transport.rs` — `tokio::net::TcpStream` wrapper + `tokio_util::codec::LengthDelimitedCodec` framed reader/writer with 1 MiB max frame size (devp2p limit).
- [ ] Manual TCP echo via the framed transport — proves bytes flow round-trip.
- [ ] Commit + log

**Tuesday — Manual Future + `MessageRequest`**
- [ ] Async Book ch1-7 in one go.
- [ ] Watch Crust of Rust: Async/Await (full) — implement a trivial executor.
- [ ] **Build**: `crates/eth-network-codec/src/request.rs` — `MessageRequest<R>` future that registers a request_id and resolves when the response arrives via a `oneshot::Receiver<R>`. Same shape as reth's `HeadersRequest` / `BodiesRequest`.
- [ ] Counter Future exercise applied: build a `RetryFuture<F: Future>` that retries the inner future up to N times — used for transient network errors.
- [ ] Commit + log

**Wednesday — Pin/Unpin via `MessageStream`**
- [ ] Watch Crust of Rust: The Drop Check; read `std::pin` docs.
- [ ] **Build**: `crates/eth-network-codec/src/stream.rs` — `MessageStream<C: Codec, IO>` implementing `tokio_stream::Stream<Item = Result<C::Message, CodecError>>`. Requires `Pin` projection — use `pin_project_lite` (the same crate reth-eth-wire uses).
- [ ] Demonstrate why `MessageStream` cannot be `Unpin` (it holds a borrowed read buffer mid-poll). Self-referential struct exercise: rewrite `MessageStream` once with manual unsafe pin projection, then with `pin_project_lite`. Compare.
- [ ] `notes/06_pin_unpin.md` — written using your `MessageStream` as the worked example.
- [ ] Commit + log

**Thursday — `EthMessage` enum + `Codec` trait**
- [ ] **Build**: `crates/eth-network-codec/src/codec.rs` — `Codec` trait (`type Message`, `encode`, `decode`).
- [ ] **Build**: `crates/eth-network-codec/src/message.rs` — `EthMessage` enum subset: `Status { protocol_version, network_id, td, head, genesis }`, `BlockHeaders(Vec<HeaderRlp>)`, `BlockBodies(Vec<BodyRlp>)`, `NewBlock { block, td }`, `GetBlockHeaders { start, limit, skip, reverse }`. Mirrors `reth_eth_wire::EthMessage` shape (message_id + payload).
- [ ] RLP encode/decode placeholder using a tagged-byte format for now (full RLP comes Week 5; today the goal is the message-id dispatch table).
- [ ] tokio TCP server scaffold with graceful shutdown (SIGTERM/SIGINT) running the codec.
- [ ] Commit + log

**Friday — Token bucket as custom `Future` + per-peer rate limiting**
- [ ] **Build**: `crates/eth-network-codec/src/rate_limit.rs` — `TokenBucket` as a custom `Future` (NOT `tokio::time::interval`-based). Wakers driven by a `tokio::time::Sleep`. This is the same shape reth uses for per-peer rate limiting on the `eth/68` protocol.
- [ ] **Build**: `RateLimitedStream<S: Stream>` that wraps `MessageStream` and enforces per-peer token bucket. Type-state pattern preview: `RateLimitedStream` is parameterized over the rate config to make zero-cost when limits are disabled.
- [ ] Test under load with mock clients (1k concurrent peers, fixed token rate).
- [ ] Commit + log

**Saturday — `BackpressureStrategy` + observability + tag v0.1.0**
- [ ] **Build**: `BackpressureStrategy` enum (`DropOldest`, `DropNewest`, `Block`) wired into a bounded peer-message channel. All three impls — same options reth offers on its peer message buffers.
- [ ] Tracing spans for connection lifecycle (peer_id, protocol_version, connected_at).
- [ ] Prometheus metrics via `metrics` crate (`peer_msgs_in_total`, `peer_msgs_dropped_total{strategy}`, `peer_rate_limit_events_total`), `/metrics` endpoint.
- [ ] Load test with 10k concurrent connections, document findings in README.
- [ ] Tag `eth-network-codec v0.1.0`.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] Inheritance check: `Codec` trait + `MessageStream` + `RateLimitedStream` are the substrate Phase 5 will use for engine API; Week 11 will add `Connection<Disconnected/Handshaking/Established>` type-states.

---

### Week 4 — Atomics, unsafe, variance, macros via `eth-primitives` v0.2

**Mirror target**: `reth_primitives::SealedHeader` (atomic-cached hash via OnceLock), `alloy_primitives::U256` (`repr(transparent)` over ruint), `alloy_primitives::b256!` const macro, `alloy-rlp-derive` (proc macro scaffold).
**Crate extended**: `eth-primitives` from v0.1 → v0.2 (no new crate). Add `crates/eth-primitives-derive/` for the proc macro.
**Inherits from**: Week 1 `eth-primitives` (FixedBytes, Bytes, Address, B256, U256).
**Feeds into**: Week 5 `eth-rlp` derive macro builds on the proc macro infrastructure here; `consensus-engine` Phase 5 uses the SeqLock-protected `ChainHead` for fork choice updates.

**Monday — Layout audit on existing `eth-primitives`**
- [ ] Rustonomicon ch1 (Meet Safe and Unsafe), ch2 (Data Layout), ch3 (selected sections).
- [ ] Run `size_of`/`align_of` over every type in `eth-primitives`. Verify `FixedBytes<N>` is `repr(transparent)` (so `Address` has the same ABI as `[u8; 20]` — same as alloy).
- [ ] Add `repr(C)` to `Account` (in `eth-storage-cache`) and document why it matters for the future MDBX serialization in Phase 3.
- [ ] Inspect `Bytes` layout — `Arc<[u8]>` has 2-word size; document in `notes/07_variance.md` why this matters (covariance over the contained slice).
- [ ] Commit + log

**Tuesday — Atomics via `SealedHeader` + `ChainHead` SeqLock**
- [ ] Watch Crust of Rust: Atomics and Memory Ordering (full).
- [ ] **Build**: `crates/eth-primitives/src/atomic_hash.rs` — `OnceLock<B256>` lazy hash cache. Define a `Sealable` trait with `fn hash_slow(&self) -> B256;` — impls cache via `OnceLock`. This is exactly how reth's `SealedHeader` lazily caches `keccak256(rlp(header))`.
- [ ] **Build**: `crates/eth-primitives/src/chain_head.rs` — `ChainHead { hash: B256, number: u64 }` protected by a SeqLock (writer increments seq, reader retries on odd seq). Used Phase 5 by `consensus-engine` for `engine_forkchoiceUpdated` reads-hot/writes-rare pattern.
- [ ] Re-read your own Ryuo disruptor code with fresh atomics eyes — note any bugs/improvements.
- [ ] Commit + log

**Wednesday — Variance + PhantomData via `Sealed<T>`**
- [ ] Watch Crust of Rust: Subtyping and Variance.
- [ ] **Build**: `crates/eth-primitives/src/sealed.rs` — `Sealed<T> { inner: T, hash: OnceLock<B256> }` newtype. Same shape as `reth_primitives::SealedHeader<H>` — a generic wrapper that adds a cached hash to any header-like.
- [ ] Make `Sealed<T>` covariant via `PhantomData<&'a T>` for a borrowed variant `SealedRef<'a, T>`. Demonstrate the variance bites when you try to mutate through a shared reference — `notes/07_variance.md` worked example uses `Sealed`.
- [ ] R4R ch6.
- [ ] Commit + log

**Thursday — Unsafe + miri via `BytesMut::reserve`**
- [ ] Read Nomicon chapters on aliasing, UB.
- [ ] **Build**: `crates/eth-primitives/src/bytes_mut.rs` — `BytesMut` (mirrors `bytes::BytesMut` / `alloy_primitives::BytesMut`). `reserve` and `extend_from_slice` written with raw pointer arithmetic (`NonNull<u8>` + `Layout`). Convert to `Bytes` via `BytesMut::freeze` (cheap — moves the allocation into `Arc<[u8]>`).
- [ ] Run `cargo +nightly miri test -p eth-primitives`. Chase every UB report. Don't move on until miri is clean.
- [ ] Commit + log

**Friday — Macros via `b256!` + `SimpleEncode` derive**
- [ ] Read Rust for Rustaceans ch7 (Macros) + Little Book of Rust Macros (declarative).
- [ ] **Build**: `crates/eth-primitives/src/macros.rs` — `b256!("0xdead...")` and `address!("0x...")` const macros (`macro_rules!`) that hex-decode at compile time and produce `FixedBytes<N>`. Mirrors `alloy_primitives::b256!`.
- [ ] **Build**: `crates/eth-primitives-derive/` proc-macro crate using `syn` + `quote`. Implement `#[derive(SimpleEncode)]` for fixed-size field types — placeholder for Week 5's `RlpEncodable`. The crate scaffolding (Cargo.toml `proc-macro = true`, `syn`/`quote`/`proc-macro2` deps, basic `DeriveInput` parsing) IS the Week 5 derive's home — no rewrite needed next week.
- [ ] Test the derive on a 3-field struct.
- [ ] Commit + log

**Saturday — R4R + integration polish**
- [ ] R4R ch1-5 (Foundations, Types, Designing Interfaces, Error Handling, Project Structure) — already partially done; finish.
- [ ] Apply at least one R4R insight to refactor `eth-storage-cache` or `eth-network-codec` (e.g., sealed traits on the public API, `#[non_exhaustive]` on error enums).
- [ ] Tag `eth-primitives v0.2.0`.
- [ ] Commit + log

**Sunday — Rest + End Month 1 review**
- [ ] Honest assessment: "Could I read reth-trie source today and follow it?"
- [ ] Inheritance check: 4 crates shipped — `eth-primitives v0.2`, `eth-storage-cache v0.1`, `eth-network-codec v0.1`, `eth-primitives-derive v0.1`. Every later Phase 1 week imports from these.
- [ ] If any Rust topic is shaky, queue a Week-5 re-read; do NOT redo any artifact (it's all production now).
- [ ] Update North Star M1 metrics

---

## Month 2: Production Rust + Early Alloy (Weeks 5-8)

### Week 5 — `eth-rlp` crate + Alloy onboarding

**Mirror target**: `alloy-rlp` + `alloy-rlp-derive`. Specifically `Encodable`, `Decodable`, `Header`, `length_of_length`, and the `RlpEncodable`/`RlpDecodable` derive macros.
**Crate created**: `crates/eth-rlp/` + extends `crates/eth-primitives-derive/` to host the RLP derive.
**Inherits from**: `eth-primitives` v0.2 (Bytes, BytesMut, Address, B256, U256), `eth-primitives-derive` (proc-macro crate scaffold from Week 4 Friday).
**Feeds into**: `eth-consensus` Week 6 uses RLP for Header + tx encoding; `eth-trie` Week 10 uses it for trie node hashing; `storage-trie` Phase 3 uses it for DB serialization; `exec-vm` Phase 4 uses it for receipts.

**Monday — Spec + traits**
- [ ] Read the RLP spec (ethereum.org) end-to-end. Read `alloy-rlp`'s `Encodable` and `Decodable` source — copy the trait shapes exactly (your downstream consumers expect alloy's signatures).
- [ ] **Build**: `crates/eth-rlp/src/lib.rs` — `pub trait Encodable { fn length(&self) -> usize; fn encode(&self, out: &mut dyn BufMut); }`, `pub trait Decodable: Sized { fn decode(buf: &mut &[u8]) -> Result<Self, RlpError>; }`. Match alloy's signatures.
- [ ] R4R ch7 (Macros) cross-reference for the derive groundwork.
- [ ] Commit + log

**Tuesday — `Header` + scalar encoding**
- [ ] **Build**: `crates/eth-rlp/src/header.rs` — `Header { list: bool, payload_length: usize }`, `decode_header`, `encode_header`. Test against ethereumjs RLP fixtures.
- [ ] **Build**: `crates/eth-rlp/src/encodable.rs` — `Encodable`/`Decodable` impls for `u8`, `u16`, `u32`, `u64`, `U256`, `bool`, `&[u8]`, `Vec<u8>`, `String`, `Address`, `B256`, `Bytes` (using your `eth-primitives` types).
- [ ] R4R ch9-11 — note FFI patterns for libmdbx-rs Phase 3.
- [ ] Commit + log

**Wednesday — List encoding + `Vec<T>` + `length_of_length`**
- [ ] **Build**: `Encodable` for `Vec<T: Encodable>`, `Option<T>`, `(A, B)`, fixed-size arrays. Length-of-length helper (the alloy `length_of_length` function).
- [ ] Nested list test: encode `Vec<Vec<u64>>` matches Geth's RLP output byte-for-byte.
- [ ] Buffer-size-class optimization: pre-size the output `BytesMut` when `Encodable::length` is exact (this is the same pattern alloy uses).
- [ ] R4R ch12.
- [ ] Commit + log

**Thursday — Alloy code tour (compare not copy)**
- [ ] Clone alloy-rs/alloy. Browse workspace `Cargo.toml`. Read `alloy-primitives` source AND DIFF against your `eth-primitives`. Note 5 specific places they diverge — for each, decide: keep yours, port theirs, or document trade.
- [ ] Read `alloy-rlp` source — confirm your trait signatures match. Note any helpers alloy has that you don't (e.g., `encode_iter`, `MaxEncodedLen`).
- [ ] Commit notes + diff log.
- [ ] Commit + log

**Friday — `RlpEncodable` / `RlpDecodable` derive macros**
- [ ] **Build**: extend `crates/eth-primitives-derive/` (renamed `eth-rlp-derive` if cleaner) with `#[derive(RlpEncodable, RlpDecodable)]` proc macros. Mirror `alloy-rlp-derive` API: tuple-struct support, named-struct support, `#[rlp(skip)]`, `#[rlp(flatten)]`.
- [ ] Test on a 5-field struct (`TestHeader { parent: B256, beneficiary: Address, number: u64, gas_used: u64, extra: Bytes }`) — bytes match alloy's derive output.
- [ ] Commit + log

**Saturday — `etherscanlite` CLI on `eth-primitives`**
- [ ] **Build**: `crates/etherscanlite/` — CLI: `etherscanlite <address>` → fetches balance, nonce, last-5-tx hashes via `alloy-provider` JSON-RPC, but parses the responses INTO your `eth-primitives::Address` / `B256` / `U256`. ~500 LOC. Demonstrates eth-primitives interop with the alloy ecosystem.
- [ ] First Alloy issue scan — `good first issue` in `alloy-rlp` or `alloy-primitives` preferred (you now know the codebase). Pick one for next week.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] Tag `eth-rlp v0.1.0`. Inheritance check: derive macros + `Encodable`/`Decodable` ready for Week 6 `eth-consensus`.

---

### Week 6 — `eth-consensus` core: Header + transaction envelopes

**Mirror target**: `alloy-consensus` (`Header`, `TxLegacy`, `TxEip2930`, `TxEip1559`, `TxEip4844`, `Transaction` trait, `Signed<T>`, `TxEnvelope`).
**Crate created**: `crates/eth-consensus/`.
**Inherits from**: `eth-primitives` (Address, B256, U256, Bytes), `eth-rlp` (RLP traits + derive).
**Feeds into**: `eth-trie` Week 10-11 (header → state root verification); `exec-vm` Phase 4 (`BlockEnv` + `TxEnv` construction); `consensus-engine` Phase 5 (payload validation, fork choice).

**Monday — Yellow Paper §4 + `Header`**
- [ ] ME ch3 (Clients), ch4 (Cryptography). Yellow Paper §4 (Block, State, Account).
- [ ] Run reth on Sepolia, observe sync logs.
- [ ] **Build**: `crates/eth-consensus/src/header.rs` — `Header` struct mirroring `alloy_consensus::Header` exactly (parent_hash, ommers_hash, beneficiary, state_root, transactions_root, receipts_root, logs_bloom, difficulty, number, gas_limit, gas_used, timestamp, extra_data, mix_hash, nonce, base_fee_per_gas, withdrawals_root, blob_gas_used, excess_blob_gas, parent_beacon_block_root, requests_hash). `#[derive(RlpEncodable, RlpDecodable)]` from your Week-5 derive.
- [ ] Test: encode mainnet block 1's header → bytes match `cast block 1 --raw`.
- [ ] Commit + log

**Tuesday — Tx types + `Transaction` trait**
- [ ] ME ch5 (Wallets), ch6 (Transactions). Yellow §6.
- [ ] **Build**: `crates/eth-consensus/src/transaction/legacy.rs` — `TxLegacy { chain_id, nonce, gas_price, gas_limit, to: TxKind, value, input }`. `TxKind = Call(Address) | Create`.
- [ ] **Build**: `eip1559.rs` — `TxEip1559 { chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, to, value, input, access_list }`.
- [ ] **Build**: `eip4844.rs` — `TxEip4844 { ..., max_fee_per_blob_gas, blob_versioned_hashes }`.
- [ ] **Build**: `Transaction` trait with the same shape as alloy (chain_id, nonce, gas_limit, gas_price, value, input, to, signature_hash). Default-method-rich.
- [ ] Sign each tx type via `alloy-signer` against your `signature_hash()`. Verify recovery.
- [ ] Commit + log

**Wednesday — EIP-1559 + EIP-4844 fee math**
- [ ] Read EIP-1559 spec + Paradigm 1559 analysis. Read EIP-4844 spec.
- [ ] **Build**: `crates/eth-consensus/src/eip1559.rs` — `pub fn calc_next_block_base_fee(parent: &Header) -> u64`. Mirrors alloy-eips. Test against mainnet pre/post-1559 block pairs.
- [ ] **Build**: `crates/eth-consensus/src/eip4844.rs` — `calc_excess_blob_gas`, `calc_blob_fee`, `MAX_BLOB_GAS_PER_BLOCK`, `BLOB_GASPRICE_UPDATE_FRACTION`. Test fee fork transitions.
- [ ] Commit + log

**Thursday — Alloy issue hunt + claim (now informed by your eth-consensus work)**
- [ ] Browse alloy-rs/alloy issues: `good first issue`, `help wanted`, `docs`.
- [ ] Prefer issues in `alloy-consensus`, `alloy-eips`, or `alloy-rlp` — domains you now own in code. Pick one, claim.
- [ ] Read CONTRIBUTING.md + skim 5 recently merged PRs.
- [ ] Commit notes.

**Friday — First Alloy PR work**
- [ ] Fork, branch with convention, implement. Use what you learned building `eth-consensus`.
- [ ] Commit + log

**Saturday — First Alloy PR submitted + `Signed<T>`**
- [ ] `cargo fmt`, `cargo clippy --all`, `cargo nextest`. Open PR with clear motivation + test plan.
- [ ] **Build**: `crates/eth-consensus/src/signed.rs` — `Signed<T> { tx: T, signature: Signature, hash: OnceLock<B256> }` mirroring `alloy_consensus::Signed`. Reuses `OnceLock` hash cache pattern from Week-4 `eth-primitives`.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 7 — `eth-consensus`: Authorization (EIP-7702), Requests (EIP-7685), EOF + more PRs

**Crate extended**: `eth-consensus` v0.1 → v0.2.
**Mirror target**: `alloy_eips::eip7702::Authorization`, `alloy_eips::eip7685::Requests`, `revm_primitives::Bytecode` (legacy/EOF dispatch).

**Monday — PR #1 review iteration + EIP-2930 access list**
- [ ] Address Alloy PR #1 review.
- [ ] **Build**: `crates/eth-consensus/src/eip2930.rs` — `AccessList(Vec<AccessListItem { address, storage_keys: Vec<B256> }>)`. RLP derive. Wire into `TxEip1559` + `TxEip4844`.
- [ ] Commit + log

**Tuesday — EIP-7702 `Authorization` + `TxEip7702`**
- [ ] Read EIP-7702 + EIP-7685 specs end-to-end.
- [ ] **Build**: `crates/eth-consensus/src/eip7702.rs` — `Authorization { chain_id, address, nonce, y_parity, r, s }`, `SignedAuthorization`, `recover_authority` (delegates to `eth-primitives` `k256` recovery). Mirrors `alloy_eips::eip7702`.
- [ ] **Build**: `TxEip7702 { ..., authorization_list: Vec<SignedAuthorization> }`.
- [ ] Commit + log

**Wednesday — EIP-7685 + EOF skeleton**
- [ ] Read EOF EIPs: 3540, 3670, 4200, 4750.
- [ ] **Build**: `crates/eth-consensus/src/eip7685.rs` — `Request` enum + `requests_root` keccak hashing of RLP-encoded requests. Mirrors `alloy_eips::eip7685`.
- [ ] **Build**: `crates/eth-consensus/src/bytecode.rs` — `Bytecode` enum (`Legacy(Bytes)`, `LegacyAnalyzed { bytecode: Bytes, jumpdests: Bitvec }`, `Eof(EofBytecode)`) mirroring `revm_primitives::Bytecode`. EOF parser skeleton — at least the EIP-3540 magic prefix `0xEF00 01` + section table parse. Full EOF execution comes Phase 4.
- [ ] Commit + log

**Thursday — `TxEnvelope` + Second Alloy PR**
- [ ] **Build**: `crates/eth-consensus/src/envelope.rs` — `TxEnvelope` enum dispatching across all tx types (`Legacy`, `Eip2930`, `Eip1559`, `Eip4844`, `Eip7702`). Implements `Decodable` to parse the leading type byte (0x01/0x02/0x03/0x04). Mirrors `alloy_consensus::TxEnvelope`.
- [ ] Pick + implement second Alloy PR.
- [ ] Commit + log

**Friday — Third Alloy PR (medium)**
- [ ] Substantive PR — prefer something in `alloy-consensus` or `alloy-eips` since you've built mirrors.
- [ ] Commit + log

**Saturday — PR #3 submitted + Foundry intro**
- [ ] Submit PR #3.
- [ ] Clone foundry-rs/foundry, browse `forge` + `cast` crates. Note `cast` uses `alloy-primitives` directly — your `eth-primitives` could in principle drop into foundry.
- [ ] Commit notes.

**Sunday — Rest + Weekly Ritual**

---

### Week 8 — Foundry PR + revm familiarization + `eth-consensus` Receipt/Log

**Crate extended**: `eth-consensus` v0.2 → v0.3.

**Monday — Foundry issue hunt + claim**
- [ ] Browse Foundry issues, pick good first. Prefer issues touching `cast` (uses alloy primitives, area you know).
- [ ] Commit notes.

**Tuesday — First Foundry PR**
- [ ] Implement + submit.
- [ ] Commit + log.

**Wednesday — revm overview (read with `exec-vm` Phase 4 in mind)**
- [ ] Clone bluealloy/revm, read README + arch doc.
- [ ] Browse crate structure: `revm-primitives`, `revm-interpreter`, `revm-precompile`, `revm`. Note: your Phase 4 `exec-vm` is going to mirror this exact split.
- [ ] Cross-reference `revm-primitives::Database` trait against your `eth-storage-cache::StateCache` trait (Week 2). Note where they diverge — adjust `StateCache` if needed for Phase-4 compatibility (better to fix the trait now than later).
- [ ] Commit notes.

**Thursday — revm-interpreter + `Receipt` build**
- [ ] Read revm-primitives, compare with `eth-primitives` (note: revm depends on alloy-primitives — your eth-primitives will need to provide the same surface for exec-vm).
- [ ] Read revm-interpreter — opcode dispatch, gas metering. Trace ADD opcode end-to-end, document in `notes/`.
- [ ] **Build**: `crates/eth-consensus/src/receipt.rs` — `Receipt { status: Eip658Value, cumulative_gas_used: u64, logs: Vec<Log> }` mirroring `alloy_consensus::Receipt`. `ReceiptEnvelope` enum dispatching by tx type (Legacy/2930/1559/4844/7702). RLP derive.
- [ ] Commit + log.

**Friday — ME ch13 + `Log` + `Bloom`**
- [ ] ME ch13 full chapter. Walk evm.codes top 20 opcodes.
- [ ] **Build**: `crates/eth-consensus/src/log.rs` — `Log { address, topics: Vec<B256>, data: Bytes }` mirroring alloy. `bloom_filter(logs: &[Log]) -> Bloom` keccak-based bloom (uses `eth-primitives::Bloom = FixedBytes<256>`).
- [ ] Commit notes.

**Saturday — PR cleanup + tag `eth-consensus v0.3.0`**
- [ ] Address all reviewer feedback across open PRs.
- [ ] Tag `eth-consensus v0.3.0`. README documents Header, Tx*, Signed, TxEnvelope, Receipt, Log, fee math.
- [ ] Commit + log.

**Sunday — Rest + End Month 2 review**
- [ ] Update North Star M2 metrics.
- [ ] Target check: 3+ Alloy PRs opened (some merged), 1+ Foundry PR.
- [ ] Inheritance check: 5 crates production: `eth-primitives v0.2`, `eth-rlp v0.1`, `eth-storage-cache v0.1`, `eth-network-codec v0.1`, `eth-consensus v0.3`. Phase 1 Month 3 now seeds `exec-vm` and `eth-trie` (NOT throwaway).

---

## Month 3: `exec-vm` + `eth-trie` seeds (Weeks 9-12)

### Week 9 — `exec-vm` Phase-1 seed (NOT throwaway — same crate ships v1.0 in Phase 4)

**Mirror target**: `revm-interpreter` subset — `Stack`, `SharedMemory`, `Gas`, `instructions/arithmetic.rs`, `instructions/control.rs`, `instructions/host.rs`. Same crate split as revm.
**Crate created**: `crates/exec-vm/` — Phase 1 establishes the architectural skeleton; Phase 4 expands to full opcode coverage. NO discard.
**Inherits from**: `eth-primitives` (U256, Address, B256, Bytes), `eth-storage-cache` (StateCache trait for SSTORE/SLOAD), `eth-consensus` (Bytecode, TxEnvelope to source bytecode + tx env).
**Feeds into**: itself in Phase 4. Receipts written by `exec-vm` are encoded with `eth-rlp`. State reads/writes go through `eth-storage-cache::StateCache`.

> Same module names as revm: `interpreter/`, `instructions/`, `gas/`. When you start Phase 4 you're not creating a new crate — you're filling out modules in this one.

**Monday — `Stack` + arithmetic opcodes**
- [ ] **Build**: `crates/exec-vm/src/interpreter/stack.rs` — `Stack { data: Vec<U256> }` with 1024-deep limit, mirroring `revm_interpreter::Stack` (same SOA layout, same overflow semantics, same `push_b256`/`pop_b256` API). `push`, `pop`, `peek`, `swap`, `dup`.
- [ ] **Build**: `crates/exec-vm/src/instructions/arithmetic.rs` — ADD, SUB, MUL, DIV, MOD as separate functions taking `&mut Interpreter`. Wraps via `U256` (eth-primitives). Match revm gas costs (3 gas/op static).
- [ ] **Build**: `crates/exec-vm/src/interpreter/mod.rs` — `Interpreter { stack, memory, gas, pc, bytecode, return_data }` skeleton + `step()` dispatcher (match on `bytecode[pc]`).
- [ ] Commit + log.

**Tuesday — `SharedMemory` + control flow**
- [ ] **Build**: `crates/exec-vm/src/interpreter/memory.rs` — `SharedMemory` mirroring revm (single contiguous Vec shared across calls, with frame offsets). `mload`, `mstore`, `mstore8`, `resize` with quadratic gas calc.
- [ ] **Build**: `crates/exec-vm/src/instructions/control.rs` — JUMP, JUMPI, JUMPDEST, PC, STOP, INVALID. Valid-jumpdest analysis cached via `Bytecode::LegacyAnalyzed { jumpdests: Bitvec }` (Week 7's `eth-consensus::Bytecode`).
- [ ] **Build**: `crates/exec-vm/src/instructions/comparison.rs` — LT, GT, SLT, SGT, EQ, ISZERO, AND, OR, XOR, NOT.
- [ ] Commit + log.

**Wednesday — `Gas` + SSTORE/SLOAD against `StateCache`**
- [ ] **Build**: `crates/exec-vm/src/interpreter/gas.rs` — `Gas { limit, remaining, refunded }` with `record_cost`, `record_refund`. Mirrors revm.
- [ ] **Build**: `crates/exec-vm/src/instructions/host.rs` — SSTORE, SLOAD, BALANCE, EXTCODESIZE. These take a generic `Host` parameter. Define `Host` trait minimal subset that delegates to `eth_storage_cache::StateCache` (you wrote the trait Week 2 with this exact use in mind).
- [ ] 15-20 opcodes total. Test against hand-rolled bytecode programs (`60 01 60 02 01` = PUSH1 1 PUSH1 2 ADD).
- [ ] Run `cargo test -p exec-vm`. Tag `exec-vm v0.0.1` (Phase-1 seed marker).
- [ ] Commit + log.

**Thursday — First revm PR**
- [ ] Browse revm issues, pick good first, implement, submit. (Now you read revm with your `exec-vm` already in your head.)
- [ ] Commit + log.

**Friday — `eth-rlp` extension: typed envelopes**
- [ ] You ALREADY have `eth-rlp` from Week 5 — no throwaway. Today: extend it.
- [ ] **Extend**: `crates/eth-consensus/src/envelope.rs` — implement RLP for `TxEnvelope` with the leading type byte (0x01/0x02/0x03/0x04) per EIP-2718. Test against mainnet typed-tx test vectors.
- [ ] **Extend**: same for `ReceiptEnvelope`.
- [ ] Diff against `alloy-eips::eip2718` to confirm the framing matches.
- [ ] Commit + log.

**Saturday — More PRs (Alloy/revm)**
- [ ] Whichever is unblocked, push velocity. Prefer revm issues now that `exec-vm` is bootstrapped — you have shared vocabulary.
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 10 — `eth-trie` Phase-1 seed (NOT throwaway)

**Mirror target**: `alloy-trie` subset — `Nibbles`, `Node` enum (Empty/Leaf/Extension/Branch), `HashBuilder`, root hash via keccak. Plus `reth_trie::HashedPostState` (Week 11).
**Crate created**: `crates/eth-trie/` — Phase 3 `storage-trie` extends THIS crate, no fresh start.
**Inherits from**: `eth-primitives` (B256, Bytes, Bloom), `eth-rlp` (RLP encoding for trie nodes), `eth-storage-cache` (Week 11 will plug `TrieStorage` into `StateCache`).
**Feeds into**: `storage-trie` Phase 3 (full MPT with proofs, witnesses, MDBX backing); `consensus-engine` Phase 5 (state root verification on engine_newPayload).

**Monday — MPT theory + `Nibbles`**
- [ ] ethereum.org MPT docs + 2-3 blog explanations to triangulate.
- [ ] Draw extension/branch/leaf/hash node diagrams in `notes/`.
- [ ] **Build**: `crates/eth-trie/src/nibbles.rs` — `Nibbles(SmallVec<[u8; 64]>)` mirroring `alloy_trie::Nibbles`. Pack/unpack from `&[u8]`, convert to/from `B256` keys. Hex-prefix encoding for leaf vs extension nodes (the EIP-1186 nibble compression).
- [ ] Commit + log.

**Tuesday — `Node` enum + insert/get**
- [ ] **Build**: `crates/eth-trie/src/node.rs` — `Node` enum: `Empty`, `Leaf(LeafNode { key: Nibbles, value: Bytes })`, `Extension(ExtensionNode { key: Nibbles, child: NodeRef })`, `Branch(BranchNode { children: [NodeRef; 16], value: Option<Bytes> })`. Mirrors alloy-trie shapes.
- [ ] **Build**: `crates/eth-trie/src/storage.rs` — `TrieStorage` trait: `get_node(&self, hash: B256) -> Result<Node, TrieError>`, `put_node(&mut self, hash: B256, node: Node)`. Initial impl `MemoryStorage(HashMap<B256, Node>)`. (Phase 3 will add an mmap-backed impl that lives in `storage-trie`.)
- [ ] Insert + get on the trie via the storage trait. Test on `[("do", "verb"), ("dog", "puppy"), ("doge", "coin")]` — matches geth's trie test fixtures.
- [ ] Commit + log.

**Wednesday — `HashBuilder` + root hash**
- [ ] **Build**: `crates/eth-trie/src/hash_builder.rs` — `HashBuilder` mirroring `alloy_trie::HashBuilder`. Stream-builds the trie root by accepting sorted `(key, value)` pairs and emitting node hashes incrementally (keccak256 of RLP-encoded nodes via `eth-rlp`).
- [ ] Test against EIP-1186 simplest vectors + alloy-trie's test fixtures.
- [ ] Tag `eth-trie v0.0.1` (Phase-1 seed marker).
- [ ] Commit + log.

**Thursday — Second revm PR**
- [ ] Pick, implement, submit.
- [ ] Commit + log.

**Friday — Reth passive exposure (read with `eth-trie` mental model)**
- [ ] Clone paradigmxyz/reth, `cargo build --release`.
- [ ] Browse `reth/crates/trie`. Identify `HashBuilder`, `TrieWalker`, `HashedPostState`, `TrieUpdates`. Note where YOUR `eth-trie` will plug into the `MerkleStage` in Phase 3.
- [ ] Read 5 recently merged trie or storage PRs for style.
- [ ] Commit notes.

**Saturday — `peer-keepalive` state machine on `eth-network-codec`**
- [ ] Custom-Future state machine, but applied: build `peer-keepalive` ping/pong oscillator inside `eth-network-codec` (a `Future` that periodically sends `Ping` and times out the peer if no `Pong` within N seconds). Same shape as reth-eth-wire's keepalive.
- [ ] Property tests with `proptest` over the state transitions.
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 11 — Type-state on `eth-network-codec` + `HashedPostState` on `eth-trie` + reth survey

**Mirror targets**: `reth_eth_wire::EthStream` (type-state lifecycle), `reth_trie::HashedPostState`.

**Monday — Type-state pattern applied to `eth-network-codec`**
- [ ] Type-state, sealed trait, extension trait reading.
- [ ] **Refactor**: `crates/eth-network-codec/src/connection.rs` — introduce `Connection<S>` parameterized by phantom state types `Disconnected`, `Handshaking { hello_received: bool }`, `Established { protocol_version: u8 }`. Methods only available in correct states (e.g., `send_message` only on `Established`). Same type-state pattern reth-eth-wire uses for `P2PStream` ↔ `EthStream` lifecycle.
- [ ] Commit + log.

**Tuesday — Erigon staged sync (read with your crates as the substrate)**
- [ ] Read Erigon staged sync design doc.
- [ ] Browse `reth/crates/stages`. Map: headers → bodies → senders → execution → hashing → merkle.
- [ ] For each stage, name the `eth-*` crate of yours that would feed it: `headers/bodies` → eth-network-codec (data ingestion) + eth-consensus (Header/Body types); `senders` → eth-consensus signature recovery; `execution` → exec-vm + eth-storage-cache; `hashing/merkle` → eth-trie + eth-storage-cache.
- [ ] Commit notes — this map is the Phase 3-5 wiring diagram.

**Wednesday — `HashedPostState` + `TrieUpdates`**
- [ ] Browse `reth/crates/trie` source for `HashedPostState`, `TrieUpdates`.
- [ ] **Build**: `crates/eth-trie/src/hashed_state.rs` — `HashedPostState { accounts: HashMap<B256, Option<Account>>, storages: HashMap<B256, HashedStorage> }` mirroring `reth_trie::HashedPostState`. The "Option<Account>" represents deletion (None = self-destructed account).
- [ ] **Build**: `TrieUpdates { account_nodes: HashMap<Nibbles, BranchNodeCompact>, storage_tries: HashMap<B256, StorageTrieUpdates>, removed_nodes: HashSet<Nibbles> }`.
- [ ] These are the data structures `MerkleStage` consumes in reth — your Phase 3 `storage-trie` will produce them.
- [ ] Commit + log.

**Thursday — Third revm PR (medium difficulty)**
- [ ] Pick substantive issue, implement.
- [ ] Commit + log.

**Friday — Twitter + GitHub presence warm-up**
- [ ] First thoughtful technical reply on a reth/paradigm tweet.
- [ ] Star key repos (reth, revm, alloy, foundry, ethers-rs, erigon). Watch reth.
- [ ] Follow 20 more Ethereum infra engineers.
- [ ] Commit notes.

**Saturday — Outstanding PR cleanup + tag**
- [ ] Address all reviewer feedback across open PRs.
- [ ] Tag `eth-trie v0.1.0` (Nibbles + Node + HashBuilder + HashedPostState shipped). `eth-network-codec v0.2.0` (type-state Connection).
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 12 — Phase 1 close + Phase 2 prep

**Monday — MDBX overview (read with `eth-storage-cache` already in head)**
- [ ] Read libmdbx high-level README + libmdbx-rs crate skim.
- [ ] Note: your `eth-storage-cache` is the in-memory layer; MDBX in Phase 3 becomes the persistent backing for `storage-trie`. Sketch the layering: `StateCache` trait → `MdbxStateCache` (Phase 3 impl) → MDBX env.
- [ ] Commit notes.

**Tuesday — Reth architecture talk + consensus background**
- [ ] Watch any gakonst reth architecture talk on YouTube.
- [ ] Mastering Ethereum consensus chapter; understand The Merge at high level.
- [ ] Commit notes.

**Wednesday — Final Alloy/revm PR for Phase 1**
- [ ] Push one more PR over the finish line.
- [ ] Commit + log.

**Thursday — Maintainer tracker**
- [ ] Note which maintainers reviewed which PRs of yours.
- [ ] Identify mentor candidate (likely Matthias Seitz).
- [ ] Commit notes.

**Friday — Reth Telegram + Discord**
- [ ] Join reth Telegram, observe (don't post yet).
- [ ] Commit notes.

**Saturday — Phase 1 review**
- [ ] Verify shipped crates: `eth-primitives v0.2`, `eth-rlp v0.1`, `eth-storage-cache v0.1`, `eth-network-codec v0.2`, `eth-consensus v0.3`, `exec-vm v0.0.1` (seed), `eth-trie v0.1`, `eth-primitives-derive v0.1`. All with bench (where applicable), tests, README.
- [ ] Verify: 3-5 Alloy PRs, 2-3 revm PRs, 1-2 Foundry PRs (some merged).
- [ ] Verify: notes/01-07 all written, grounded in your real crate code (no toy programs).
- [ ] Verify: workspace `cargo test --workspace` is green; `cargo clippy --workspace -- -D warnings` is clean; `cargo +nightly miri test -p eth-primitives` clean.
- [ ] Phase 1 reflection in `progress.md`.
- [ ] Commit + log.

**Sunday — End Phase 1 ritual**
- [ ] Full Phase 1 assessment.
- [ ] Update North Star M3 metrics.
- [ ] **Note**: you enter Phase 2 with 6+ ecosystem PRs AND 8 production crates. The "Tiny EVM" and "Tiny MPT" tasks in Phase 2 (Weeks 17, 20) are NOT throwaways anymore — they become EXPANSION of `exec-vm` and `eth-trie`. Phase 2 should focus on revm depth + Foundry velocity + extending the existing crates.
- [ ] Phase 2 starts tomorrow.

---

# PHASE 2: ETHEREUM FOUNDATION + ECOSYSTEM PRs (Month 4-6)

## Month 4: Ethereum Protocol + Alloy PRs

### Week 13 — Ethereum fundamentals + `eth-consensus` deepening (`SealedHeader`, signer recovery, Account)

**Note**: most of Phase 1 Month 2 already covered Mastering Ethereum ch3-6 + Yellow Paper §4/§6 in passing. This week deepens those reads and converts each into an `eth-consensus` extension instead of a re-read.

**Monday — ME ch3 + `SealedHeader` finalize**
- [ ] ME ch3 (Clients) + ethereum.org "Intro to Ethereum" (skim).
- [ ] Run `reth` on Sepolia, observe sync logs.
- [ ] **Build**: `crates/eth-consensus/src/sealed.rs` — `SealedHeader { header: Header, hash: OnceLock<B256> }` mirroring `reth_primitives::SealedHeader`. Reuses `OnceLock` pattern from Week-4 `eth-primitives::Sealable`. `SealedHeader::hash_ref(&self) -> &B256` lazy-computes via `keccak256(rlp(header))`.
- [ ] Test: hash matches mainnet block hashes pulled via alloy-provider.
- [ ] Commit + log.

**Tuesday — ME ch4 + signer recovery**
- [ ] ME ch4 (Cryptography). Understand keccak256, secp256k1.
- [ ] **Build**: `crates/eth-consensus/src/recovery.rs` — `recover_signer(signature: &Signature, hash: B256) -> Result<Address, RecoveryError>` using `k256` directly (not via alloy-signer). Test against alloy-signer's output to confirm identical.
- [ ] **Build**: `Signed<T: Transaction>::recover_signer()` method on the `Signed` wrapper from Week 6.
- [ ] Commit + log.

**Wednesday — ME ch5-6 + `Block` + `Body`**
- [ ] ME ch5 (Wallets) + ch6 (Transactions).
- [ ] **Build**: `crates/eth-consensus/src/block.rs` — `Block { header: Header, body: BlockBody }`, `BlockBody { transactions: Vec<TxEnvelope>, ommers: Vec<Header>, withdrawals: Option<Vec<Withdrawal>> }`. Mirrors `alloy_consensus::Block` + `BlockBody`.
- [ ] **Build**: `SealedBlock` analogous to `SealedHeader`.
- [ ] Sign each tx type using your `eth-consensus::TxEip1559::signature_hash()` then alloy-signer; assert recovered address matches.
- [ ] Commit + log.

**Thursday — ME ch7 + `encode_tx` round-trip**
- [ ] ME ch7 (Smart Contracts Solidity).
- [ ] Deploy simple contract on Sepolia using Foundry.
- [ ] **Build**: `crates/eth-consensus/src/encode_tx.rs` — `encode_signed_tx(signed: &Signed<TxEnvelope>) -> Bytes` produces the raw bytes Foundry would broadcast. Send a tx using ONLY your bytes via `eth_sendRawTransaction` against Sepolia. Assert it lands on-chain.
- [ ] Commit + log.

**Friday — Yellow Paper §4 + `Account` + `StorageEntry`**
- [ ] Yellow Paper §4 (Block, State, Account). Understand state trie / account trie / storage trie distinction.
- [ ] **Build**: `crates/eth-consensus/src/account.rs` — `Account { nonce: u64, balance: U256, code_hash: B256, storage_root: B256 }` mirroring `alloy_consensus::TrieAccount`. RLP derive (this is the leaf value of the state trie).
- [ ] **Build**: `StorageEntry { key: B256, value: U256 }`.
- [ ] Note: this `Account` is the on-disk RLP form; the `Account` in `eth-storage-cache` (Week 2) is the in-memory form WITH inline code. Add `From`/`To` conversions between them.
- [ ] Draw state diagrams in `notes/`.
- [ ] Commit + log.

**Saturday — Yellow Paper §6 + intrinsic gas calculator**
- [ ] Yellow Paper §6 (Transaction Execution).
- [ ] **Build**: `crates/eth-consensus/src/gas.rs` — `intrinsic_gas(tx: &TxEnvelope, is_contract_creation: bool) -> u64`. 21000 base + zero/non-zero byte calldata + access list cost (EIP-2930) + auth list cost (EIP-7702). Test against revm's `validate_initial_tx_gas`.
- [ ] Tag `eth-consensus v0.4.0`.
- [ ] Commit notes.

**Sunday — Rest + Weekly Ritual**

---

### Week 14 — EIP deep dives via `eth-eips` extraction + medium Alloy PRs

**Note**: you already wrote EIP-1559/4844/7702 fee math + structures in Phase 1 (Weeks 6-7). This week pulls them into a dedicated `eth-eips` crate so they can be consumed independently of `eth-consensus` (mirrors how alloy splits `alloy-eips` from `alloy-consensus`).

**Crate created**: `crates/eth-eips/` — re-homes EIP fee math + structures.
**Mirror target**: `alloy-eips` (eip1559, eip2930, eip4844, eip7685, eip7702 modules).

**Monday — EIP-1559 deep + extract `eth-eips/eip1559`**
- [ ] Re-read EIP-1559 full spec + Paradigm's analysis posts (depth, not skim).
- [ ] **Refactor**: move `eth-consensus/src/eip1559.rs` to `crates/eth-eips/src/eip1559.rs`. Add `BaseFeeParams { max_change_denominator, elasticity_multiplier }` for chain-specific overrides (Optimism, Base differ). `eth-consensus` re-exports from `eth-eips`.
- [ ] Test base fee against mainnet, Optimism, Base genesis params.
- [ ] Commit + log.

**Tuesday — EIP-4844 (blobs) deep + KZG**
- [ ] Read EIP-4844 spec + Proto-Danksharding roadmap.
- [ ] **Refactor**: move blob fee math to `crates/eth-eips/src/eip4844.rs`. Add `BlobTransactionSidecar { blobs, commitments, proofs }` (mirrors alloy). Skeleton `KzgSettings` placeholder; full KZG verification deferred to Phase 5.
- [ ] Commit notes.

**Wednesday — EIP-7702 deep + `eth-eips/eip7702`**
- [ ] Re-read EIP-7702 spec.
- [ ] **Refactor**: move `Authorization` + `SignedAuthorization` from `eth-consensus` to `eth-eips/src/eip7702.rs`. Implement `recover_authority` using `eth-consensus::recovery`.
- [ ] Tag `eth-eips v0.1.0`.
- [ ] Commit notes.

**Thursday — Alloy issues scan (target `alloy-eips` now)**
- [ ] Browse alloy-rs/alloy issues. Filter `good first issue`, `help wanted`. PREFER issues in `alloy-eips` — you've now mirrored it module-for-module.
- [ ] Identify 3-5 candidate issues, pick one, claim.
- [ ] Commit notes.

**Friday — Medium-difficulty Alloy PR work**
- [ ] You're past the "first PR" phase — push for substantive change in `alloy-eips` or `alloy-consensus` (you have your own mirrors of both, so you can confidently propose API changes).
- [ ] Commit + log.

**Saturday — Alloy PR submitted**
- [ ] Finish implementation. `cargo fmt`, `cargo clippy --all`, `cargo nextest`. Open PR with motivation referencing your `eth-eips` design notes.
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 15 — EIP-7685 + EOF parser in `exec-vm` + more PRs

**Monday — Respond to Alloy PR reviews**
- [ ] Address reviewer feedback on prior PR; iterate until merge or close.
- [ ] Commit + log.

**Tuesday — EIP-7685 finalize in `eth-eips`**
- [ ] Re-read EIP-7685 spec.
- [ ] **Refactor**: move `Requests` from `eth-consensus` to `crates/eth-eips/src/eip7685.rs`. Add `compute_requests_hash(requests: &[Request]) -> B256`.
- [ ] Tag `eth-eips v0.2.0`.
- [ ] Commit notes.

**Wednesday — EOF parser deepening in `exec-vm`**
- [ ] Re-read EIP-3540, 3670, 4200, 4750.
- [ ] **Build**: `crates/exec-vm/src/eof/parser.rs` — full EOF container parser: magic bytes (0xEF00 01), version, type section, code sections, data section, container sections. Mirrors revm's `EofBody`.
- [ ] **Build**: `crates/exec-vm/src/eof/validate.rs` — EIP-3670 code validation pass (no truncated PUSH, no invalid opcodes in EOF, function ID range checks per EIP-4750).
- [ ] Test against revm's EOF test vectors.
- [ ] Commit notes.

**Thursday — Second Alloy PR**
- [ ] Pick next candidate issue, implement.
- [ ] Commit + log.

**Friday — Third Alloy PR work (medium)**
- [ ] Substantive contribution.
- [ ] Commit + log.

**Saturday — Third Alloy PR complete**
- [ ] Finish, submit.
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 16 — Alloy/Foundry PRs + `eth-rpc-types` extraction

**Crate created**: `crates/eth-rpc-types/` — JSON-RPC request/response types over `eth-consensus`.
**Mirror target**: `alloy-rpc-types-eth` (Block, Transaction, TransactionRequest, Filter, Log).

**Monday — `eth-rpc-types` + 4th Alloy PR**
- [ ] **Build**: `crates/eth-rpc-types/src/block.rs` — RPC `Block` (with hex-encoded fields via serde), `Transaction` (RPC view of `Signed<TxEnvelope>` with from/blockHash/blockNumber). Mirrors `alloy_rpc_types_eth::Block`. Used Phase 5 by `consensus-engine` for `eth_getBlockByHash`.
- [ ] Pick + implement 4th Alloy PR.
- [ ] Commit + log.

**Tuesday — Foundry codebase intro**
- [ ] Clone foundry-rs/foundry. Read Foundry Book briefly for user perspective.
- [ ] Browse `forge` crate source. Note: forge uses revm + alloy throughout — your `exec-vm` could in principle drop in.
- [ ] Commit notes.

**Wednesday — Foundry cast + `eth-rpc-types/filter`**
- [ ] Read cast crate source.
- [ ] **Build**: `crates/eth-rpc-types/src/filter.rs` — `Filter`, `FilterBlockOption`, `Topic`. Mirrors alloy. Used by `eth_getLogs` calls.
- [ ] Commit notes.

**Thursday — First Foundry PR**
- [ ] Browse Foundry issues, pick good first. Prefer cast issues (now you know the surface).
- [ ] Implement.
- [ ] Commit + log.

**Friday — Foundry PR complete + Alloy review responses**
- [ ] Finish Foundry PR. Address Alloy review feedback.
- [ ] Commit + log.

**Saturday — `eth-rpc-types/transaction_request` + 5th Alloy PR**
- [ ] **Build**: `crates/eth-rpc-types/src/transaction_request.rs` — `TransactionRequest` (the loose-typed builder for `eth_sendTransaction` / `eth_call`). Mirrors alloy.
- [ ] Tag `eth-rpc-types v0.1.0`.
- [ ] Either submit 5th Alloy PR or polish existing.
- [ ] Commit + log.

**Sunday — Rest + End Month 4 review**
- [ ] Update North Star M4 metrics.
- [ ] Target check: 5+ Alloy PRs opened, some merged. `eth-eips v0.2`, `eth-rpc-types v0.1`, `eth-consensus v0.4` in workspace.

---

## Month 5: EVM Deep Dive + revm PRs

### Week 17 — `exec-vm` expansion (Phase 1 seeded the crate; this week DOUBLES opcode coverage)

**This was "Tiny EVM throwaway" — now it's `exec-vm` Month-5 expansion.**

**Mirror target**: `revm-interpreter` `instructions/system.rs`, `instructions/host.rs`, `instructions/contract.rs`, `revm::context::evm_context`.
**Crate extended**: `exec-vm` v0.0.1 → v0.1.0.
**Inherits from**: Week 9 stack/memory/gas + Week 7 `Bytecode` + Week 13 `eth-consensus::Block`/`TxEnvelope` + Week 2 `eth-storage-cache::StateCache`.
**Feeds into**: itself in Phase 4 (full opcode coverage + journaling); `consensus-engine` Phase 5 for state transition.

**Monday — ME ch13 part 1 + `Env` types**
- [ ] ME ch13 first half. Understand EVM as stack machine. Memorize top 20 opcodes from evm.codes.
- [ ] **Build**: `crates/exec-vm/src/env.rs` — `Env { cfg, block, tx }`, `BlockEnv { number, timestamp, gas_limit, basefee, prevrandao, blob_excess_gas_and_price, beneficiary }`, `TxEnv { caller, gas_limit, gas_price, transact_to, value, data, nonce, access_list, blob_hashes, max_fee_per_blob_gas, authorization_list }`, `CfgEnv { chain_id, spec_id, ... }`. Mirrors `revm_primitives::Env`.
- [ ] **Build**: `From<&TxEnvelope> for TxEnv`, `From<&Header> for BlockEnv` — these conversions are why you needed `eth-consensus` first.
- [ ] Commit notes.

**Tuesday — ME ch13 part 2 + `instructions/system.rs`**
- [ ] ME ch13 second half. Understand gas metering basics + storage vs memory vs stack.
- [ ] **Build**: `crates/exec-vm/src/instructions/system.rs` — RETURN, REVERT, INVALID, SELFDESTRUCT (skeleton; full impl Phase 4 needs journal). Stack/memory glue around `return_data`.
- [ ] Commit notes.

**Wednesday — evm.codes deep + `instructions/stack.rs`**
- [ ] Walk every opcode on evm.codes. Practice reading bytecode.
- [ ] **Build**: `crates/exec-vm/src/instructions/stack.rs` — PUSH0..PUSH32, DUP1..DUP16, SWAP1..SWAP16, POP. All 96 stack opcodes.
- [ ] Manual trace simple contracts (`PUSH1 1 PUSH1 2 ADD MSTORE` etc.) through your interpreter — assert end memory + gas match revm.
- [ ] Commit + log.

**Thursday — `instructions/contract.rs` (CALL family)**
- [ ] **Build**: `crates/exec-vm/src/instructions/contract.rs` — CALL, CALLCODE, DELEGATECALL, STATICCALL, RETURN. Per EIP-150 63/64ths gas rule. Re-entrancy through the existing `Interpreter::step` dispatch (no journaling yet — that's Phase 4 Week 55).
- [ ] **Build**: `crates/exec-vm/src/instructions/create.rs` — CREATE, CREATE2 with init code analysis (EIP-3860 limit).
- [ ] Test: simple call-with-return via two hand-rolled bytecode programs.
- [ ] Commit + log.

**Friday — `instructions/host.rs` extension against `StateCache`**
- [ ] **Build**: extend `crates/exec-vm/src/instructions/host.rs` with BALANCE, EXTCODESIZE, EXTCODEHASH, EXTCODECOPY (needing `Bytes` clone — uses your `eth-primitives::Bytes`), BLOCKHASH, COINBASE, TIMESTAMP, NUMBER, DIFFICULTY/PREVRANDAO, GASLIMIT, CHAINID, SELFBALANCE, BASEFEE, BLOBHASH, BLOBBASEFEE.
- [ ] All routed through the `Host` trait → `eth-storage-cache::StateCache`.
- [ ] Commit + log.

**Saturday — `instructions/log.rs` + ethereum-tests subset green**
- [ ] **Build**: `crates/exec-vm/src/instructions/log.rs` — LOG0..LOG4. Emits `Log` (`eth-consensus::Log`) into the interpreter's pending log buffer.
- [ ] **Total opcode count**: 60+ now. Pass `ethereum/tests/GeneralStateTests/stArithmetic` + `stMemoryTest` subsets.
- [ ] Tag `exec-vm v0.1.0`. Update README documenting opcode coverage matrix.
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 18 — revm deep-read (now diffing against your `exec-vm`)

**Monday — revm overview + diff to `exec-vm`**
- [ ] Re-clone bluealloy/revm latest (you cloned earlier in Week 8).
- [ ] Read README, architecture doc fresh — now with 60+ opcodes implemented.
- [ ] **Diff log**: for each revm crate (revm-primitives, revm-interpreter, revm-precompile, revm), name 3 specific design choices that differ from your `exec-vm`. For each, decide: port revm's choice, keep yours, or document the trade. Save to `notes/08_revm_diff.md`.
- [ ] Commit notes.

**Tuesday — revm-primitives + `Database` trait alignment**
- [ ] Read revm-primitives source. Compare with `eth-primitives` AND `eth-storage-cache`.
- [ ] Critical alignment task: confirm your `StateCache` trait can be a `Database` for the unmodified revm. Adjust if not — this lets you swap revm in/out for `exec-vm` in benchmarks.
- [ ] Commit notes.

**Wednesday — revm-interpreter dispatch**
- [ ] Read revm-interpreter source. Study opcode dispatch mechanism (their match-on-byte vs your match-on-byte). Note gas calculation patterns.
- [ ] Identify the perf optimizations revm has that your `exec-vm` doesn't (instruction table indexing, gas precomputation, unsafe stack push). Add to a `EXEC_VM_PERF_BACKLOG.md` for Phase 4.
- [ ] Commit notes.

**Thursday — revm hot path + ADD trace**
- [ ] Trace ADD end-to-end through revm AND through your `exec-vm`. Note every function call in both. Compare overhead (revm has fewer indirection layers).
- [ ] Document in `notes/`.
- [ ] Commit + log.

**Friday — revm handler + precompile reading**
- [ ] Read revm `Handler` trait and `precompile` crate.
- [ ] Sketch in `notes/`: where would `Handler` plug into your `exec-vm`? Phase 4 Week 53 will add it.
- [ ] Commit notes.

**Saturday — First revm PR informed by the diff**
- [ ] Browse revm issues. Pick something where your `exec-vm` work gives you informed perspective (gas accounting, instruction edge cases).
- [ ] Implement, submit.
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 19 — revm PR velocity + `exec-vm` precompile skeleton

**Monday — Second revm PR**
- [ ] Pick and implement.
- [ ] Commit + log.

**Tuesday — revm PR review response + `exec-vm` precompile registry**
- [ ] Address reviewer feedback.
- [ ] **Build**: `crates/exec-vm/src/precompile/mod.rs` — `Precompile` trait (`fn run(input: &[u8], gas_limit: u64) -> Result<PrecompileOutput, PrecompileError>`), `PrecompileRegistry` map by `Address`. Mirror revm-precompile crate shape. Implement ECRECOVER first (uses `eth-consensus::recovery`). Other precompiles in Phase 4 Week 54.
- [ ] Commit + log.

**Wednesday — Third revm PR (medium)**
- [ ] Pick medium-difficulty issue, implement.
- [ ] Commit + log.

**Thursday — geth core/vm comparison**
- [ ] Read geth's core/vm package (as Go, but compare design with your exec-vm + revm).
- [ ] Add geth-specific design notes to `notes/08_revm_diff.md`.
- [ ] Commit notes.

**Friday — evmone comparison**
- [ ] Read evmone README + architecture.
- [ ] Note C++ optimization techniques (computed-goto dispatch, instruction stream pre-decode). Add to `EXEC_VM_PERF_BACKLOG.md`.
- [ ] Commit notes.

**Saturday — Continue revm PRs**
- [ ] Work on outstanding PRs or start new.
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 20 — `eth-trie` expansion (Phase 1 seeded the crate; this week adds storage abstraction, walker, proofs)

**This was "Tiny MPT throwaway" — now it's `eth-trie` Month-5 expansion.**

**Mirror target**: `alloy-trie` `proof::ProofRetainer`, `walker::TrieWalker`, `storage_root` helpers, `BranchNodeCompact`. Plus `reth_trie::StateRoot` orchestrator.
**Crate extended**: `eth-trie` v0.1 → v0.2.
**Inherits from**: Week 10 (Nibbles, Node, HashBuilder, MemoryStorage) + Week 11 (HashedPostState, TrieUpdates).
**Feeds into**: `storage-trie` Phase 3 (mmap-backed `TrieStorage` impl, full proof generation, witness building); `consensus-engine` Phase 5 (state root verification).

**Monday — MPT deeper theory + `BranchNodeCompact`**
- [ ] Re-read ethereum.org MPT docs + 2-3 blog explanations.
- [ ] **Build**: `crates/eth-trie/src/branch_compact.rs` — `BranchNodeCompact { state_mask, tree_mask, hash_mask, hashes, root_hash }` mirroring `reth_trie::BranchNodeCompact` (the on-disk-friendly compact branch representation reth uses for the intermediate trie state).
- [ ] Commit notes.

**Tuesday — `TrieStorage` abstraction over `StateCache`**
- [ ] **Refactor**: `crates/eth-trie/src/storage.rs` — split `TrieStorage` into `HashedNodeStorage` (storage-trie nodes by keccak-hash address) and `IntermediateStorage` (compacted nodes by Nibbles path, the reth pattern).
- [ ] **Build**: `CachedStorage<C: StateCache>` impl that delegates to `eth-storage-cache::StateCache` — proves the wiring works. Phase 3 swaps in MDBX.
- [ ] Commit + log.

**Wednesday — `TrieWalker` cursor**
- [ ] **Build**: `crates/eth-trie/src/walker.rs` — `TrieWalker<S: TrieStorage>` for streaming traversal. Yields `(Nibbles, Node)` in sorted order. Mirrors `reth_trie::TrieWalker`. Used Phase 3 by `MerkleStage` for incremental root computation.
- [ ] Commit + log.

**Thursday — `ProofRetainer` + EIP-1186 proofs**
- [ ] **Build**: `crates/eth-trie/src/proof/retainer.rs` — `ProofRetainer { targets: Vec<Nibbles>, proof_nodes: Vec<Bytes> }` that hooks into `HashBuilder` to capture nodes along target paths. Mirrors `alloy_trie::proof::ProofRetainer`.
- [ ] **Build**: `crates/eth-trie/src/proof/verify.rs` — `verify_proof(root: B256, key: &[u8], expected_value: Option<&[u8]>, proof: &[Bytes]) -> Result<(), ProofError>`. Mirrors alloy.
- [ ] Test against EIP-1186 test vectors and a captured mainnet `eth_getProof` response.
- [ ] Commit + log.

**Friday — `StateRoot` orchestrator**
- [ ] **Build**: `crates/eth-trie/src/state_root.rs` — `StateRoot<S: TrieStorage> { hashed_state: HashedPostState, prefix_set: PrefixSet, ... }` with `compute() -> Result<(B256, TrieUpdates), TrieError>`. Mirrors `reth_trie::StateRoot`. This is the heart of the `MerkleStage` — computing the post-block state root incrementally from the Week 11 `HashedPostState`.
- [ ] Test: reconstruct block 1 mainnet state root from the genesis state + block 1 changes.
- [ ] Commit + log.

**Saturday — `StorageRoot` + tag**
- [ ] **Build**: `crates/eth-trie/src/storage_root.rs` — `StorageRoot<S>` for per-account storage tries. Same pattern as `StateRoot` but scoped to one account's storage slots.
- [ ] Pass the simplest Ethereum trie test vectors end-to-end.
- [ ] Tag `eth-trie v0.2.0`. README documents the trie abstraction layers.
- [ ] Commit + log.

**Sunday — Rest + End Month 5 review**
- [ ] Update North Star M5 metrics.
- [ ] Inheritance check: `exec-vm v0.1` (60+ opcodes, precompile registry skeleton); `eth-trie v0.2` (Walker, ProofRetainer, StateRoot, StorageRoot). Phase 3 starts in Month 7 but the plumbing is ready.

---

## Month 6: MPT Understanding + First Maintainer Interactions

### Week 21 — `eth-rlp` extension + maintainer engagement (NO throwaway RLP)

**Note**: you ALREADY shipped `eth-rlp v0.1` in Week 5 + extended it through Phase 1. The original "throwaway RLP exercise" is REMOVED. Instead this week extends `eth-rlp` with reth-specific patterns and pushes more PRs.

**Monday — `eth-rlp` extension: trie-friendly encoding**
- [ ] Re-read RLP spec sections relevant to trie nodes (the `[ ... ]` framing of branch nodes).
- [ ] **Build**: `crates/eth-rlp/src/trie.rs` — `encode_branch_node`, `encode_extension_node`, `encode_leaf_node` helpers that produce the RLP form `eth-trie::HashBuilder` needs. Mirrors `alloy_trie` internal RLP helpers.
- [ ] **Build**: `EipTransactionRlp` helper for typed-tx envelope framing (already exists in `eth-consensus::envelope` — refactor to use `eth-rlp` helpers consistently).
- [ ] Commit + log.

**Tuesday — Reth RLP usage patterns + `eth-rlp` derive enhancements**
- [ ] Read reth's RLP usage patterns + `alloy-rlp` source freshly.
- [ ] **Extend**: `eth-rlp-derive` to support `#[rlp(trailing)]` (optional trailing fields, EIP-1559 → EIP-4844 backward compat). Mirrors alloy-rlp-derive.
- [ ] Tag `eth-rlp v0.2.0`.
- [ ] Commit + log.

**Wednesday — Fourth revm PR**
- [ ] Pick + implement.
- [ ] Commit + log.

**Thursday — Second Foundry PR**
- [ ] Pick + implement.
- [ ] Commit + log.

**Friday — Maintainer engagement**
- [ ] Identify which maintainers review which areas (alloy-eips: gakonst/yash; revm: rakita; reth-trie: rakita/mattsse).
- [ ] Engage thoughtfully in an issue discussion (substantive, NOT pester) — ideally referencing your `eth-eips` or `eth-trie` design as a counterpoint or supporting detail.
- [ ] Commit notes.

**Saturday — Consolidation**
- [ ] Review all open PRs across alloy/revm/foundry. Close out review comments.
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 22 — Staged sync architecture + `eth-stage` trait skeleton

**Crate created**: `crates/eth-stage/` — `Stage` trait + `Pipeline` skeleton mirroring `reth-stages-api`.
**Mirror target**: `reth_stages_api::Stage` trait, `reth_stages::Pipeline`.
**Feeds into**: Phase 3 `storage-trie` provides `MerkleStage` impl on top; `consensus-engine` Phase 5 may run stages live during sync.

**Monday — Erigon staged sync (deeper this time)**
- [ ] Re-read Erigon staged sync design doc with implementation eye.
- [ ] Understand stage concept, unwind, checkpoints.
- [ ] Commit notes.

**Tuesday — Reth stages source dive**
- [ ] Browse `reth/crates/stages` thoroughly. Read `Stage` trait signature exactly.
- [ ] **Build**: `crates/eth-stage/src/lib.rs` — `Stage` trait:
  ```
  trait Stage {
      fn id(&self) -> StageId;
      async fn execute(&mut self, input: ExecInput) -> Result<ExecOutput, StageError>;
      async fn unwind(&mut self, input: UnwindInput) -> Result<UnwindOutput, StageError>;
  }
  ```
  Exact shape match with reth (so reth could in principle plug in your stages later).
- [ ] **Build**: `Pipeline` runner that drives stages in order with checkpoint persistence (delegates checkpoints to `eth-storage-cache::StateCache` for now; Phase 3 swaps in MDBX).
- [ ] Commit + log.

**Wednesday — Stage dependency map**
- [ ] Map: headers → bodies → senders → execution → hashing → merkle. Draw diagram in `notes/`.
- [ ] **Build**: `crates/eth-stage/src/stages/headers.rs` — skeleton `HeaderStage` consuming `eth-network-codec::MessageStream`. Just the type wiring; Phase 3+ fills in.
- [ ] Commit + log.

**Thursday — More revm or Alloy PRs**
- [ ] Keep PR velocity up.
- [ ] Commit + log.

**Friday — Reth Telegram + Discord**
- [ ] Join reth main Telegram if haven't. Observe discussion style for a week before posting.
- [ ] Commit notes.

**Saturday — `eth-stage` consolidation + tag**
- [ ] Skeleton stages for `senders`, `execution`, `hashing`, `merkle` — each just calling the right downstream crate (execution → exec-vm; hashing/merkle → eth-trie).
- [ ] Tag `eth-stage v0.0.1`.
- [ ] Review Month 6 progress.
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 23 — Ready up for Phase 3 (`storage-trie` scaffold pre-wiring)

**Monday — Reth storage crate survey + `storage-trie` workspace setup**
- [ ] Browse reth/crates/storage (db, provider, codecs, api).
- [ ] **Build**: `crates/storage-trie/Cargo.toml` workspace member with deps on `eth-primitives`, `eth-rlp`, `eth-storage-cache`, `eth-trie`, `eth-consensus`. Empty `lib.rs` for now — full impl starts Week 25.
- [ ] Confirm `cargo build --workspace` succeeds with the empty crate.
- [ ] Commit notes + scaffold.

**Tuesday — MDBX first look + `Database` trait sketch**
- [ ] Read libmdbx high-level README.
- [ ] **Sketch**: in `storage-trie/src/lib.rs`, define `Database` trait shape (mirrors `reth_db::Database`) — to be implemented Week 25-28 over libmdbx-rs.
- [ ] Commit notes.

**Wednesday — More Alloy/revm PRs**
- [ ] Keep contribution streak.
- [ ] Commit + log.

**Thursday — Conference research**
- [ ] Research EthCC Paris 2027 + Devcon 2027 dates.
- [ ] Start budgeting.
- [ ] Commit notes.

**Friday — Relationship review**
- [ ] Update maintainer tracker. Note who has reviewed your PRs.
- [ ] Identify target mentor (likely Matthias Seitz).
- [ ] Commit notes.

**Saturday — Month 6 consolidation**
- [ ] Review all PRs merged / in review across Paradigm ecosystem.
- [ ] Check target: 5+ Alloy, 3+ revm, 2+ Foundry.
- [ ] Commit + log.

**Sunday — Rest + Weekly Ritual**

---

### Week 24 — Phase 2 close + Phase 3 prep

**Monday — Mastering Ethereum consensus + `consensus-engine` placeholder crate**
- [ ] ME consensus chapter. Understand The Merge at high level.
- [ ] **Build**: `crates/consensus-engine/Cargo.toml` workspace member with deps on `eth-primitives`, `eth-consensus`, `eth-network-codec`, `eth-rpc-types`, `eth-storage-cache`, `eth-trie`, `exec-vm`. Empty `lib.rs` for Phase 5 to fill.
- [ ] Confirms `cargo build --workspace` is green with all 12 crates.
- [ ] Commit notes.

**Tuesday — Reth architecture talk/video**
- [ ] Watch any available gakonst reth architecture talk on YouTube + any Paradigm Frontiers talk.
- [ ] Map every component you saw in the talks to one of YOUR workspace crates. Anything that doesn't map is something Phase 3-5 will need to add.
- [ ] Commit notes.

**Wednesday — Phase 3 scope + outline**
- [ ] Read Phase 3 section of this plan carefully (note: the README later in this file will need Phase 3 inheritance annotations added — schedule that for Week 24 Saturday).
- [ ] Outline approach for Month 7: `storage-trie` builds the MDBX-backed `Database`, wires it up as the persistent layer behind `eth-storage-cache::StateCache` and `eth-trie::TrieStorage`.
- [ ] Commit notes.

**Thursday — Phase 3 scaffolding + CI**
- [ ] `storage-trie` already created Week 23. Today: set up CI in `.github/workflows/ci.yml` running `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo nextest run --workspace`, `cargo +nightly miri test -p eth-primitives` weekly. Single CI for the whole workspace.
- [ ] README at workspace root listing all crates + their relationships (the dependency graph).
- [ ] Commit + log.

**Friday — Final Phase 2 PRs**
- [ ] Wrap up any outstanding PRs.
- [ ] Commit + log.

**Saturday — Phase 2 review + (todo) annotate later phases**
- [ ] Full assessment against Phase 2 exit criteria.
- [ ] **Verify shipped crates**: `eth-primitives v0.2`, `eth-rlp v0.2`, `eth-storage-cache v0.1`, `eth-network-codec v0.2`, `eth-consensus v0.4`, `eth-eips v0.2`, `eth-rpc-types v0.1`, `eth-trie v0.2`, `eth-stage v0.0.1`, `exec-vm v0.1`, `eth-primitives-derive v0.1`, `storage-trie` scaffold, `consensus-engine` scaffold.
- [ ] **TODO carry-forward to Phase 3 README pass**: when the Phase 3 plan still says "create crate `storage-trie`" it means "fill out the existing crate" — the scaffolding is already done.
- [ ] Update `progress.md` with Phase 2 summary.
- [ ] Commit + log.

**Sunday — End Phase 2 + Phase 3 prep**
- [ ] Full rest.
- [ ] Mentally prepare for Phase 3 intensity.
- [ ] Phase 3 starts tomorrow with all 12 workspace crates green.

---

# PHASE 3: STORAGE + TRIE DEEP DIVE (Month 7-12)

**Deliverable**: `storage-trie` v1.0 — MDBX-backed persistent state DB.

**Crate extended (NOT created)**: `storage-trie` was scaffolded Week 23; this phase fills out the implementation. Wherever older text below says "Create crate `storage-trie`" or "scaffold," read it as "extend the existing crate."

### Phase 3 inheritance map

`storage-trie` consumes the seed crates from Phase 1-2 and provides:
- a `Database` (MDBX-backed) that implements `eth-storage-cache::StateCache`
- a `TrieStorage` impl on top, replacing `MemoryStorage` from `eth-trie` Week 10
- the `MerkleStage` impl plugging into `eth-stage::Stage` from Week 22

| Module in `storage-trie` | Upstream mirror | Seed crate it leans on |
|--------------------------|-----------------|------------------------|
| `mdbx::env` (W25-27) | `reth-db::DatabaseEnv` over libmdbx-rs | reuses `eth-storage-cache::Page` (W2 Mon) for in-memory cache layer above MDBX |
| `mdbx::tx`, `mdbx::cursor` (W28-29) | `reth-db::Tx`, `reth-db::Cursor` | wraps libmdbx-rs |
| `tables::*` (W26, W30) | `reth-db-api::tables` | uses `eth-rlp` (W5) + `eth-consensus::*` (W6-13) for value codecs |
| `mpt::storage` (W31-32) | `reth-trie` MDBX backing | extends `eth-trie::TrieStorage` (W10) — NOT a fresh MPT |
| `mpt::pruning` (W34) | `reth-prune-types` | uses `eth-trie::HashedPostState` (W11) |
| `state_root::orchestrator` (W35) | `reth-trie::StateRoot` | extends `eth-trie::StateRoot` (W20) with persistent backing |
| `snapshot` (W36) | `reth-snapshot` | uses `eth-rlp` |
| `merkle_stage` (W39+) | `reth-stages::MerkleStage` | implements `eth-stage::Stage` (W22) |

**Reused upward**: every Phase 4-5 module that needs persistence pulls `storage-trie`. `exec-vm` Phase 4 calls into it via `eth-storage-cache::StateCache`. `consensus-engine` Phase 5 calls into it for chain head + canonical block lookup.

**Read existing daily tasks below with this lens**: when a day says "Implement Page" / "Implement node abstraction" / "Implement MPT," check whether the seed crate already provides it (it usually does) and the day's real work is "extend the seed implementation with persistent / mmap / proof-aware behavior." Cross-references are added inline at key weeks.

---

## Month 7: MDBX Foundation + First Reth Storage PRs

### Week 25 — MDBX documentation deep

**Monday — MDBX overview**
- [ ] Read libmdbx.dqdkfa.ru full overview
- [ ] Understand mmap-based design
- [ ] Commit notes

**Tuesday — MDBX internals: B-tree**
- [ ] Read MDBX B-tree structure section
- [ ] Compare with standard B+tree concepts
- [ ] Commit notes

**Wednesday — MDBX internals: MVCC**
- [ ] Read MVCC section
- [ ] Understand read tx during write tx
- [ ] Commit notes

**Thursday — MDBX internals: Durability**
- [ ] Read write-ahead logging / sync modes
- [ ] Understand crash recovery
- [ ] Commit notes

**Friday — MDBX cursor semantics**
- [ ] Read cursor documentation
- [ ] Understand efficient range scan
- [ ] Commit notes

**Saturday — libmdbx-rs source**
- [ ] Clone and read libmdbx-rs crate
- [ ] Understand Rust bindings to C library
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 26 — Research reth storage architecture

**Monday — Reth storage survey**
- [ ] Browse every crate in reth/crates/storage/
- [ ] Identify: db, provider, api, codecs
- [ ] Map high-level relationships
- [ ] Commit notes

**Tuesday — reth-db deep read part 1**
- [ ] Read reth-db/src/lib.rs
- [ ] Read table definitions
- [ ] Commit notes

**Wednesday — reth-db deep read part 2**
- [ ] Read transaction implementation
- [ ] Read cursor wrappers
- [ ] Commit notes

**Thursday — reth-provider read**
- [ ] Read reth-provider crate
- [ ] Understand abstraction over db
- [ ] Commit notes

**Friday — First reth storage PR hunt**
- [ ] Browse reth issues tagged storage
- [ ] Find good-first-issue or docs issue in storage area
- [ ] Claim issue
- [ ] Commit notes

**Saturday — First reth storage PR work**
- [ ] Implement
- [ ] Run full test suite
- [ ] Submit PR
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 27 — `storage-trie::mdbx`: mmap scaffold (extends `eth-storage-cache::Page` from W2)

**Inheritance**: `Page` already exists (`eth-storage-cache/src/page.rs`, W2 Mon). This week wraps libmdbx-rs and adds an mmap-backed page provider that yields `Page`s from a memory-mapped file.

**Monday — Research mmap in Rust**
- [ ] Read memmap2 crate docs
- [ ] Read about Rust + mmap safety considerations
- [ ] Commit notes

**Tuesday — mmap B-tree research (decide: thin wrapper vs from-scratch)**
- [ ] Research B-tree on mmap techniques
- [ ] Review academic papers if applicable
- [ ] AI-assisted research on MDBX design decisions
- [ ] **Decision**: thin wrapper over `libmdbx-rs` (ship faster, mirror reth-db) vs from-scratch B-tree (slower, deeper learning). Default to the wrapper unless you explicitly want a re-implementation milestone — record the choice in `notes/`.
- [ ] Commit notes

**Wednesday — Crate structure (extending the W23 scaffold)**
- [ ] Lay out `storage-trie/src/{mdbx,tables,mpt,state_root,merkle_stage,lib.rs}` — `mpt` and `state_root` will *re-export* from `eth-trie` (W10, W20) plus add persistent-backing impls.
- [ ] Sketch `Tx` / `Cursor` traits matching `reth-db-api`. NOTE: `Page` already exists in `eth-storage-cache`; do NOT redefine it here — import it.
- [ ] Commit + log

**Thursday — Page provider over mmap**
- [ ] Implement `MmapPageProvider` that returns `eth_storage_cache::Page` views over the mmap region (or a `MmapPage<'a>` borrowed variant if zero-copy reads are wanted).
- [ ] Allocation strategy: free-list backed by a header page.
- [ ] Unit tests
- [ ] Commit + log

**Friday — mmap wrapper + growth**
- [ ] Implement mmap-backed file wrapper with safe `remap` on growth (read memmap2's `MmapMut::flush_range` semantics).
- [ ] Unit tests
- [ ] Commit + log

**Saturday — Respond to reth PR review + continue crate**
- [ ] Address reth PR feedback
- [ ] Continue crate work
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 28 — `storage-trie` crate: B-tree core

**Monday — B-tree node design**
- [ ] Design leaf vs internal node layout
- [ ] Key/value storage in pages
- [ ] Commit + log

**Tuesday — B-tree insert**
- [ ] Implement insert with node splitting
- [ ] Unit tests
- [ ] Commit + log

**Wednesday — B-tree get**
- [ ] Implement lookup by key
- [ ] Range iteration
- [ ] Unit tests
- [ ] Commit + log

**Thursday — B-tree delete**
- [ ] Implement delete with node merging
- [ ] Unit tests
- [ ] Commit + log

**Friday — Second reth storage PR**
- [ ] Pick next issue
- [ ] Implement
- [ ] Commit + log

**Saturday — Crate polish**
- [ ] Document public API
- [ ] Run clippy, fmt
- [ ] Benchmark setup
- [ ] Commit + log

**Sunday — Rest + End Month 7 review**
- [ ] Update North Star M7 metrics

---

## Month 8: MVCC + Reth Storage Contribution Velocity

### Week 29 — MVCC in `storage-trie`

**Monday — MVCC design**
- [ ] Design MVCC approach (version chain vs copy-on-write)
- [ ] AI research: how MDBX implements MVCC
- [ ] Commit notes

**Tuesday — Read transaction**
- [ ] Implement read transaction with snapshot
- [ ] Unit tests for concurrent reads
- [ ] Commit + log

**Wednesday — Write transaction**
- [ ] Implement write transaction with copy-on-write
- [ ] Unit tests
- [ ] Commit + log

**Thursday — Concurrent read during write**
- [ ] Test read tx during write tx
- [ ] Verify snapshot isolation
- [ ] Commit + log

**Friday — Third reth storage PR**
- [ ] Pick medium-difficulty issue this time
- [ ] Implement
- [ ] Commit + log

**Saturday — Crate: durability**
- [ ] Implement fsync strategies
- [ ] Crash recovery basic
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 30 — Reth trie crate reading

**Monday — reth-trie overview**
- [ ] Browse reth/crates/trie
- [ ] Read top-level lib.rs
- [ ] Commit notes

**Tuesday — reth-trie node types**
- [ ] Read node definitions: extension, branch, leaf
- [ ] Commit notes

**Wednesday — reth-trie state root**
- [ ] Read state root computation
- [ ] Understand incremental computation
- [ ] Commit notes

**Thursday — reth-trie hashed state**
- [ ] Read hashed state abstraction
- [ ] Understand why hashing keys
- [ ] Commit notes

**Friday — First reth trie PR**
- [ ] Find trie-related issue
- [ ] Implement
- [ ] Commit + log

**Saturday — Crate: benchmarks**
- [ ] Add criterion benchmarks for B-tree ops
- [ ] Baseline vs sled, redb
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 31 — Persistent MPT in `storage-trie::mpt` (extends `eth-trie` W10/W20)

**Inheritance**: `Nibbles`, `Node` enum, `HashBuilder`, `TrieStorage`, `StateRoot`, `ProofRetainer` all exist in `eth-trie` (W10-11, W20). This week adds the **persistent** backing — a `MdbxTrieStorage` that implements `eth-trie::TrieStorage` against the W27-29 MDBX env. Do NOT reimplement nodes or hash builder.

**Monday — `MdbxTrieStorage` design**
- [ ] Design table layout for trie nodes (state nodes by hash, intermediate nodes by `Nibbles` path — same split reth uses).
- [ ] Implement `eth_trie::TrieStorage for MdbxTrieStorage` skeleton.
- [ ] Commit + log

**Tuesday — Wire `eth-trie::Node` to the table layout**
- [ ] RLP-encode/decode `Node` variants (Extension/Branch/Leaf) using `eth-rlp` from W5 — already in place via `eth-trie`. This week's job: cursor-based read path + dirty-set write path against MDBX.
- [ ] Commit + log

**Wednesday — Persistent insert via existing HashBuilder**
- [ ] Drive `eth_trie::HashBuilder` with `MdbxTrieStorage` as the read source. Persist new nodes through a write tx.
- [ ] Test: round-trip a small trie through MDBX and assert root matches the W10/W20 in-memory equivalent.
- [ ] Commit + log

**Thursday — Persistent get via existing walker**
- [ ] Drive `eth_trie::TrieWalker` against `MdbxTrieStorage` for path traversal.
- [ ] Range scans via MDBX cursor.
- [ ] Commit + log

**Friday — Root hash regression suite against `eth-trie` v0.2 fixtures**
- [ ] Re-run W20's Ethereum test vectors but with the persistent backing — assert byte-identical roots.
- [ ] Commit + log

**Saturday — Reth trie second PR**
- [ ] Continue trie contribution
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 32 — MPT proofs + more reth PRs

**Monday — MPT proof generation**
- [ ] Implement Merkle proof generation
- [ ] Unit tests
- [ ] Commit + log

**Tuesday — MPT proof verification**
- [ ] Implement standalone proof verification
- [ ] Unit tests
- [ ] Commit + log

**Wednesday — MPT delete**
- [ ] Implement MPT delete with rebalancing
- [ ] Commit + log

**Thursday — Ethereum test vectors**
- [ ] Integrate official Ethereum trie test vectors
- [ ] Pass basic suite
- [ ] Commit + log

**Friday — Reth PR volume**
- [ ] Another reth PR (storage or trie)
- [ ] Commit + log

**Saturday — Crate docs**
- [ ] Write comprehensive docs for all public APIs
- [ ] Examples in docs
- [ ] Commit + log

**Sunday — Rest + End Month 8 review**

---

## Month 9: Trie Depth + Staged Sync Understanding

### Week 33 — Advanced trie: path compression

**Monday — Path compression theory**
- [ ] Research path compression in tries
- [ ] Study Ethereum's specific approach
- [ ] Commit notes

**Tuesday — Implement path compression**
- [ ] Add path compression to crate MPT
- [ ] Verify correctness
- [ ] Commit + log

**Wednesday — Benchmark path compression**
- [ ] Benchmark with/without
- [ ] Document findings
- [ ] Commit + log

**Thursday — Reth staged sync survey**
- [ ] Browse reth/crates/stages deeply
- [ ] Identify every stage and its purpose
- [ ] Commit notes

**Friday — Stage dependencies diagram**
- [ ] Draw detailed flow diagram
- [ ] Document unwind paths
- [ ] Commit notes

**Saturday — Reth PR day**
- [ ] Another reth PR
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 34 — Pruning strategies

**Monday — Pruning research**
- [ ] Research Ethereum pruning modes: full, archive, pruned
- [ ] Understand state vs history pruning
- [ ] Commit notes

**Tuesday — Reth pruning code**
- [ ] Read reth pruner crate
- [ ] Understand configuration
- [ ] Commit notes

**Wednesday — Crate: pruning design**
- [ ] Design pruning strategy trait
- [ ] Plan integration with MPT
- [ ] Commit + log

**Thursday — Implement full pruning**
- [ ] Implement "full" retention (prune history beyond N blocks)
- [ ] Commit + log

**Friday — Implement archive mode**
- [ ] Keep everything mode
- [ ] Commit + log

**Saturday — Reth PR + integration testing**
- [ ] Reth PR in pruning area if possible
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 35 — State commitment deep

**Monday — State commitment theory**
- [ ] Read about state commitment schemes
- [ ] Understand MPT vs Verkle tradeoffs
- [ ] Commit notes

**Tuesday — Verkle Trees reading**
- [ ] Read Verkle Trees research posts (Vitalik, EF)
- [ ] Understand polynomial commitments at high level
- [ ] Commit notes

**Wednesday — Crate: incremental root**
- [ ] Design incremental state root computation
- [ ] Implement
- [ ] Commit + log

**Thursday — Benchmark incremental vs full**
- [ ] Benchmark root computation
- [ ] Document findings
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Continue PR velocity
- [ ] Commit + log

**Saturday — Crate polish**
- [ ] Clean up APIs
- [ ] Update docs
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 36 — Snapshot sync research

**Monday — Snapshot sync theory**
- [ ] Read about Ethereum snapshot sync
- [ ] Understand why different from state sync
- [ ] Commit notes

**Tuesday — Erigon snapshots**
- [ ] Read Erigon's snapshot strategy
- [ ] Note file format
- [ ] Commit notes

**Wednesday — Reth snapshots**
- [ ] Read reth's snapshot approach
- [ ] Commit notes

**Thursday — Crate: snapshot export**
- [ ] Design snapshot export format
- [ ] Implement basic export
- [ ] Commit + log

**Friday — Crate: snapshot import**
- [ ] Implement snapshot import
- [ ] Commit + log

**Saturday — End Month 9 PR push**
- [ ] 1-2 more reth PRs
- [ ] Commit + log

**Sunday — Rest + End Month 9 review**
- [ ] Update North Star M9 metrics
- [ ] Check: 15+ reth PRs, 10+ in storage/trie

---

## Month 10: Cross-Subsystem Storage PRs + Integration

### Week 37 — Medium-sized reth PRs

**Monday — Identify meaningful PR target**
- [ ] Look for enhancement issues (not just docs)
- [ ] Identify 1 medium PR candidate
- [ ] Design approach
- [ ] Commit notes

**Tuesday — Medium PR: implement**
- [ ] Start implementation
- [ ] Commit + log

**Wednesday — Medium PR: tests**
- [ ] Comprehensive testing
- [ ] Commit + log

**Thursday — Medium PR: benchmark**
- [ ] Add perf measurements if relevant
- [ ] Commit + log

**Friday — Medium PR: submit**
- [ ] Polish and submit
- [ ] Commit + log

**Saturday — Crate work**
- [ ] Continue `storage-trie` enhancements
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 38 — Codec / compression deep

**Monday — Reth codecs**
- [ ] Read reth codecs crate
- [ ] Understand compact encoding
- [ ] Commit notes

**Tuesday — Zstd compression in reth**
- [ ] Read how reth uses compression
- [ ] Commit notes

**Wednesday — Crate: codec support**
- [ ] Add compact encoding to crate
- [ ] Commit + log

**Thursday — Crate: compression**
- [ ] Optional compression layer
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Codec-related PR ideally
- [ ] Commit + log

**Saturday — Crate benchmarks**
- [ ] Bench compression tradeoffs
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 39 — Reth storage architecture contributions

**Monday — Read storage discussions**
- [ ] Read all recent GitHub discussions on storage
- [ ] Commit notes

**Tuesday — Substantive comment**
- [ ] Find appropriate discussion
- [ ] Write substantive technical comment
- [ ] Commit notes

**Wednesday — More reth PR**
- [ ] Continue velocity
- [ ] Commit + log

**Thursday — Crate: composition test**
- [ ] Integration test: B-tree + MPT + transaction combined
- [ ] Commit + log

**Friday — Crate: example**
- [ ] Write example showing typical usage
- [ ] Commit + log

**Saturday — Consolidation**
- [ ] Review everything
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 40 — Medium reth feature development

**Monday — Feature proposal**
- [ ] Identify storage improvement opportunity from own reading
- [ ] Draft proposal comment on GitHub
- [ ] Commit notes

**Tuesday — Discuss with maintainers**
- [ ] If got feedback, iterate
- [ ] If approved, start design
- [ ] Commit notes

**Wednesday — Feature implementation part 1**
- [ ] Start coding
- [ ] Commit + log

**Thursday — Feature implementation part 2**
- [ ] Continue
- [ ] Commit + log

**Friday — Feature: tests**
- [ ] Comprehensive tests
- [ ] Commit + log

**Saturday — Feature: submit**
- [ ] Submit PR
- [ ] Commit + log

**Sunday — Rest + End Month 10 review**

---

## Month 11: Feature Shipping + Crate v1.0

### Week 41 — Ship reth feature

**Monday — Address feature PR reviews**
- [ ] Iterate on reviews
- [ ] Commit + log

**Tuesday — More iteration**
- [ ] Address remaining feedback
- [ ] Commit + log

**Wednesday — Feature merged ideally**
- [ ] If merged, celebrate + blog draft
- [ ] If not, keep iterating
- [ ] Commit + log

**Thursday — Crate: performance pass**
- [ ] Profile `storage-trie`
- [ ] Identify hot paths
- [ ] Commit + log

**Friday — Crate: optimizations**
- [ ] Implement optimizations
- [ ] Commit + log

**Saturday — Another reth PR**
- [ ] Keep velocity
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 42 — Geth comparison study

**Monday — Geth core/state package**
- [ ] Read Geth's Go implementation
- [ ] Note architectural differences from reth
- [ ] Commit notes

**Tuesday — Geth core/trie**
- [ ] Read Geth trie implementation
- [ ] Commit notes

**Wednesday — Write comparison doc**
- [ ] Internal doc: reth vs Geth storage decisions
- [ ] Potential blog material
- [ ] Commit + log

**Thursday — Reth PR**
- [ ] Continue
- [ ] Commit + log

**Friday — Crate: fuzz targets**
- [ ] Add cargo-fuzz targets to crate
- [ ] Commit + log

**Saturday — Crate: property tests**
- [ ] Add proptest for MPT invariants
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 43 — Crate v1.0 preparation

**Monday — API review**
- [ ] Review all public APIs
- [ ] Stabilize naming, signatures
- [ ] Commit + log

**Tuesday — Documentation pass**
- [ ] Every public item has docs
- [ ] Top-level crate-level docs comprehensive
- [ ] Commit + log

**Wednesday — Examples expansion**
- [ ] Multiple examples in examples/
- [ ] Cover main use cases
- [ ] Commit + log

**Thursday — CI hardening**
- [ ] All CI checks pass
- [ ] Coverage reporting
- [ ] MSRV policy
- [ ] Commit + log

**Friday — README + design doc**
- [ ] Comprehensive README
- [ ] DESIGN.md explaining architectural choices (AI-researched)
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] More PR activity
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 44 — Crate v1.0 ship

**Monday — Final benchmarks**
- [ ] Comprehensive bench suite
- [ ] Compare against reth, sled, redb
- [ ] Commit + log

**Tuesday — Security review self-audit**
- [ ] Review unsafe blocks
- [ ] Review error handling
- [ ] Commit + log

**Wednesday — Crate v1.0 tag**
- [ ] Tag release
- [ ] Consider crates.io publication (optional)
- [ ] Commit + log

**Thursday — Blog: crate intro**
- [ ] If in writing mood, draft "Building storage-trie" post in your disruptor style
- [ ] No deadline
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Continue reth contributions
- [ ] Commit + log

**Saturday — Month 11 review**
- [ ] Assess crate quality
- [ ] Assess reth PR portfolio
- [ ] Commit + log

**Sunday — Rest + End Month 11 review**

---

## Month 12: Phase 3 Close + Phase 4 Prep

### Week 45 — Final reth storage feature

**Monday — Identify second feature**
- [ ] Find another meaningful opportunity
- [ ] Design
- [ ] Commit notes

**Tuesday — Implement**
- [ ] Code
- [ ] Commit + log

**Wednesday — Continue**
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Submit**
- [ ] Submit PR
- [ ] Commit + log

**Saturday — Iterate on reviews**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 46 — Recognition signals

**Monday — Review PRs of others**
- [ ] Start reviewing others' storage PRs substantively
- [ ] Commit notes

**Tuesday — Help newcomers**
- [ ] Answer questions in Telegram
- [ ] Commit notes

**Wednesday — More PR reviews**
- [ ] Build reviewing muscle
- [ ] Commit notes

**Thursday — Maintainer relationship check**
- [ ] Which maintainers have engaged with me?
- [ ] Update tracker
- [ ] Commit notes

**Friday — Active issue engagement**
- [ ] Participate in design discussions
- [ ] Commit notes

**Saturday — Another small reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 47 — revm preview for Phase 4

**Monday — revm architecture refresher**
- [ ] Re-read revm with Phase 3 eyes
- [ ] Note storage integration points
- [ ] Commit notes

**Tuesday — Identify revm learning gaps**
- [ ] Map what needs deep understanding in Phase 4
- [ ] Commit notes

**Wednesday — Reth evm crate**
- [ ] Read reth/crates/evm
- [ ] Understand reth's revm integration
- [ ] Commit notes

**Thursday — More reth PR**
- [ ] Continue
- [ ] Commit + log

**Friday — Crate maintenance**
- [ ] Any bug fixes on storage-trie
- [ ] Commit + log

**Saturday — Phase 4 prep (exec-vm already scaffolded — review state)**
- [ ] `exec-vm` was seeded W9 + extended W17. Today: re-read its README + opcode coverage matrix. Identify the gap between current state and Phase 4 v1.0 (full opcodes, journal, full precompiles, EOF, perf). Write a Phase 4 outline in `notes/`.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 48 — Phase 3 close

**Monday — Phase 3 reflection**
- [ ] Full assessment vs exit criteria
- [ ] Commit notes

**Tuesday — Metrics update**
- [ ] Update all North Star metrics
- [ ] Target check: 30 storage PRs, 1+ feature
- [ ] Commit notes

**Wednesday — Blog if in mood**
- [ ] Consider writing Phase 3 retrospective in disruptor style
- [ ] Focus on storage-trie design choices
- [ ] No pressure if not ready
- [ ] Commit + log

**Thursday — Relationship stock-take**
- [ ] Update maintainer tracker
- [ ] Identify mentor candidate (likely Matthias)
- [ ] Commit notes

**Friday — Final Phase 3 PRs**
- [ ] Wrap any outstanding
- [ ] Commit + log

**Saturday — Clean transition prep**
- [ ] Mental prep for Phase 4
- [ ] Storage maintenance minimum during Phase 4
- [ ] Commit notes

**Sunday — End Phase 3 rest**
- [ ] Full rest
- [ ] Phase 4 starts tomorrow

---

# PHASE 4: EXECUTION DEEP DIVE (Month 13-18)

**Deliverable**: `exec-vm` v1.0 — full revm-equivalent EVM with journal, all precompiles, EOF, perf-tuned dispatch.

**Crate extended (NOT created)**: `exec-vm` was seeded W9 (stack/memory/gas + 20 opcodes) and extended W17 (60+ opcodes, BlockEnv/TxEnv, precompile registry skeleton, ECRECOVER). It now exists at `~v0.1`. Wherever older text below says "scaffold exec-vm" or "Setup crate structure," read it as "extend the existing crate." Wherever older text says "implement arithmetic opcodes," check the W9/W17 coverage matrix first — those weeks may already be done.

### Phase 4 inheritance map

| Module in `exec-vm` | Upstream mirror | Existing seed |
|---------------------|-----------------|---------------|
| `interpreter::stack`, `memory`, `gas` | `revm-interpreter` core | shipped W9 |
| `instructions/{arithmetic,control,comparison,host,system,contract,create,log,stack}` | revm `instructions/` | W9 + W17 cover ~60 opcodes; Phase 4 finishes the set + tunes |
| `env::{BlockEnv,TxEnv,CfgEnv}` | `revm_primitives::Env` | shipped W17 Mon (uses `eth-consensus` conversions) |
| `eof::{parser,validate}` | revm EOF | shipped W15 (parser + validation skeleton) — Phase 4 adds full EIP-4750 control flow |
| `precompile::*` | `revm-precompile` | ECRECOVER shipped W19; Phase 4 adds sha256, ripemd, modexp, BN256, blake2f, KZG |
| `journal::{account,storage,checkpoint}` | `revm` journal | NEW in Phase 4 (the missing piece for nested calls + revert) |
| `dispatch::{match,jump_table}` | `revm-interpreter` instruction table | NEW in Phase 4 (perf milestone) |
| `Database` impl | `revm::Database` trait | shipped W2 as `eth-storage-cache::StateCache`; Phase 4 wires through `storage-trie::MdbxStateCache` from Phase 3 for end-to-end execution |

**Reused upward**: `consensus-engine` Phase 5 calls `exec-vm::Evm::transact` to execute payloads under `engine_newPayload`.

**Read existing daily tasks below with this lens**: any task labeled "Implement X opcode" should first check the W9/W17 coverage matrix; if X is already done, the day's real work is perf tuning, gas-schedule fork variants, or filling Cancun/Prague-only behavior. The Month 14 "complete opcode set" weeks become "fill the residual Cancun + Prague + EOF residual."

## Month 13: Revm Full Codebase + First revm Perf PRs

### Week 49 — Revm architecture deep

**Monday — Revm top-level**
- [ ] Re-read revm from top
- [ ] Map all crates
- [ ] Commit notes

**Tuesday — Revm interpreter core**
- [ ] Read revm-interpreter in full
- [ ] Focus on main execution loop
- [ ] Commit notes

**Wednesday — Revm Host trait**
- [ ] Read Host trait and implementations
- [ ] Understand decoupling from storage
- [ ] Commit notes

**Thursday — Revm Database trait**
- [ ] Read Database trait
- [ ] Note how it integrates with any storage
- [ ] Commit notes

**Friday — Revm precompiles**
- [ ] Read revm-precompiles crate
- [ ] Understand each precompile
- [ ] Commit notes

**Saturday — First revm perf-oriented PR**
- [ ] Find performance issue
- [ ] Implement
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 50 — Revm journaling

**Monday — Journaling design**
- [ ] Read revm-interpreter journal module
- [ ] Understand revert semantics
- [ ] Commit notes

**Tuesday — Nested checkpoints**
- [ ] Study nested call handling
- [ ] Commit notes

**Wednesday — State access patterns**
- [ ] Read state management in revm
- [ ] Map read/write patterns
- [ ] Commit notes

**Thursday — Second revm PR**
- [ ] Another contribution
- [ ] Commit + log

**Friday — `exec-vm`: align traits with revm `Database`/`Host`**
- [ ] `exec-vm` and its `Host` trait already exist (W9). Today: refactor signatures so that any `impl Database for T` from revm Just Works as `Host` for `exec-vm` (or vice versa). Goal: swap revm in/out of your exec-vm's tests with one type alias change.
- [ ] Commit + log

**Saturday — Interpreter loop refactor (consolidate W9/W17 dispatch)**
- [ ] The match-based dispatch from W9 + W17 is split across files; today consolidate into `interpreter/dispatch.rs` to set up Week 58's jump-table swap. Don't add new opcodes — refactor for clean perf swap.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 51 — Opcode coverage gap-fill (W9 + W17 already shipped ~60; complete the residual)

**Inheritance**: most basic opcodes are DONE (W9 Mon: ADD/SUB/MUL/DIV/MOD; W9 Tue: MSTORE/MLOAD/JUMP/JUMPI/JUMPDEST + comparison; W9 Wed: SSTORE/SLOAD; W17 Mon: env conversions; W17 Wed: PUSH0..PUSH32, DUP, SWAP, POP; W17 Thu: CALL family + CREATE; W17 Fri: BALANCE/EXTCODE*/BLOCKHASH/COINBASE/TIMESTAMP/NUMBER/CHAINID/SELFBALANCE/BASEFEE/BLOBHASH/BLOBBASEFEE; W17 Sat: LOG0..LOG4). This week fills only the missing arithmetic/bitwise pieces and Cancun-specific opcodes.

**Monday — Missing arithmetic: SDIV, SMOD, ADDMOD, MULMOD, EXP, SIGNEXTEND**
- [ ] These are NOT in W9. Implement now. Unit tests against revm's outputs for edge cases (negative wrap, overflow, EXP gas).
- [ ] Commit + log

**Tuesday — Missing bitwise: BYTE, SHL, SHR, SAR**
- [ ] Not in W9. Implement against revm fixtures.
- [ ] Commit + log

**Wednesday — KECCAK256 + missing call-frame envs**
- [ ] KECCAK256 (uses `eth-primitives::keccak256` already wired).
- [ ] CALLDATALOAD, CALLDATASIZE, CALLDATACOPY, CODESIZE, CODECOPY, RETURNDATASIZE, RETURNDATACOPY, GASPRICE, ORIGIN, CALLER, CALLVALUE — anything missing from W17 Fri's host pass.
- [ ] Commit + log

**Thursday — PREVRANDAO + DIFFICULTY post-Merge handling**
- [ ] Same opcode byte (0x44) — semantics differ pre/post-Merge based on `CfgEnv::spec_id`. Add fork-aware behavior.
- [ ] PC, MSIZE, GAS, JUMPDEST coverage check (mostly done in W9).
- [ ] Commit + log

**Friday — TLOAD/TSTORE (EIP-1153, Cancun)**
- [ ] Transient storage scoped to the call frame. NEW in Phase 4 — adds a `transient: HashMap<Address, HashMap<U256, U256>>` to the call-frame state; cleared on call exit. This is groundwork for the W55 journal.
- [ ] Commit + log

**Saturday — MCOPY (EIP-5656, Cancun) + opcode-coverage matrix audit**
- [ ] MCOPY copies memory regions. Quadratic memory gas already in W17 Tue.
- [ ] Run a script that diffs your opcode coverage table against revm's instruction table — every gap goes into the W53 follow-up.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 52 — `exec-vm` control flow + gas

**Monday — Gas accounting**
- [ ] Implement gas tracking
- [ ] Gas schedule for London
- [ ] Commit + log

**Tuesday — Memory expansion gas**
- [ ] Quadratic memory gas cost
- [ ] Commit + log

**Wednesday — Gas schedule per fork**
- [ ] Shanghai, Cancun schedules
- [ ] Commit + log

**Thursday — LOG opcodes**
- [ ] LOG0-LOG4
- [ ] Commit + log

**Friday — CREATE/CALL family**
- [ ] CREATE, CREATE2
- [ ] CALL, CALLCODE, DELEGATECALL, STATICCALL
- [ ] Commit + log

**Saturday — Another revm PR + reth storage PR**
- [ ] Maintain storage PR velocity
- [ ] revm contribution
- [ ] Commit + log

**Sunday — Rest + End Month 13 review**

---

## Month 14: Full Opcode Coverage + Precompiles

### Week 53 — Complete opcode set

**Monday — RETURN, REVERT, INVALID**
- [ ] Terminal opcodes
- [ ] Commit + log

**Tuesday — SELFDESTRUCT**
- [ ] Implement
- [ ] Commit + log

**Wednesday — EIP-1153 transient storage**
- [ ] TLOAD/TSTORE if not done
- [ ] Commit + log

**Thursday — Test vector integration**
- [ ] Integrate Ethereum execution test vectors
- [ ] Start passing basic suite
- [ ] Commit + log

**Friday — revm PR**
- [ ] Contribution
- [ ] Commit + log

**Saturday — Reth evm PR**
- [ ] Find reth evm crate issue
- [ ] Implement
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 54 — Precompiles in `exec-vm`

**Monday — ecrecover**
- [ ] Implement ecrecover precompile
- [ ] Test vectors
- [ ] Commit + log

**Tuesday — sha256, ripemd160, identity**
- [ ] Implement all three
- [ ] Commit + log

**Wednesday — modexp**
- [ ] Implement (using num-bigint)
- [ ] Commit + log

**Thursday — BN256 operations**
- [ ] BN256Add, BN256ScalarMul, BN256Pairing
- [ ] Use ark or similar
- [ ] Commit + log

**Friday — blake2f**
- [ ] Implement Blake2 F compression
- [ ] Commit + log

**Saturday — KZG precompile**
- [ ] Point evaluation precompile (EIP-4844)
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 55 — Journaling in `exec-vm`

**Monday — Journal design**
- [ ] Design journal structure
- [ ] Mirror revm's approach
- [ ] Commit + log

**Tuesday — Account journal**
- [ ] Track account changes with undo log
- [ ] Commit + log

**Wednesday — Storage journal**
- [ ] Track storage changes
- [ ] Commit + log

**Thursday — Nested checkpoints**
- [ ] Support nested call checkpoint/commit
- [ ] Commit + log

**Friday — Revert semantics tests**
- [ ] Test revert properly undoes all changes
- [ ] Commit + log

**Saturday — revm PR**
- [ ] Contribution
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 56 — Test vector push

**Monday — Ethereum tests repo**
- [ ] Integrate comprehensive test vectors
- [ ] Commit + log

**Tuesday — General state tests**
- [ ] Run general state tests
- [ ] Fix failures
- [ ] Commit + log

**Wednesday — More failure fixing**
- [ ] Continue
- [ ] Commit + log

**Thursday — Validator tests**
- [ ] Run validator test suite if applicable
- [ ] Commit + log

**Friday — reth PR**
- [ ] Storage or evm
- [ ] Commit + log

**Saturday — revm PR**
- [ ] Contribution
- [ ] Commit + log

**Sunday — Rest + End Month 14 review**
- [ ] Crate passing majority of test vectors

---

## Month 15: Dispatch Strategies + EthCC Prep

### Week 57 — EthCC Paris trip

**Monday-Friday — Conference attendance**
- [ ] Attend EthCC sessions
- [ ] Target: 1-on-1 with 3 reth core contributors
- [ ] Arrange meetings in advance via Twitter DM
- [ ] Attend side events (hacker houses, dinners)
- [ ] Take notes on talks

**Saturday — Travel home**
- [ ] Rest

**Sunday — Post-conference ritual**
- [ ] Update maintainer tracker with new connections
- [ ] Capture insights
- [ ] Follow-up emails/DMs

---

### Week 58 — Back to work: dispatch strategies

**Monday — Match dispatch (baseline)**
- [ ] Current implementation is match-based
- [ ] Baseline benchmark
- [ ] Commit + log

**Tuesday — Jump table research**
- [ ] Research function pointer jump tables
- [ ] Commit notes

**Wednesday — Implement jump table dispatch**
- [ ] Implement in `exec-vm`
- [ ] Feature-flagged
- [ ] Commit + log

**Thursday — Computed goto research**
- [ ] Research unsafe computed goto via asm
- [ ] Note portability tradeoffs
- [ ] Commit notes

**Friday — Benchmark match vs jump table**
- [ ] Measure instruction-level differences
- [ ] Commit + log

**Saturday — Dispatch strategy docs**
- [ ] Document findings (blog material in your style)
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 59 — evmone comparison

**Monday — evmone overview**
- [ ] Read evmone README deeply
- [ ] Note architecture choices
- [ ] Commit notes

**Tuesday — evmone basic interpreter**
- [ ] Read evmone basic mode
- [ ] Commit notes

**Wednesday — evmone advanced mode**
- [ ] Read advanced interpreter with caching
- [ ] Commit notes

**Thursday — Apply learnings to `exec-vm`**
- [ ] Implement any applicable optimizations
- [ ] Commit + log

**Friday — Benchmark exec-vm vs revm**
- [ ] Comprehensive benchmark
- [ ] Identify gaps
- [ ] Commit + log

**Saturday — revm PR**
- [ ] Another contribution
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 60 — Hot path optimization

**Monday — Profile `exec-vm`**
- [ ] Profile with perf or similar
- [ ] Identify hot spots
- [ ] Commit notes

**Tuesday — Stack optimization**
- [ ] Inline stack ops
- [ ] Commit + log

**Wednesday — Memory access**
- [ ] Optimize memory reads/writes
- [ ] Commit + log

**Thursday — Gas calculation**
- [ ] Optimize gas tracking in hot path
- [ ] Commit + log

**Friday — Benchmark improvements**
- [ ] Measure gains
- [ ] Commit + log

**Saturday — More reth PRs**
- [ ] Keep reth velocity up (10+ execution PRs target M18)
- [ ] Commit + log

**Sunday — Rest + End Month 15 review**

---

## Month 16: EOF + Integration with storage-trie

### Week 61 — EOF implementation

**Monday — EOF EIP deep re-read**
- [ ] Re-read EIP-3540, 3670
- [ ] Understand EOF container format
- [ ] Commit notes

**Tuesday — EOF validation**
- [ ] Implement stack validation per EIP-3670
- [ ] Commit + log

**Wednesday — Static relative jumps**
- [ ] Implement EIP-4200 opcodes
- [ ] Commit + log

**Thursday — Functions (EIP-4750)**
- [ ] Implement CALLF, RETF, JUMPF
- [ ] Commit + log

**Friday — EOF tests**
- [ ] Integrate EOF test vectors if available
- [ ] Commit + log

**Saturday — revm EOF PR**
- [ ] If revm has EOF issues, contribute
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 62 — `exec-vm` + `storage-trie` integration

**Monday — Integration design**
- [ ] Design how `exec-vm` uses `storage-trie` via Database trait
- [ ] Commit + log

**Tuesday — Implement integration**
- [ ] Wire up the two crates
- [ ] Commit + log

**Wednesday — Integration tests**
- [ ] End-to-end execution with real storage
- [ ] Commit + log

**Thursday — Benchmark integrated stack**
- [ ] Measure performance vs revm + reth storage
- [ ] Commit + log

**Friday — reth evm PR**
- [ ] Reth-side contribution
- [ ] Commit + log

**Saturday — Crate maintenance**
- [ ] storage-trie fixes if needed
- [ ] exec-vm polish
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 63 — Fuzz targets

**Monday — Fuzz setup**
- [ ] Setup cargo-fuzz
- [ ] First fuzz target on opcode sequences
- [ ] Commit + log

**Tuesday — Run fuzz, fix findings**
- [ ] Run fuzzer
- [ ] Address any crashes
- [ ] Commit + log

**Wednesday — More fuzz targets**
- [ ] Fuzz gas metering
- [ ] Fuzz call operations
- [ ] Commit + log

**Thursday — Differential fuzzing**
- [ ] Fuzz `exec-vm` vs revm for consistency
- [ ] Commit + log

**Friday — reth or revm PR**
- [ ] Contribution
- [ ] Commit + log

**Saturday — Docs pass**
- [ ] `exec-vm` documentation
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 64 — Revm performance PR push

**Monday — Identify revm perf opportunity**
- [ ] Deep profile revm in common scenarios
- [ ] Find improvement area
- [ ] Commit notes

**Tuesday — Design optimization**
- [ ] Plan approach
- [ ] Commit notes

**Wednesday — Implement**
- [ ] Code optimization
- [ ] Commit + log

**Thursday — Benchmark**
- [ ] Measure improvement
- [ ] Commit + log

**Friday — Submit revm PR**
- [ ] Clean PR
- [ ] Commit + log

**Saturday — Respond to reviews**
- [ ] Iterate
- [ ] Commit + log

**Sunday — Rest + End Month 16 review**

---

## Month 17: Architectural Discussions + Reth evm Features

### Week 65 — Architectural engagement

**Monday — GitHub discussions**
- [ ] Browse ongoing execution-layer architecture discussions
- [ ] Commit notes

**Tuesday — Substantive comment**
- [ ] Write a substantive architectural comment
- [ ] Commit notes

**Wednesday — Proposal draft**
- [ ] Draft a small design proposal for reth evm
- [ ] Commit notes

**Thursday — Submit proposal**
- [ ] Post as GitHub discussion
- [ ] Commit notes

**Friday — Engage discussion**
- [ ] Respond to feedback
- [ ] Commit notes

**Saturday — Reth PR**
- [ ] Storage or evm
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 66 — Reth evm feature

**Monday — Feature identification**
- [ ] Find meaningful reth evm improvement
- [ ] Design
- [ ] Commit notes

**Tuesday — Implementation**
- [ ] Start coding
- [ ] Commit + log

**Wednesday — Continue**
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Submit**
- [ ] PR ready
- [ ] Commit + log

**Saturday — Another storage PR (maintain velocity)**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 67 — exec-vm v1.0 prep

**Monday — API stabilization**
- [ ] Review all public APIs
- [ ] Freeze signatures
- [ ] Commit + log

**Tuesday — Docs pass**
- [ ] Every item documented
- [ ] Examples
- [ ] Commit + log

**Wednesday — Final benchmarks**
- [ ] Comprehensive suite
- [ ] Commit + log

**Thursday — DESIGN.md**
- [ ] Document architectural decisions
- [ ] AI-researched insights from revm
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Commit + log

**Saturday — Crate polish**
- [ ] Final cleanup
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 68 — exec-vm v1.0 ship

**Monday — Tag v1.0**
- [ ] Tag release
- [ ] Commit + log

**Tuesday — Blog if ready**
- [ ] Consider writing exec-vm intro blog in your style
- [ ] No pressure
- [ ] Commit + log

**Wednesday — Reth feature iteration**
- [ ] Address reviews on feature PR
- [ ] Commit + log

**Thursday — More reth**
- [ ] Continue velocity
- [ ] Commit + log

**Friday — Reviews given**
- [ ] Review 5+ others' PRs substantively
- [ ] Commit notes

**Saturday — Month 17 close**
- [ ] Commit + log

**Sunday — Rest + End Month 17 review**

---

## Month 18: Phase 4 Close + Consensus Prep

### Week 69 — Final execution PRs

**Monday — Final feature push**
- [ ] Last medium-sized feature
- [ ] Commit + log

**Tuesday — Implementation**
- [ ] Commit + log

**Wednesday — Tests + submit**
- [ ] Commit + log

**Thursday — Reviews**
- [ ] Commit + log

**Friday — Another small PR**
- [ ] Commit + log

**Saturday — Close outstanding work**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 70 — Consensus layer preview

**Monday — Ethereum consensus overview**
- [ ] Read Ethereum consensus layer intro
- [ ] Understand proof-of-stake at high level
- [ ] Commit notes

**Tuesday — Engine API spec preview**
- [ ] Read Engine API specification at high level
- [ ] Commit notes

**Wednesday — Lighthouse survey**
- [ ] Browse Lighthouse code at a high level
- [ ] Commit notes

**Thursday — Reth engine crate preview**
- [ ] Browse reth/crates/engine
- [ ] Commit notes

**Friday — Reth consensus crate preview**
- [ ] Browse reth/crates/consensus
- [ ] Commit notes

**Saturday — Phase 5 prep (consensus-engine already scaffolded W24 Mon)**
- [ ] `consensus-engine` is a workspace member with deps wired since W24. Today: re-read its empty `lib.rs`, sketch the module layout for Phase 5 in `notes/` (engine_api, fork_choice, payload_builder, jwt, builder_api, state_root_validator).
- [ ] Identify which `eth-*` crates each module will import. Confirm the dependency graph builds.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 71 — Phase 4 reflection

**Monday — Full Phase 4 assessment**
- [ ] Check exit criteria
- [ ] Commit notes

**Tuesday — Metrics**
- [ ] Update North Star M18 metrics
- [ ] Check 20+ execution PRs
- [ ] Commit notes

**Wednesday — Relationship update**
- [ ] Which maintainers engaged? Depth?
- [ ] Update tracker
- [ ] Commit notes

**Thursday — Blog consideration**
- [ ] Consider Phase 4 retrospective
- [ ] No deadline
- [ ] Commit notes

**Friday — Wrap**
- [ ] Close outstanding PRs
- [ ] Commit + log

**Saturday — Rest prep**
- [ ] Light day
- [ ] Commit notes

**Sunday — Rest**

---

### Week 72 — Transition week

**Monday — Mental prep Phase 5**
- [ ] Read Phase 5 section of plan
- [ ] Outline Month 19
- [ ] Commit notes

**Tuesday — Reading list for consensus**
- [ ] Compile reading list
- [ ] Commit notes

**Wednesday — Reach out to Lighthouse folks**
- [ ] If any connections, warm up
- [ ] Commit notes

**Thursday — Maintenance on previous crates**
- [ ] storage-trie, exec-vm bug fixes
- [ ] Commit + log

**Friday — Final exec-vm polish**
- [ ] Any remaining items
- [ ] Commit + log

**Saturday — Month 18 close**
- [ ] Final PRs
- [ ] Commit + log

**Sunday — Rest**
- [ ] Phase 5 starts tomorrow

---

# PHASE 5: CONSENSUS + ENGINE API (Month 19-24)

**Deliverable**: `consensus-engine` v1.0 + end-to-end integration of all 13 workspace crates capable of syncing Sepolia.

**Crate extended (NOT created)**: `consensus-engine` was scaffolded W24 Mon with deps on `eth-primitives`, `eth-consensus`, `eth-network-codec`, `eth-rpc-types`, `eth-storage-cache`, `eth-trie`, `exec-vm`. Phase 5 fills out the impl. Wherever older text below says "Create `consensus-engine` crate scaffold" or "Start crate skeleton," read it as "extend the existing crate."

### Phase 5 inheritance map

| Module in `consensus-engine` | Upstream mirror | Existing seed it composes |
|------------------------------|-----------------|---------------------------|
| `engine_api::server` (W74-75) | `reth-rpc-engine-api` | `eth-network-codec::Codec` (W3) for JSON-RPC framing; `eth-rpc-types` (W16) for request/response types |
| `engine_api::jwt` (W74) | `reth-rpc-builder` JWT layer | new this phase |
| `engine_api::new_payload` (W75) | reth `EngineApiHandler` | calls `exec-vm::Evm::transact` (Phase 4) + `storage-trie::MdbxStateCache` (Phase 3) |
| `state_root_validator` (W77) | `reth-engine-tree::StateRootProvider` | uses `eth-trie::StateRoot` (W20) on persistent backing |
| `fork_choice` (W78) | `reth-engine-primitives::ForkchoiceState` | uses `eth-primitives::ChainHead` SeqLock (W4) for hot-read tracking |
| `block_tree` (W80) | `reth-engine-tree::tree` | uses `eth-storage-cache::ShardedCache` (W2) for branch caching |
| `reorg::rollback` (W79) | `reth-blockchain-tree::reorg` | uses `eth-trie::HashedPostState` (W11) reverse-applied |
| `builder_api::*` (W82-83) | `reth-mev-rpc` / `mev-boost` Builder API | uses `eth-rpc-types` (W16) |
| `pipeline_runtime` (W85) | `reth-stages::Pipeline` | uses `eth-stage::Pipeline` (W22) |

**Three-crate integration target (W85 Sepolia sync)**: `consensus-engine` orchestrates `eth-network-codec` → block ingestion → `eth-stage::Pipeline` (driving `exec-vm` for execution + `storage-trie` for persistence + `eth-trie::StateRoot` for verification) → `engine_api` for CL coordination. The Sepolia sync is the proof that all 13 crates compose.

**Read existing daily tasks below with this lens**: tasks like "consensus-engine scaffold" (W74 Fri) are already done. "Wire up with storage-trie" (W75 Fri) really means "write the adapter from `consensus-engine::PayloadStore` to `storage-trie::MdbxStateCache`," not new crate creation.

## Month 19: Engine API Deep Dive

### Week 73 — Engine API specification

**Monday — Engine API full read part 1**
- [ ] Read Engine API spec sections 1-3
- [ ] Commit notes

**Tuesday — Engine API full read part 2**
- [ ] Read sections 4-6
- [ ] Commit notes

**Wednesday — newPayload deep**
- [ ] Study newPayload V1, V2, V3, V4
- [ ] Commit notes

**Thursday — forkchoiceUpdated deep**
- [ ] Study fcU variants
- [ ] Commit notes

**Friday — getPayload deep**
- [ ] Study getPayload variants
- [ ] Commit notes

**Saturday — JWT auth**
- [ ] Study JWT authentication used by Engine API
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 74 — Reth engine crate

**Monday — reth-engine structure**
- [ ] Browse reth/crates/engine
- [ ] Map files
- [ ] Commit notes

**Tuesday — Engine tree**
- [ ] Read engine tree implementation
- [ ] Understand block tree for forks
- [ ] Commit notes

**Wednesday — Payload builder**
- [ ] Read reth payload builder
- [ ] Commit notes

**Thursday — First engine PR**
- [ ] Find docs or small fix
- [ ] Commit + log

**Friday — `consensus-engine::engine_api` module skeleton (crate already exists)**
- [ ] Create `consensus-engine/src/engine_api/{mod.rs,server.rs,types.rs}`. Define the `EngineApi` trait with the V1-V4 method signatures. Wire `eth-network-codec::Codec` for JSON-RPC framing.
- [ ] Re-export `eth-rpc-types` request/response types where applicable.
- [ ] Commit + log

**Saturday — JWT auth in `consensus-engine::engine_api::jwt`**
- [ ] Implement HS256 JWT auth middleware (per Engine API spec). Test against a fixture token from a Lighthouse deployment.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 75 — `consensus-engine`: core methods

**Monday — newPayload implementation**
- [ ] Implement newPayload V3 handler
- [ ] Commit + log

**Tuesday — Payload validation**
- [ ] Block header validation
- [ ] Commit + log

**Wednesday — forkchoiceUpdated**
- [ ] Implement fcU handler
- [ ] Commit + log

**Thursday — getPayload**
- [ ] Implement getPayload
- [ ] Commit + log

**Friday — Storage + engine integration**
- [ ] Wire up with storage-trie
- [ ] Commit + log

**Saturday — Engine + exec-vm integration**
- [ ] Execute payload using exec-vm
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 76 — Lighthouse CL perspective

**Monday — Lighthouse code survey**
- [ ] Browse Lighthouse execution interaction layer
- [ ] Commit notes

**Tuesday — Lighthouse Engine API client**
- [ ] Read Lighthouse's side of Engine API
- [ ] Commit notes

**Wednesday — Prysm perspective**
- [ ] Read Prysm equivalent (less depth)
- [ ] Commit notes

**Thursday — CL/EL lifecycle**
- [ ] Map full CL/EL communication flow
- [ ] Commit notes

**Friday — Another reth engine PR**
- [ ] Continue velocity
- [ ] Commit + log

**Saturday — Crate: connection handling**
- [ ] Websocket/HTTP Engine API transport
- [ ] Commit + log

**Sunday — Rest + End Month 19 review**

---

## Month 20: Full Engine API + State Transition Validation

### Week 77 — State transition validation

**Monday — STF theory**
- [ ] Read state transition function theory
- [ ] Commit notes

**Wednesday — Consensus rules in execution**
- [ ] What execution layer validates per consensus rules
- [ ] Commit notes

**Wednesday — Block validation**
- [ ] Implement block validation in crate
- [ ] Commit + log

**Thursday — Receipt validation**
- [ ] Receipt consistency checks
- [ ] Commit + log

**Friday — Gas limit validation**
- [ ] Block gas limit checks
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Engine or consensus area
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 78 — Fork choice integration

**Monday — Fork choice theory**
- [ ] Read fork choice rule (LMD-GHOST, Casper FFG)
- [ ] Commit notes

**Tuesday — Reth fork choice code**
- [ ] Read reth's fork choice handling
- [ ] Commit notes

**Wednesday — Crate: fork choice**
- [ ] Implement fork choice processing
- [ ] Commit + log

**Thursday — Safe/finalized tracking**
- [ ] Track safe, finalized, head blocks
- [ ] Commit + log

**Friday — Reorg detection**
- [ ] Detect reorgs from fork choice updates
- [ ] Commit + log

**Saturday — More reth PRs**
- [ ] Consensus or engine
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 79 — Reorg handling

**Monday — Reorg theory**
- [ ] Deep understand reorg handling in execution
- [ ] Commit notes

**Tuesday — State rollback**
- [ ] Implement state rollback on reorg
- [ ] Leverage storage-trie snapshots
- [ ] Commit + log

**Wednesday — Receipt reindexing**
- [ ] Handle receipt/log reindexing
- [ ] Commit + log

**Thursday — Transaction re-pool**
- [ ] Handle moving txs back to mempool on reorg
- [ ] Commit + log

**Friday — Reorg integration tests**
- [ ] Test various reorg scenarios
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 80 — Multi-branch state

**Monday — Multi-branch theory**
- [ ] Understand maintaining state across forks
- [ ] Commit notes

**Tuesday — Branch state design**
- [ ] Design multi-branch state in crate
- [ ] Commit + log

**Wednesday — Implement**
- [ ] Code branch state management
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Integration with exec-vm**
- [ ] Speculative execution across branches
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + End Month 20 review**

---

## Month 21: PBS + Builder API + Invalid Payload Handling

### Week 81 — Invalid payload handling

**Monday — Invalid payload scenarios**
- [ ] Catalog all invalid payload cases from spec
- [ ] Commit notes

**Tuesday — Invalid header**
- [ ] Handle invalid headers
- [ ] Commit + log

**Wednesday — Invalid transactions**
- [ ] Handle invalid tx in payload
- [ ] Commit + log

**Thursday — Invalid state root**
- [ ] Handle state root mismatch
- [ ] Commit + log

**Friday — Latest valid hash logic**
- [ ] Implement LVH tracking
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 82 — PBS introduction

**Monday — PBS theory**
- [ ] Read PBS (Proposer-Builder Separation) spec
- [ ] Commit notes

**Tuesday — MEV-Boost architecture**
- [ ] Read MEV-Boost architecture
- [ ] Commit notes

**Wednesday — Builder API spec**
- [ ] Read Builder API specification
- [ ] Commit notes

**Thursday — Builder API in reth**
- [ ] Check reth's builder API support
- [ ] Commit notes

**Friday — Crate: Builder API compat**
- [ ] Design builder API support
- [ ] Commit + log

**Saturday — Implementation start**
- [ ] Begin builder API endpoints
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 83 — Builder API implementation

**Monday — Header submissions**
- [ ] Implement header submission flow
- [ ] Commit + log

**Tuesday — Block submissions**
- [ ] Implement block submission flow
- [ ] Commit + log

**Wednesday — Builder client**
- [ ] Implement builder client perspective
- [ ] Commit + log

**Thursday — Builder integration tests**
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Commit + log

**Saturday — Flashbots docs**
- [ ] Study Flashbots additional docs
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 84 — Test harness

**Monday — Test harness design**
- [ ] Design CL/EL test harness
- [ ] Commit notes

**Tuesday — Deterministic CL**
- [ ] Implement mock CL for testing
- [ ] Commit + log

**Wednesday — Scenario DSL**
- [ ] Define DSL for test scenarios
- [ ] Commit + log

**Thursday — Reorg scenarios**
- [ ] Reorg simulation tests
- [ ] Commit + log

**Friday — Engine API conformance**
- [ ] Run crate against spec conformance tests
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + End Month 21 review**

---

## Month 22: Cross-Subsystem Features + Integration Push

### Week 85 — Three-crate integration push

**Monday — Integration architecture**
- [ ] Design toy execution client using all 3 crates
- [ ] Commit + log

**Tuesday — Boot sequence**
- [ ] Implement node startup
- [ ] Commit + log

**Wednesday — Engine API → execution → storage flow**
- [ ] End-to-end flow
- [ ] Commit + log

**Thursday — Sync from testnet**
- [ ] Attempt sync from Sepolia using own stack
- [ ] Commit + log

**Friday — Debug failures**
- [ ] Fix issues
- [ ] Commit + log

**Saturday — More debugging**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 86 — Cross-subsystem reth feature

**Monday — Feature identification**
- [ ] Find reth feature touching engine + storage
- [ ] Commit notes

**Tuesday — Design**
- [ ] Commit notes

**Wednesday — Implementation**
- [ ] Commit + log

**Thursday — Continue**
- [ ] Commit + log

**Friday — Tests**
- [ ] Commit + log

**Saturday — Submit**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 87 — PR reviews velocity

**Monday — Review 2 PRs substantively**
- [ ] Commit notes

**Tuesday — Review 2 more**
- [ ] Commit notes

**Wednesday — Review discussion comments**
- [ ] Engage design discussions
- [ ] Commit notes

**Thursday — Reth PR**
- [ ] Commit + log

**Friday — Review 2 more**
- [ ] Commit notes

**Saturday — Crate maintenance**
- [ ] All three crates
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 88 — Devcon attendance

**Monday-Friday — Devcon (dates vary)**
- [ ] Attend Devcon
- [ ] Meet maintainers in person
- [ ] Side events
- [ ] Notes

**Saturday — Travel home**

**Sunday — Post-conference ritual**
- [ ] Update tracker
- [ ] Follow-ups

---

## Month 23: Mentorship + RFC Work

### Week 89 — RFC consideration

**Monday — Identify RFC opportunity**
- [ ] Find area needing design doc
- [ ] Commit notes

**Tuesday — Draft RFC**
- [ ] Write initial draft
- [ ] Commit notes

**Wednesday — Refine RFC**
- [ ] Iterate
- [ ] Commit notes

**Thursday — Post RFC**
- [ ] Post as GitHub discussion
- [ ] Commit notes

**Friday — Respond to feedback**
- [ ] Engage commenters
- [ ] Commit notes

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 90 — Mentorship practice

**Monday — Identify newcomer**
- [ ] Find newer contributor in Telegram
- [ ] Offer help on their first PR
- [ ] Commit notes

**Tuesday — Help them**
- [ ] Pair review
- [ ] Commit notes

**Wednesday — Another mentee**
- [ ] Help another newcomer
- [ ] Commit notes

**Thursday — Crate PR**
- [ ] Reth contribution
- [ ] Commit + log

**Friday — Consensus-engine v1.0 prep**
- [ ] API review
- [ ] Commit + log

**Saturday — Docs pass**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 91 — consensus-engine v1.0 ship

**Monday — Final benchmarks**
- [ ] Commit + log

**Tuesday — DESIGN.md**
- [ ] Commit + log

**Wednesday — Release tag**
- [ ] v1.0 tag
- [ ] Commit + log

**Thursday — Integration example**
- [ ] Full 3-crate example
- [ ] Commit + log

**Friday — Blog consideration**
- [ ] If ready, draft consensus-engine post
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 92 — Recognition push

**Monday — Engage major discussions**
- [ ] Architecture-level discussion contributions
- [ ] Commit notes

**Tuesday — More PR reviews**
- [ ] 5+ substantive reviews
- [ ] Commit notes

**Wednesday — Second RFC**
- [ ] If applicable, another design proposal
- [ ] Commit notes

**Thursday — Reth PR**
- [ ] Commit + log

**Friday — Maintainer touch points**
- [ ] Engage each target maintainer at least once
- [ ] Commit notes

**Saturday — End Month 23**
- [ ] Commit + log

**Sunday — Rest + End Month 23 review**

---

## Month 24: Phase 5 Close + Reassessment

### Week 93 — Final feature push

**Monday — Feature identification**
- [ ] Last major reth feature for Phase 5
- [ ] Commit notes

**Tuesday — Implementation**
- [ ] Commit + log

**Wednesday — Continue**
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Submit**
- [ ] Commit + log

**Saturday — Reviews**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 94 — Final PR push

**Monday — PR volume**
- [ ] Multiple smaller PRs
- [ ] Commit + log

**Tuesday — Continue**
- [ ] Commit + log

**Wednesday — Reviews given**
- [ ] 5+ reviews
- [ ] Commit notes

**Thursday — Continue**
- [ ] Commit + log

**Friday — Final PRs**
- [ ] Commit + log

**Saturday — Wrap up**
- [ ] All outstanding items
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 95 — Reassessment preparation

**Monday — Data collection**
- [ ] Count all PRs merged
- [ ] List all features shipped
- [ ] Update maintainer tracker
- [ ] Commit notes

**Tuesday — Signal assessment**
- [ ] Any approaches from companies?
- [ ] Any mentions by maintainers?
- [ ] Commit notes

**Wednesday — Three crates assessment**
- [ ] Quality of each crate
- [ ] Commit notes

**Thursday — Energy assessment**
- [ ] Sustainability check
- [ ] Commit notes

**Friday — Post-reth pull**
- [ ] Chronicle Queue / matching engine urges?
- [ ] Commit notes

**Saturday — Market state**
- [ ] Crypto cycle, Rust infra hiring climate
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 96 — Month 24 Decision

**Monday — Path A analysis**
- [ ] Extend reth for 6-12 months
- [ ] What would this look like?
- [ ] Commit notes

**Tuesday — Path B analysis**
- [ ] Pivot to post-reth (Chronicle Queue etc)
- [ ] What would this look like?
- [ ] Commit notes

**Wednesday — Path C analysis**
- [ ] Catch-up if slipped
- [ ] What would this look like?
- [ ] Commit notes

**Thursday — Decision**
- [ ] Choose path based on signals
- [ ] Commit notes

**Friday — Month 25-30 rough plan**
- [ ] Draft based on chosen path
- [ ] Commit notes

**Saturday — Phase 5 close**
- [ ] Full Phase 5 review
- [ ] Commit + log

**Sunday — End 24-month plan**
- [ ] Celebrate milestone
- [ ] Prep for new chapter

---

# Tracking Sections

## Daily Log

```
| Date | Hrs | Phase | Focus | Output | Energy |
|------|-----|-------|-------|--------|--------|
```

Fill one row per day.

---

## North Star Metrics

| Metric | M6 | M12 | M18 | M24 |
|--------|----|----|----|----|
| Paradigm ecosystem PRs merged | 10 | 25 | 50 | 80 |
| Reth PRs merged | 0 | 15 | 35 | 60 |
| Storage/Trie PRs | 0 | 10 | 20 | 30 |
| Execution PRs (revm + reth evm) | 0 | 3 | 10 | 20 |
| Consensus/Engine PRs | 0 | 0 | 3 | 10 |
| PR reviews given (substantive) | 0 | 10 | 40 | 100 |
| Features led end-to-end | 0 | 0 | 1 | 3 |
| Production crates shipped | 0 | 1 | 2 | 3 |
| Direct relationships with maintainers | 1 | 3 | 5 | 8 |
| Conferences attended | 0 | 0 | 0 | 1 |

---

## Open Questions

*Running list. Close what resolves, carry what doesn't. If survives 2 weeks, dedicated slot.*

- [ ]
- [ ]

---

## Maintainer Relationship Tracker

| Name | Role | First interaction | Last interaction | Depth 0-5 | Notes |
|------|------|------------------|-----------------|-----------|-------|
| Matthias Seitz (mattsse_) | Core reth | — | — | 0 | Target primary mentor |
| Georgios Konstantopoulos (gakonst) | CTO Paradigm | — | — | 0 | Ecosystem leader |
| Dan Cline | Core reth | — | — | 0 | Storage/trie area |
| Oliver Nordbjerg | Core reth | — | — | 0 | — |
| Roman Krasiuk | Core reth | — | — | 0 | — |
| Dragan Rakita | Core reth / revm author | — | — | 0 | Critical Phase 4 |
| joshieDo | Core reth | — | — | 0 | — |

Depth: 0=none, 1=reviewed PR, 2=back-and-forth, 3=tags you for area reviews, 4=DM relationship, 5=co-design

---

## Risk Register

| Risk | Prob | Mitigation | Status |
|------|------|-----------|--------|
| Rust foundations extend Phase 1 | 70% | Budget +4 wk, weekly monitor | — |
| Reth PR cycles slow | 80% | Many small PRs in parallel | — |
| Reth major arch change | 60% | Telegram presence, release notes | — |
| Multiplier spike | 70% | Coast mode, 4h floor | — |
| Burnout M8-14 | 80% | Rest weeks, energy monitor | — |
| Conference budget delay | 30% | Year 1 savings earmarked | — |
| Family emergency | 40% | Accept slip, adjust | — |
| Motivation dip M12-14 | 70% | Pre-commit to Phase 4 | — |
| Crypto winter | 40% | Storage/exec portable | — |
| Crate scope creep | 60% | Lock scope at phase start | — |

---

## Principles

1. Deliverables over hours. 5h target, 4h floor. Done at 3h → rest. Stuck at 6h → diagnose.
2. Three production crates are the real output. Phase 1-2 exercises are disposable.
3. Depth over breadth. 3 subsystems mastered > 6 shallow.
4. Code reading > code writing in Phase 3+.
5. Ship imperfect > perfect never.
6. AI leverage for architecture research.
7. Blogging optional. Write when material earns essay.
8. Post-reth trajectory deferred, not forgotten.
9. Conferences non-negotiable Year 2.
10. Multiplier is infrastructure. Coast mode.
11. Energy is the only real budget. Sleep 7h, fitness 3x/week minimum.
12. Month 24 is a decision point.
13. Scope discipline on crates. No feature creep.

---

*Plan is a skeleton. Adjust weekly. Review monthly. Recalibrate quarterly. Reassess at Month 24.*