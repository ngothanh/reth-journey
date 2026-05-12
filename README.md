# Reth + Tempo Contributor — 24-Month Daily Plan

> **Start**: 2026-04-25
> **Horizon**: 24 months, reassess at Month 24
> **Commitment**: 5h/day × 6 days/week = 30h/week
> **Schedule**: Mon-Sat work, Sunday rest + weekly ritual

**Strategic frame (revised M5/2026)**: Tempo is built on the Reth SDK (Apache/MIT, `github.com/tempoxyz/tempo`), incubated by Stripe + Paradigm, with overlapping maintainer set (gakonst, rakita, joshieDo). The Reth bet is unchanged — Tempo is leverage on top, not a parallel track. ~2-3 hrs/wk M6-M12, 4-5 hrs/wk M13-M18, 5-7 hrs/wk M19-M24, all sourced from the existing "Cross-ecosystem breadth" slot (Move dropped, Solana demoted to 1 hr/wk). 30 hrs/wk ceiling preserved. The Reth deliverables stay primary; Tempo crates layer on top.

**Final-phase deliverables (Reth primary)**:
- `storage-trie` crate (Month 7-12) — reth storage + trie re-implementation
- `exec-vm` crate (Month 13-18) — revm + reth evm re-implementation
- `consensus-engine` crate (Month 19-24) — reth consensus + engine API re-implementation

**Tempo crate deliverables (additive)**:
- `tempo-tx-envelope` (M14, ships W66) — mirrors `TempoTransaction` (type 0x76). Extends `eth-consensus::TxEnvelope`.
- `tempo-evm-ext` (scaffold W54, v0.1.0 W91) — mirrors `TempoEvm` extending revm. Extends `exec-vm` v1.0.
- `tempo-payment-lane` (scaffold W83, v0.1.0 W91) — lane reservation strategy. Extends `consensus-engine` v1.0.

**Phase 1-2 seed crates** (built during Rust mastery, NOT throwaway — extended in later phases):
- `eth-primitives` (Week 1-4) — mirrors `alloy-primitives`. Newtypes, hashing, atomic-cached hashes.
- `eth-rlp` (Week 5) — mirrors `alloy-rlp`. Encodable/Decodable traits + derive macro.
- `eth-storage-cache` (Week 2) — mirrors `revm::CacheDB` + reth in-memory state cache.
- `eth-network-codec` (Week 3) — mirrors `reth-eth-wire` framing layer.
- `eth-consensus` (Week 6-13) — mirrors `alloy-consensus`. Header, tx envelopes, EIP fee math.
- `eth-trie` (Week 10, 20) — mirrors `alloy-trie`. Nibbles, HashBuilder, proof retainer.
- `exec-vm` seed (Week 9, 17) — mirrors `revm-interpreter` subset. Same crate that ships v1.0 in Phase 4.

---

## How to Use

Check off tasks as completed. One day = one section. If you fall behind, adjust forward — don't delete. Sunday ritual reviews the week.

**Daily 5h block structure**:
- 15 min warm-up (review notes, set intent)
- 90 min deep work 1
- 10 min break
- 90 min deep work 2
- 15 min wrap-up (commit, log, questions)

**Tempo additions are marked `[Tempo]` at the start of each added bullet.** They are additive — never replace a primary Reth task. If a Reth task is over time budget, drop the Tempo bullet for that day.

---

## Curriculum Principle: Inherited Exercises

**No throwaways.** Every exercise in Phase 1 and Phase 2 builds a real component in a workspace crate that mirrors a specific upstream module in alloy / reth / revm AND is reused in a later phase.

Workspace layout (built incrementally):

```
crates/
  eth-primitives/      Week 1-4    -> mirrors alloy-primitives
  eth-storage-cache/   Week 2      -> mirrors revm::CacheDB + reth in-memory state cache
  eth-network-codec/   Week 3      -> mirrors reth-eth-wire framing
  eth-rlp/             Week 5      -> mirrors alloy-rlp + alloy-rlp-derive
  eth-consensus/       Week 6-13   -> mirrors alloy-consensus
  exec-vm/             Week 9, 17, Phase 4   -> mirrors revm-interpreter + revm
  eth-trie/            Week 10, 20, Phase 3  -> mirrors alloy-trie
  storage-trie/        Phase 3     -> owns mmap/MDBX-backed state DB
  consensus-engine/    Phase 5     -> owns engine API + fork choice
  tempo-tx-envelope/   W66         -> mirrors tempo::primitives::TempoTransaction
  tempo-evm-ext/       W54, W91    -> mirrors TempoEvm extending revm
  tempo-payment-lane/  W83, W91    -> lane reservation strategy
```

If you ever feel an exercise is "just to learn the syntax," stop — find the matching alloy/reth/revm module instead.

---

# PHASE 1: RUST MASTERY (Month 1-3)

> **No Tempo entries this phase.** Rust mastery is the prerequisite.

## Month 1: Rust Core (Weeks 1-4)

### Week 1 — Ownership/borrowing/lifetimes via `eth-primitives` foundation

**Mirror target**: `alloy-primitives` (Address, B256, U256, Bytes, FixedBytes)
**Crate created**: `crates/eth-primitives/`.
**Feeds into**: every later week.

**Pre-week setup**: ✓ already done

**Monday — Skim the Book chs 1-9, write nothing + workspace scaffold**
- [X] Speed-read Book ch1-3 (~30 min)
- [X] Speed-read Book ch5-9 (~90 min): structs, enums, modules, collections, error handling
- [X] Skip beginner Rustlings: `intro`, `variables`, `functions`, `if`, `primitive_types`, `strings`, `vecs`, `hashmaps`, `modules`
- [X] Write `notes/01_kotlin_to_rust_delta.md`
- [X] Create workspace `Cargo.toml` (resolver = "2", members = ["crates/*"])
- [X] Create `crates/eth-primitives` with Cargo.toml, src/lib.rs, src/error.rs skeleton
- [X] Read alloy-primitives top-level lib.rs + map the 8 types you'll build this week
- [X] Commit + log

**Tuesday — Book ch4 + `FixedBytes<const N: usize>`**
- [X] Book ch4.1 (Ownership) — read twice
- [X] Book ch4.2 (References and Borrowing) — read twice
- [X] Book ch4.3 (Slices)
- [X] Rustlings `move_semantics` (all 6)
- [X] **Build**: `crates/eth-primitives/src/fixed_bytes.rs` — `FixedBytes<const N: usize>([u8; N])` with Copy, Default, From, AsRef, AsMut, Deref, PartialEq, Hash. repr(transparent).
- [X] Test: zero-init, equality, slice access, hash stability. Match alloy-primitives test cases.
- [X] Borrow-checker drill: `fn split(&mut self) -> (&mut [u8], &mut [u8])` resolved via split_at_mut. Document in `notes/02_borrow_checker_errors.md`.
- [X] Commit + log

**Wednesday — Lifetimes + `Bytes` + `BytesView<'a>`**
- [X] Book ch10.3 (Lifetimes) — read twice
- [X] Watch Crust of Rust: Lifetime Annotations (full)
- [X] **Build**: `crates/eth-primitives/src/bytes.rs` — `Bytes(Arc<[u8]>)` cheap-clone wrapper. Methods: new, from_static, slice, len, is_empty, as_ref.
- [X] **Build**: `BytesView<'a>(&'a [u8])` borrowed views. Add `Bytes::view(&self) -> BytesView<'_>`.
- [X] Implement `From<Vec<u8>>`, `From<&'static [u8]>`, `Display` (lowercase hex with 0x prefix).
- [X] Document lifetime elision rules in `notes/03_lifetimes.md`.
- [X] Commit + log

**Thursday — Traits + `Address` + `B256` + sealed-trait pattern**
- [X] Book ch10.1 + ch10.2 (Generics, Traits)
- [X] Rustlings `generics`, `traits` — all
- [X] Read about orphan rule, coherence, sealed traits
- [X] **Build**: `crates/eth-primitives/src/address.rs` — `pub type Address = FixedBytes<20>;` + EIP-55 checksum encoding.
- [X] **Build**: `crates/eth-primitives/src/aliases.rs` — `B256 = FixedBytes<32>`, `B64 = FixedBytes<8>`.
- [X] **Build**: sealed-trait pattern — `mod private { pub trait Sealed {} }`. impl Sealed for Address, B256, Bytes.
- [X] Write 4 functions over the sealed trait (&dyn, Box<dyn>, impl, <T:>). Observe what compiles.
- [X] Notes in `notes/04_traits.md`: static vs dynamic dispatch.
- [X] Commit + log

**Friday — Error handling + iterators via `PrimitivesError` + hex parsing**
- [X] Book ch9 + ch13.1 + ch13.2
- [X] Rustlings `error_handling`, `options`, `iterators` — all
- [X] Read `thiserror` and `anyhow` docs end-to-end
- [X] **Build**: `crates/eth-primitives/src/error.rs` — `PrimitivesError` enum with thiserror::Error.
- [X] **Build**: `FromStr` for Address, B256, Bytes — iterator-driven byte-pair decoder.
- [X] Three rewrites of `parse_address`: panic, Result+thiserror, anyhow.
- [X] Watch Crust of Rust: Iterators (full).
- [X] Implement `flatten()` from scratch — applied as `Bytes::concat`.
- [X] Commit + log

**Saturday — Closures + Fn/FnMut/FnOnce + `U256` + R4R**
- [X] Book ch13.1 (Closures) — FnOnce/FnMut/Fn semantics
- [X] **Build**: `crates/eth-primitives/src/uint.rs` — `pub use ruint::aliases::U256;` + extension trait `U256Ext`.
- [X] Closure exercise: `Bytes::map_chunks<F: FnMut(&[u8]) -> Bytes>` — used by RLP encoder Week 5.
- [X] Read Rust for Rustaceans ch1-2.
- [X] `cargo clippy --all -- -D warnings`, `cargo test`, tag `eth-primitives v0.1.0-week1`.
- [X] Commit + log

**Sunday — Rest + Weekly Ritual**
- [X] "Can I explain ownership/borrowing/lifetimes using `Bytes::slice` and `FixedBytes` without looking up?"
- [X] Inheritance check: eth-primitives exports complete.

---

### Week 2 — Smart pointers + sync concurrency via `eth-storage-cache`

**Mirror target**: `revm::db::CacheDB` + reth_provider in-memory layer + `revm_primitives::Database` trait shape.
**Crate created**: `crates/eth-storage-cache/`.
**Inherits from**: `eth-primitives`.
**Feeds into**: `exec-vm` Phase 4; `storage-trie` Phase 3.

**Monday — Box, Deref, Drop via `Page` primitive**
- [X] Book ch15.1-15.4
- [X] **Build**: `crates/eth-storage-cache/src/page.rs` — `Page(Box<[u8; 4096]>)` with Deref, DerefMut, Drop instrumented via tracing::trace!. This 4 KiB page is reused for mmap-backed layout in Phase 3.
- [X] Implement `MyBox<T>` exercise applied as `PageBox<T: ?Sized>` — single-allocation deserialize-in-place. Shape MDBX cursors use.
- [ ] Single-linked list of Pages as a free-list allocator (`PageAllocator`). Attempt doubly-linked free list to feel the pain → motivates Rc/Weak Tuesday.
- [ ] Commit + log

**Tuesday — RefCell, Rc, Arc via `Account` cache**
- [X] Book ch15.5-15.6
- [ ] Watch Crust of Rust: Smart Pointers and Interior Mutability
- [ ] **Build**: `crates/eth-storage-cache/src/account.rs` — `Account { nonce: u64, balance: U256, code_hash: B256, code: Option<Bytes> }` mirroring revm_primitives::Account.
- [ ] **Build**: `LocalAccountCache(HashMap<Address, Rc<RefCell<Account>>>)` first — single-threaded. Add get_or_load, commit. Use RefCell::borrow_mut and observe the runtime panic when you double-borrow.
- [ ] **Migrate**: clone the file to `SharedAccountCache(HashMap<Address, Arc<RwLock<Account>>>)`. Document the diff in `notes/05_smart_pointers.md`.
- [ ] Commit + log

**Wednesday — Threads, channels, Mutex via `StateCache` trait**
- [X] Book ch16 (whole chapter)
- [ ] Watch Crust of Rust: Channels — implement bounded MPSC from scratch
- [ ] **Build**: `crates/eth-storage-cache/src/database.rs` — `StateCache` trait shaped like revm's `Database` (basic, code_by_hash, storage, block_hash).
- [ ] Implement `MutexCache` and `RwLockCache`. Apply bounded-MPSC as write-batch queue.
- [ ] Read `parking_lot::Mutex` vs std — keep parking_lot (reth uses it).
- [ ] Commit + log

**Thursday — Send/Sync via `ShardedCache`**
- [X] Book ch16.4 (Send and Sync)
- [ ] Read `std::marker` docs carefully
- [ ] **Build**: `ShardedCache<const N: usize>` — `[parking_lot::RwLock<HashMap<Address, Account>>; N]` hash-routed by `Address::word()[0] % N`. Implement StateCache.
- [ ] Send/!Sync + !Send/Sync exercises grounded in the cache.
- [ ] Commit + log

**Friday — `EvictionPolicy` + criterion benches**
- [ ] **Build**: `crates/eth-storage-cache/src/eviction.rs` — `EvictionPolicy` trait. LruEviction + BlockTagEviction.
- [ ] Wire eviction into ShardedCache.
- [ ] criterion bench: Mutex vs RwLock vs Sharded(N=16, N=64). Plot and commit.
- [ ] Read parking_lot, dashmap, arc-swap docs — 1-paragraph summary each.
- [ ] Commit + log

**Saturday — Polish + R4R + tag v0.1.0**
- [ ] thiserror StateCacheError, tracing spans, loom tests on tiny subset.
- [ ] Read Rust for Rustaceans ch1-2.
- [ ] README + tag `eth-storage-cache v0.1.0`.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] Inheritance check: StateCache mirrors revm's Database.

---

### Week 3 — Async/Pin/Future via `eth-network-codec`

**Mirror target**: `reth-eth-wire` framing layer + `tokio_util::codec::Framed`.
**Crate created**: `crates/eth-network-codec/`.

**Monday — Tokio fast track + transport scaffold**
- [ ] Read Tokio tutorial cover-to-cover.
- [ ] **Build**: `crates/eth-network-codec/src/transport.rs` — TcpStream wrapper + `LengthDelimitedCodec` with 1 MiB max frame.
- [ ] Manual TCP echo via framed transport.
- [ ] Commit + log

**Tuesday — Manual Future + `MessageRequest`**
- [ ] Async Book ch1-7 in one go.
- [ ] Watch Crust of Rust: Async/Await (full) — implement trivial executor.
- [ ] **Build**: `crates/eth-network-codec/src/request.rs` — `MessageRequest<R>` future.
- [ ] Counter Future applied: `RetryFuture<F: Future>`.
- [ ] Commit + log

**Wednesday — Pin/Unpin via `MessageStream`**
- [ ] Watch Crust of Rust: The Drop Check; read `std::pin` docs.
- [ ] **Build**: `crates/eth-network-codec/src/stream.rs` — `MessageStream<C: Codec, IO>` implementing `tokio_stream::Stream`. Use `pin_project_lite`.
- [ ] Demonstrate why MessageStream cannot be Unpin. Rewrite once with manual unsafe pin projection, then with pin_project_lite. Compare.
- [ ] `notes/06_pin_unpin.md` — worked example.
- [ ] Commit + log

**Thursday — `EthMessage` enum + `Codec` trait**
- [ ] **Build**: `crates/eth-network-codec/src/codec.rs` — `Codec` trait.
- [ ] **Build**: `crates/eth-network-codec/src/message.rs` — `EthMessage` enum subset: Status, BlockHeaders, BlockBodies, NewBlock, GetBlockHeaders.
- [ ] RLP placeholder (tagged-byte format; full RLP comes Week 5).
- [ ] tokio TCP server with graceful shutdown.
- [ ] Commit + log

**Friday — Token bucket as custom `Future` + per-peer rate limiting**
- [ ] **Build**: `crates/eth-network-codec/src/rate_limit.rs` — `TokenBucket` as custom Future.
- [ ] **Build**: `RateLimitedStream<S: Stream>`.
- [ ] Test under load (1k concurrent peers).
- [ ] Commit + log

**Saturday — `BackpressureStrategy` + observability + tag v0.1.0**
- [ ] **Build**: `BackpressureStrategy` enum (DropOldest, DropNewest, Block).
- [ ] Tracing spans for connection lifecycle.
- [ ] Prometheus metrics via `metrics` crate.
- [ ] Load test with 10k concurrent connections.
- [ ] Tag `eth-network-codec v0.1.0`.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] Inheritance check.

---

### Week 4 — Atomics, unsafe, variance, macros via `eth-primitives` v0.2

**Crate extended**: `eth-primitives` v0.1 → v0.2. Add `crates/eth-primitives-derive/`.

**Monday — Layout audit on existing `eth-primitives`**
- [ ] Rustonomicon ch1, ch2, ch3.
- [ ] Run size_of/align_of over every type. Verify FixedBytes<N> is repr(transparent).
- [ ] Add repr(C) to Account in eth-storage-cache.
- [ ] Inspect Bytes layout — Arc<[u8]> 2-word size; `notes/07_variance.md`.
- [ ] Commit + log

**Tuesday — Atomics via `SealedHeader` + `ChainHead` SeqLock**
- [ ] Watch Crust of Rust: Atomics and Memory Ordering (full).
- [ ] **Build**: `crates/eth-primitives/src/atomic_hash.rs` — OnceLock<B256> lazy hash cache. `Sealable` trait.
- [ ] **Build**: `crates/eth-primitives/src/chain_head.rs` — `ChainHead { hash, number }` protected by SeqLock.
- [ ] Re-read Ryuo disruptor code with fresh atomics eyes.
- [ ] Commit + log

**Wednesday — Variance + PhantomData via `Sealed<T>`**
- [ ] Watch Crust of Rust: Subtyping and Variance.
- [ ] **Build**: `crates/eth-primitives/src/sealed.rs` — `Sealed<T> { inner, hash: OnceLock<B256> }`.
- [ ] Make covariant via PhantomData<&'a T> for SealedRef<'a, T>.
- [ ] R4R ch6.
- [ ] Commit + log

**Thursday — Unsafe + miri via `BytesMut::reserve`**
- [ ] Read Nomicon chapters on aliasing, UB.
- [ ] **Build**: `crates/eth-primitives/src/bytes_mut.rs` — `BytesMut`. reserve + extend_from_slice with raw pointer arithmetic. `BytesMut::freeze` to Bytes.
- [ ] Run `cargo +nightly miri test -p eth-primitives`. Chase every UB report.
- [ ] Commit + log

**Friday — Macros via `b256!` + `SimpleEncode` derive**
- [ ] Read R4R ch7 + Little Book of Rust Macros.
- [ ] **Build**: `crates/eth-primitives/src/macros.rs` — `b256!`, `address!` const macros.
- [ ] **Build**: `crates/eth-primitives-derive/` proc-macro crate (syn + quote). `#[derive(SimpleEncode)]` placeholder for Week 5's RlpEncodable.
- [ ] Test the derive on a 3-field struct.
- [ ] Commit + log

**Saturday — R4R + integration polish**
- [ ] R4R ch1-5 — finish.
- [ ] Apply at least one R4R insight to refactor existing crates.
- [ ] Tag `eth-primitives v0.2.0`.
- [ ] Commit + log

**Sunday — Rest + End Month 1 review**
- [ ] Honest assessment: "Could I read reth-trie source today?"
- [ ] Inheritance check: 4 crates shipped.
- [ ] Update North Star M1 metrics.

---

## Month 2: Production Rust + Early Alloy (Weeks 5-8)

### Week 5 — `eth-rlp` crate + Alloy onboarding

**Mirror target**: `alloy-rlp` + `alloy-rlp-derive`.

**Monday — Spec + traits**
- [ ] Re-read W4 Fri's `eth-primitives-derive` scaffold (Cargo.toml proc-macro = true, syn/quote/proc-macro2 deps, basic DeriveInput parsing). 5 min refresh so you don't spend 30 min re-orienting Friday.
- [ ] Read RLP spec. Read alloy-rlp's `Encodable` and `Decodable` source.
- [ ] **Build**: `crates/eth-rlp/src/lib.rs` — `Encodable` and `Decodable` traits matching alloy's signatures.
- [ ] R4R ch7 cross-reference.
- [ ] Commit + log

**Tuesday — `Header` + scalar encoding**
- [ ] **Build**: `crates/eth-rlp/src/header.rs` — Header { list, payload_length }. Test against ethereumjs fixtures.
- [ ] **Build**: `crates/eth-rlp/src/encodable.rs` — impls for u8..u64, U256, bool, slices, Vec, String, Address, B256, Bytes.
- [ ] R4R ch9-11.
- [ ] Commit + log

**Wednesday — List encoding + `Vec<T>` + `length_of_length`**
- [ ] **Build**: Encodable for `Vec<T: Encodable>`, Option<T>, tuples, arrays. length_of_length helper.
- [ ] Nested list test: `Vec<Vec<u64>>` matches Geth's RLP byte-for-byte.
- [ ] Buffer-size-class optimization: pre-size BytesMut.
- [ ] R4R ch12.
- [ ] Commit + log

**Thursday — Alloy code tour (compare not copy)**
- [ ] Clone alloy-rs/alloy. Read alloy-primitives source AND DIFF against your eth-primitives. Note 5 divergences.
- [ ] Read alloy-rlp source — confirm trait signatures match.
- [ ] Commit notes + diff log.
- [ ] Commit + log

**Friday — `RlpEncodable` / `RlpDecodable` derive macros**
- [ ] **Build**: extend `crates/eth-primitives-derive/` with `#[derive(RlpEncodable, RlpDecodable)]`. Mirror alloy-rlp-derive API.
- [ ] Test on 5-field struct — bytes match alloy's derive output.
- [ ] Commit + log

**Saturday — `etherscanlite` CLI**
- [ ] **Build**: `crates/etherscanlite/` — CLI fetching balance/nonce/last-5-tx via alloy-provider, parsed into your types. ~500 LOC.
- [ ] First Alloy issue scan.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] Tag `eth-rlp v0.1.0`.

---

### Week 6 — `eth-consensus` core: Header + transaction envelopes

**Mirror target**: `alloy-consensus`.

**Monday — Yellow Paper §4 + `Header`**
- [ ] ME ch3, ch4. Yellow Paper §4.
- [ ] Run reth on Sepolia.
- [ ] **Build**: `crates/eth-consensus/src/header.rs` — Header mirroring alloy_consensus::Header (all fields incl. requests_hash). `#[derive(RlpEncodable, RlpDecodable)]`.
- [ ] Test: encode mainnet block 1's header → bytes match `cast block 1 --raw`.
- [ ] Commit + log

**Tuesday — Tx types + `Transaction` trait**
- [ ] ME ch5-6. Yellow §6.
- [ ] **Build**: TxLegacy, TxEip1559, TxEip4844.
- [ ] **Build**: `Transaction` trait matching alloy.
- [ ] Sign each tx type via alloy-signer; verify recovery.
- [ ] Commit + log

**Wednesday — EIP-1559 + EIP-4844 fee math**
- [ ] Read EIP-1559 + EIP-4844 specs.
- [ ] **Build**: `crates/eth-consensus/src/eip1559.rs` — calc_next_block_base_fee.
- [ ] **Build**: `crates/eth-consensus/src/eip4844.rs` — calc_excess_blob_gas, calc_blob_fee.
- [ ] Commit + log

**Thursday — Alloy issue hunt + claim**
- [ ] Browse alloy issues. Prefer alloy-consensus, alloy-eips, alloy-rlp.
- [ ] Read CONTRIBUTING.md + 5 recently merged PRs.
- [ ] Pick one, claim.
- [ ] Commit notes.

**Friday — First Alloy PR work**
- [ ] Fork, branch, implement.
- [ ] Commit + log

**Saturday — First Alloy PR submitted + `Signed<T>`**
- [ ] cargo fmt, clippy, nextest. Open PR.
- [ ] **Build**: `crates/eth-consensus/src/signed.rs` — `Signed<T> { tx, signature, hash: OnceLock<B256> }`.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 7 — `eth-consensus`: EIP-7702, EIP-7685, EOF + more PRs

**Crate extended**: `eth-consensus` v0.1 → v0.2.

**Monday — PR #1 review iteration + EIP-2930 access list**
- [ ] Address Alloy PR #1 review.
- [ ] **Build**: `crates/eth-consensus/src/eip2930.rs` — AccessList. Wire into TxEip1559 + TxEip4844.
- [ ] Commit + log

**Tuesday — EIP-7702 `Authorization` + `TxEip7702`**
- [ ] Read EIP-7702 + EIP-7685 specs end-to-end.
- [ ] **Build**: `crates/eth-consensus/src/eip7702.rs` — Authorization, SignedAuthorization, recover_authority.
- [ ] **Build**: TxEip7702 with authorization_list.
- [ ] Commit + log

**Wednesday — EIP-7685 + EOF skeleton**
- [ ] Read EOF EIPs: 3540, 3670, 4200, 4750.
- [ ] **Build**: `crates/eth-consensus/src/eip7685.rs` — Request enum + requests_root.
- [ ] **Build**: `crates/eth-consensus/src/bytecode.rs` — Bytecode enum. EOF parser skeleton.
- [ ] Commit + log

**Thursday — `TxEnvelope` + Second Alloy PR**
- [ ] **Build**: `crates/eth-consensus/src/envelope.rs` — TxEnvelope enum dispatching across all tx types.
- [ ] Pick + implement second Alloy PR.
- [ ] Commit + log

**Friday — Third Alloy PR (medium)**
- [ ] Substantive PR — prefer alloy-consensus or alloy-eips.
- [ ] Commit + log

**Saturday — PR #3 submitted + Foundry intro**
- [ ] Submit PR #3.
- [ ] Clone foundry-rs/foundry, browse forge + cast.
- [ ] Commit notes.

**Sunday — Rest + Weekly Ritual**

---

### Week 8 — Foundry PR + revm familiarization + `eth-consensus` Receipt/Log

**Crate extended**: `eth-consensus` v0.2 → v0.3.

**Monday — Foundry issue hunt + claim**
- [ ] Browse Foundry issues, pick good first. Prefer `cast`.
- [ ] Commit notes.

**Tuesday — First Foundry PR**
- [ ] Implement + submit.
- [ ] Commit + log

**Wednesday — revm overview (read with `exec-vm` Phase 4 in mind)**
- [ ] Clone bluealloy/revm, read README + arch doc.
- [ ] Cross-reference revm-primitives::Database against your eth-storage-cache::StateCache. Adjust StateCache if needed.
- [ ] Commit notes.

**Thursday — revm-interpreter + `Receipt` build**
- [ ] Read revm-primitives, compare with eth-primitives.
- [ ] Read revm-interpreter — opcode dispatch, gas. Trace ADD end-to-end.
- [ ] **Build**: `crates/eth-consensus/src/receipt.rs` — Receipt + ReceiptEnvelope. RLP derive.
- [ ] Commit + log

**Friday — ME ch13 + `Log` + `Bloom`**
- [ ] ME ch13 full. Walk evm.codes top 20 opcodes.
- [ ] **Build**: `crates/eth-consensus/src/log.rs` — Log + bloom_filter(logs).
- [ ] Commit notes.

**Saturday — PR cleanup + tag `eth-consensus v0.3.0`**
- [ ] Address all reviewer feedback.
- [ ] Tag `eth-consensus v0.3.0`.
- [ ] Commit + log

**Sunday — Rest + End Month 2 review**
- [ ] Target check: 3+ Alloy PRs, 1+ Foundry PR.

---

## Month 3: `exec-vm` + `eth-trie` seeds (Weeks 9-12)

### Week 9 — `exec-vm` Phase-1 seed

**Mirror target**: revm-interpreter subset.
**Crate created**: `crates/exec-vm/`.

**Monday — `Stack` + arithmetic opcodes**
- [ ] **Build**: `crates/exec-vm/src/interpreter/stack.rs` — 1024-deep Stack mirroring revm_interpreter::Stack.
- [ ] **Build**: `crates/exec-vm/src/instructions/arithmetic.rs` — ADD, SUB, MUL, DIV, MOD. 3 gas/op.
- [ ] **Build**: `crates/exec-vm/src/interpreter/mod.rs` — Interpreter skeleton + step() dispatcher.
- [ ] Commit + log

**Tuesday — `SharedMemory` + control flow**
- [ ] **Build**: `crates/exec-vm/src/interpreter/memory.rs` — SharedMemory. mload/mstore/mstore8/resize with quadratic gas.
- [ ] **Build**: `crates/exec-vm/src/instructions/control.rs` — JUMP, JUMPI, JUMPDEST, PC, STOP, INVALID.
- [ ] **Build**: `crates/exec-vm/src/instructions/comparison.rs` — LT, GT, SLT, SGT, EQ, ISZERO, AND, OR, XOR, NOT.
- [ ] Commit + log

**Wednesday — `Gas` + SSTORE/SLOAD against `StateCache`**
- [ ] **Build**: `crates/exec-vm/src/interpreter/gas.rs` — Gas { limit, remaining, refunded }.
- [ ] **Build**: `crates/exec-vm/src/instructions/host.rs` — SSTORE, SLOAD, BALANCE, EXTCODESIZE. `Host` trait delegating to eth_storage_cache::StateCache.
- [ ] 15-20 opcodes total. Test against hand-rolled bytecode.
- [ ] cargo test -p exec-vm. Tag `exec-vm v0.0.1`.
- [ ] Commit + log

**Thursday — First revm PR**
- [ ] Browse revm issues, pick good first, implement, submit.
- [ ] Commit + log

**Friday — `eth-rlp` extension: typed envelopes**
- [ ] **Extend**: `crates/eth-consensus/src/envelope.rs` — implement RLP for TxEnvelope with leading type byte per EIP-2718. Test against mainnet typed-tx test vectors.
- [ ] **Extend**: same for ReceiptEnvelope.
- [ ] Diff against alloy-eips::eip2718.
- [ ] Commit + log

**Saturday — More PRs (Alloy/revm)**
- [ ] Whichever is unblocked. Prefer revm now that exec-vm is bootstrapped.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 10 — `eth-trie` Phase-1 seed

**Mirror target**: `alloy-trie` subset.
**Crate created**: `crates/eth-trie/`.

**Monday — MPT theory + `Nibbles`**
- [ ] ethereum.org MPT docs + 2-3 blog explanations.
- [ ] Draw extension/branch/leaf/hash node diagrams.
- [ ] **Build**: `crates/eth-trie/src/nibbles.rs` — `Nibbles(SmallVec<[u8; 64]>)`. Hex-prefix encoding.
- [ ] Commit + log

**Tuesday — `Node` enum + insert/get**
- [ ] **Build**: `crates/eth-trie/src/node.rs` — Node enum (Empty, Leaf, Extension, Branch).
- [ ] **Build**: `crates/eth-trie/src/storage.rs` — `TrieStorage` trait. Initial impl `MemoryStorage`.
- [ ] Insert + get on trie. Test on `[("do","verb"),("dog","puppy"),("doge","coin")]`.
- [ ] Commit + log

**Wednesday — `HashBuilder` + root hash**
- [ ] **Build**: `crates/eth-trie/src/hash_builder.rs` — HashBuilder. Stream-builds root via keccak256 of RLP-encoded nodes.
- [ ] Test against EIP-1186 vectors + alloy-trie fixtures.
- [ ] Tag `eth-trie v0.0.1`.
- [ ] Commit + log

**Thursday — Second revm PR**
- [ ] Pick, implement, submit.
- [ ] Commit + log

**Friday — Reth passive exposure**
- [ ] Clone paradigmxyz/reth. cargo build --release.
- [ ] Browse `reth/crates/trie`. Identify HashBuilder, TrieWalker, HashedPostState, TrieUpdates.
- [ ] Read 5 recently merged trie/storage PRs for style.
- [ ] Commit notes.

**Saturday — `peer-keepalive` state machine on `eth-network-codec`**
- [ ] Build peer-keepalive ping/pong oscillator inside eth-network-codec.
- [ ] Property tests with proptest.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 11 — Type-state + `HashedPostState` + reth survey

**Monday — Type-state pattern applied to `eth-network-codec`**
- [ ] Type-state, sealed trait, extension trait reading.
- [ ] **Refactor**: `crates/eth-network-codec/src/connection.rs` — `Connection<S>` with phantom states Disconnected/Handshaking/Established.
- [ ] Commit + log

**Tuesday — Erigon staged sync (read with your crates as the substrate)**
- [ ] Read Erigon staged sync design doc.
- [ ] Map: headers → bodies → senders → execution → hashing → merkle. For each stage, name the eth-* crate that feeds it.
- [ ] Commit notes.

**Wednesday — `HashedPostState` + `TrieUpdates`**
- [ ] Browse reth/crates/trie source.
- [ ] **Build**: `crates/eth-trie/src/hashed_state.rs` — HashedPostState mirroring reth_trie.
- [ ] **Build**: TrieUpdates struct.
- [ ] Commit + log

**Thursday — Third revm PR (medium difficulty)**
- [ ] Pick substantive issue, implement.
- [ ] Commit + log

**Friday — Twitter + GitHub presence warm-up**
- [ ] First thoughtful technical reply on a reth/paradigm tweet.
- [ ] Star key repos. Follow 20 more Ethereum infra engineers.
- [ ] Commit notes.

**Saturday — Outstanding PR cleanup + tag**
- [ ] Address all reviewer feedback.
- [ ] Tag `eth-trie v0.1.0`. `eth-network-codec v0.2.0`.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 12 — Phase 1 close + Phase 2 prep

**Monday — MDBX overview**
- [ ] Read libmdbx high-level README + libmdbx-rs crate skim.
- [ ] Sketch layering: StateCache → MdbxStateCache (Phase 3) → MDBX env.
- [ ] Commit notes.

**Tuesday — Reth architecture talk + consensus background**
- [ ] Watch gakonst reth architecture talk on YouTube.
- [ ] Mastering Ethereum consensus chapter; The Merge high level.
- [ ] Commit notes.

**Wednesday — Final Alloy/revm PR for Phase 1**
- [ ] Push one more PR over the finish line.
- [ ] Commit + log

**Thursday — Maintainer tracker**
- [ ] Note which maintainers reviewed PRs.
- [ ] Identify mentor candidate (likely Matthias Seitz).
- [ ] Commit notes.

**Friday — Reth Telegram + Discord**
- [ ] Join reth Telegram, observe (don't post yet).
- [ ] Commit notes.

**Saturday — Phase 1 review**
- [ ] Verify shipped crates: eth-primitives v0.2, eth-rlp v0.1, eth-storage-cache v0.1, eth-network-codec v0.2, eth-consensus v0.3, exec-vm v0.0.1, eth-trie v0.1, eth-primitives-derive v0.1.
- [ ] Verify: 3-5 Alloy PRs, 2-3 revm PRs, 1-2 Foundry PRs.
- [ ] cargo test --workspace green; clippy clean; miri clean on eth-primitives.
- [ ] Phase 1 reflection in `progress.md`.
- [ ] Commit + log

**Sunday — End Phase 1 ritual**
- [ ] Full assessment.
- [ ] Update North Star M3 metrics.
- [ ] Phase 2 starts tomorrow.

---

# PHASE 2: ETHEREUM FOUNDATION + ECOSYSTEM PRs (Month 4-6)

> Tempo enters this phase as reading/orientation only. First touch W16 Tue. No Tempo PRs in Phase 2.

## Month 4: Ethereum Protocol + Alloy PRs

### Week 13 — Ethereum fundamentals + `eth-consensus` deepening

**Monday — ME ch3 + `SealedHeader` finalize**
- [ ] ME ch3 (Clients) + ethereum.org intro skim.
- [ ] Run reth on Sepolia, observe sync logs.
- [ ] **Build**: `crates/eth-consensus/src/sealed.rs` — SealedHeader mirroring reth_primitives. hash_ref via keccak256(rlp(header)).
- [ ] Test: hash matches mainnet block hashes via alloy-provider.
- [ ] Commit + log

**Tuesday — ME ch4 + signer recovery**
- [ ] ME ch4 (Cryptography). keccak256, secp256k1.
- [ ] **Build**: `crates/eth-consensus/src/recovery.rs` — recover_signer using k256 directly.
- [ ] **Build**: `Signed<T>::recover_signer()`.
- [ ] Commit + log

**Wednesday — ME ch5-6 + `Block` + `Body`**
- [ ] ME ch5-6.
- [ ] **Build**: `crates/eth-consensus/src/block.rs` — Block, BlockBody, SealedBlock.
- [ ] Sign each tx type via your signature_hash() then alloy-signer; assert recovered address matches.
- [ ] Commit + log

**Thursday — ME ch7 + `encode_tx` round-trip**
- [ ] ME ch7 (Smart Contracts Solidity).
- [ ] Deploy simple contract on Sepolia via Foundry.
- [ ] **Build**: `crates/eth-consensus/src/encode_tx.rs` — encode_signed_tx. Send via eth_sendRawTransaction against Sepolia.
- [ ] Commit + log

**Friday — Yellow Paper §4 + `Account` + `StorageEntry`**
- [ ] Yellow Paper §4.
- [ ] **Build**: `crates/eth-consensus/src/account.rs` — Account (RLP on-disk form). From/To conversions to eth-storage-cache's in-memory Account.
- [ ] Draw state diagrams.
- [ ] Commit + log

**Saturday — Yellow Paper §6 + intrinsic gas calculator**
- [ ] Yellow Paper §6.
- [ ] **Build**: `crates/eth-consensus/src/gas.rs` — intrinsic_gas. Test against revm's validate_initial_tx_gas.
- [ ] Tag `eth-consensus v0.4.0`.
- [ ] Commit notes.

**Sunday — Rest + Weekly Ritual**

---

### Week 14 — EIP deep dives via `eth-eips` extraction + medium Alloy PRs

**Crate created**: `crates/eth-eips/`.

**Monday — EIP-1559 deep + extract `eth-eips/eip1559`**
- [ ] Re-read EIP-1559 + Paradigm analysis.
- [ ] **Refactor**: move to crates/eth-eips/src/eip1559.rs. Add BaseFeeParams for Optimism/Base chain-specific overrides.
- [ ] Test against mainnet, Optimism, Base genesis.
- [ ] Commit + log

**Tuesday — EIP-4844 (blobs) deep + KZG**
- [ ] Read EIP-4844 + Proto-Danksharding roadmap.
- [ ] **Refactor**: move blob fee math to crates/eth-eips/src/eip4844.rs. Add BlobTransactionSidecar. KZG placeholder.
- [ ] Commit notes.

**Wednesday — EIP-7702 deep + `eth-eips/eip7702`**
- [ ] Re-read EIP-7702.
- [ ] **Refactor**: move Authorization + SignedAuthorization from eth-consensus to eth-eips.
- [ ] Tag `eth-eips v0.1.0`.
- [ ] Commit notes.

**Thursday — Alloy issues scan (target `alloy-eips`)**
- [ ] Browse alloy issues. PREFER alloy-eips.
- [ ] Identify 3-5 candidates, pick one, claim.
- [ ] Commit notes.

**Friday — Medium-difficulty Alloy PR work**
- [ ] Substantive change in alloy-eips or alloy-consensus.
- [ ] Commit + log

**Saturday — Alloy PR submitted**
- [ ] Finish. Open PR with motivation referencing eth-eips design notes.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 15 — EIP-7685 + EOF parser in `exec-vm` + more PRs

**Monday — Respond to Alloy PR reviews**
- [ ] Address feedback; iterate.
- [ ] Commit + log

**Tuesday — EIP-7685 finalize in `eth-eips`**
- [ ] Re-read EIP-7685.
- [ ] **Refactor**: move Requests from eth-consensus to crates/eth-eips/src/eip7685.rs.
- [ ] Tag `eth-eips v0.2.0`.
- [ ] Commit notes.

**Wednesday — EOF parser deepening in `exec-vm`**
- [ ] Re-read EIP-3540, 3670, 4200, 4750.
- [ ] **Build**: `crates/exec-vm/src/eof/parser.rs` — full EOF container parser.
- [ ] **Build**: `crates/exec-vm/src/eof/validate.rs` — EIP-3670 code validation.
- [ ] Test against revm's EOF vectors.
- [ ] Commit notes.

**Thursday — Second Alloy PR**
- [ ] Pick next candidate, implement.
- [ ] Commit + log

**Friday — Third Alloy PR work (medium)**
- [ ] Substantive contribution.
- [ ] Commit + log

**Saturday — Third Alloy PR complete**
- [ ] Finish, submit.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 16 — Alloy/Foundry PRs + `eth-rpc-types` extraction

**Crate created**: `crates/eth-rpc-types/`.

**Monday — `eth-rpc-types` + 4th Alloy PR**
- [ ] **Build**: `crates/eth-rpc-types/src/block.rs` — RPC Block, Transaction.
- [ ] Pick + implement 4th Alloy PR.
- [ ] Commit + log

**Tuesday — Foundry codebase intro**
- [ ] Clone foundry-rs/foundry. Read Foundry Book briefly.
- [ ] Browse forge crate source.
- [ ] [Tempo] 15 min at end of day: open `github.com/tempoxyz/tempo-foundry` in browser. Read top of README. Note: fork of Foundry adding TempoEvm extending revm, plus `--tempo.fee-token` support. Close tab. No PR, no commit. Awareness only.
- [ ] Commit notes.

**Wednesday — Foundry cast + `eth-rpc-types/filter`**
- [ ] Read cast crate source.
- [ ] **Build**: `crates/eth-rpc-types/src/filter.rs` — Filter, FilterBlockOption, Topic.
- [ ] Commit notes.

**Thursday — First Foundry PR**
- [ ] Browse Foundry issues, pick good first. Prefer cast.
- [ ] Implement.
- [ ] [Tempo] 30 min at end of day: open `tempoxyz/tempo` README + `docs.tempo.xyz` landing page. Create `notes/tempo_orientation.md` with one paragraph: "What Tempo is, why it's relevant to my Reth bet."
- [ ] Commit + log

**Friday — Foundry PR complete + Alloy review responses**
- [ ] Finish Foundry PR. Address Alloy review feedback.
- [ ] Commit + log

**Saturday — `eth-rpc-types/transaction_request` + 5th Alloy PR**
- [ ] **Build**: `crates/eth-rpc-types/src/transaction_request.rs` — TransactionRequest.
- [ ] Tag `eth-rpc-types v0.1.0`.
- [ ] Submit 5th Alloy PR or polish existing.
- [ ] Commit + log

**Sunday — Rest + End Month 4 review**
- [ ] Target check: 5+ Alloy PRs opened, some merged.

---

## Month 5: EVM Deep Dive + revm PRs

### Week 17 — `exec-vm` expansion (DOUBLES opcode coverage)

**Crate extended**: exec-vm v0.0.1 → v0.1.0.

**Monday — ME ch13 part 1 + `Env` types**
- [ ] ME ch13 first half. Memorize top 20 opcodes.
- [ ] **Build**: `crates/exec-vm/src/env.rs` — Env, BlockEnv, TxEnv, CfgEnv mirroring revm_primitives::Env.
- [ ] **Build**: `From<&TxEnvelope> for TxEnv`, `From<&Header> for BlockEnv`.
- [ ] Commit notes.

**Tuesday — ME ch13 part 2 + `instructions/system.rs`**
- [ ] ME ch13 second half.
- [ ] **Build**: `crates/exec-vm/src/instructions/system.rs` — RETURN, REVERT, INVALID, SELFDESTRUCT (skeleton — full impl Phase 4 needs journal).
- [ ] Commit notes.

**Wednesday — evm.codes deep + `instructions/stack.rs`**
- [ ] Walk every opcode on evm.codes.
- [ ] **Build**: `crates/exec-vm/src/instructions/stack.rs` — PUSH0..PUSH32, DUP1..DUP16, SWAP1..SWAP16, POP. All 96.
- [ ] Manual trace simple bytecode through interpreter.
- [ ] [Tempo] 30 min at end of day: read `tempoxyz/tempo` repo top-level Cargo.toml. Note which reth-* and revm-* crates it pins. Confirm: every Reth crate you've been mirroring is also depended on by Tempo. Add 3-line note to tempo_orientation.md.
- [ ] Commit + log

**Thursday — `instructions/contract.rs` (CALL family)**
- [ ] **Build**: `crates/exec-vm/src/instructions/contract.rs` — CALL, CALLCODE, DELEGATECALL, STATICCALL. EIP-150 63/64ths gas.
- [ ] **Build**: `crates/exec-vm/src/instructions/create.rs` — CREATE, CREATE2 with init code analysis.
- [ ] Test: simple call-with-return via two hand-rolled programs.
- [ ] Commit + log

**Friday — `instructions/host.rs` extension against `StateCache`**
- [ ] **Build**: extend host.rs with BALANCE, EXTCODESIZE, EXTCODEHASH, EXTCODECOPY, BLOCKHASH, COINBASE, TIMESTAMP, NUMBER, DIFFICULTY/PREVRANDAO, GASLIMIT, CHAINID, SELFBALANCE, BASEFEE, BLOBHASH, BLOBBASEFEE.
- [ ] All routed through Host trait → eth-storage-cache::StateCache.
- [ ] Commit + log

**Saturday — `instructions/log.rs` + ethereum-tests subset green**
- [ ] **Build**: `crates/exec-vm/src/instructions/log.rs` — LOG0..LOG4.
- [ ] Total opcode count: 60+. Pass GeneralStateTests/stArithmetic + stMemoryTest subsets.
- [ ] Tag `exec-vm v0.1.0`. README documents opcode coverage matrix.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 18 — revm deep-read (diffing against your `exec-vm`)

**Monday — revm overview + diff to `exec-vm`**
- [ ] Re-clone bluealloy/revm latest.
- [ ] Read README, arch doc fresh — with 60+ opcodes implemented.
- [ ] **Diff log**: for each revm crate, name 3 design choices that differ. Save to `notes/08_revm_diff.md`.
- [ ] Commit notes.

**Tuesday — revm-primitives + `Database` trait alignment**
- [ ] Read revm-primitives source.
- [ ] Confirm your StateCache trait can be a Database for unmodified revm. Adjust if not.
- [ ] Commit notes.

**Wednesday — revm-interpreter dispatch**
- [ ] Read revm-interpreter source. Study opcode dispatch.
- [ ] Identify revm perf optimizations your exec-vm lacks. Add to `EXEC_VM_PERF_BACKLOG.md`.
- [ ] Commit notes.

**Thursday — revm hot path + ADD trace**
- [ ] Trace ADD end-to-end through revm AND your exec-vm. Compare overhead.
- [ ] Commit + log

**Friday — revm handler + precompile reading**
- [ ] Read revm Handler trait and precompile crate.
- [ ] Sketch where Handler plugs into your exec-vm. Phase 4 W53 adds it.
- [ ] Commit notes.

**Saturday — First revm PR informed by the diff**
- [ ] Browse revm issues. Pick something where your exec-vm gives informed perspective.
- [ ] Implement, submit.
- [ ] [Tempo] 45 min at end of day: while revm is fresh, browse `tempoxyz/tempo`'s evm crate. Note how TempoEvm wraps revm's Evm. Sketch in `notes/tempo_diff.md` the 3 most obvious extension points (precompile registry, tx handler, fee accounting). No code.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 19 — revm PR velocity + `exec-vm` precompile skeleton

**Monday — Second revm PR**
- [ ] Pick and implement.
- [ ] Commit + log

**Tuesday — revm PR review response + `exec-vm` precompile registry**
- [ ] Address reviewer feedback.
- [ ] **Build**: `crates/exec-vm/src/precompile/mod.rs` — Precompile trait, PrecompileRegistry. Implement ECRECOVER first.
- [ ] Commit + log

**Wednesday — Third revm PR (medium)**
- [ ] Pick medium-difficulty issue, implement.
- [ ] Commit + log

**Thursday — geth core/vm comparison**
- [ ] Read geth's core/vm package.
- [ ] Add geth-specific notes to 08_revm_diff.md.
- [ ] Commit notes.

**Friday — evmone comparison**
- [ ] Read evmone README + architecture.
- [ ] Note C++ optimizations. Add to EXEC_VM_PERF_BACKLOG.md.
- [ ] Commit notes.

**Saturday — Continue revm PRs**
- [ ] Work on outstanding or start new.
- [ ] [Tempo] 30 min at end of day: skim Tempo TIP index on docs.tempo.xyz. Read just titles and one-line summaries. List the 5 most execution-relevant TIPs (likely: TIP-20, TIP-1020, TIP-1031, TIP-403, plus one more).
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 20 — `eth-trie` expansion

**Crate extended**: eth-trie v0.1 → v0.2.

**Monday — MPT deeper theory + `BranchNodeCompact`**
- [ ] Re-read ethereum.org MPT docs + 2-3 blog explanations.
- [ ] **Build**: `crates/eth-trie/src/branch_compact.rs` — BranchNodeCompact mirroring reth_trie.
- [ ] Commit notes.

**Tuesday — `TrieStorage` abstraction over `StateCache`**
- [ ] **Refactor**: split TrieStorage into HashedNodeStorage + IntermediateStorage.
- [ ] **Build**: `CachedStorage<C: StateCache>` delegating to eth-storage-cache.
- [ ] Commit + log

**Wednesday — `TrieWalker` cursor**
- [ ] **Build**: `crates/eth-trie/src/walker.rs` — TrieWalker<S: TrieStorage> streaming traversal.
- [ ] Commit + log

**Thursday — `ProofRetainer` + EIP-1186 proofs**
- [ ] **Build**: `crates/eth-trie/src/proof/retainer.rs` — ProofRetainer mirroring alloy_trie.
- [ ] **Build**: `crates/eth-trie/src/proof/verify.rs` — verify_proof.
- [ ] Test against EIP-1186 vectors + captured mainnet eth_getProof response.
- [ ] Commit + log

**Friday — `StateRoot` orchestrator**
- [ ] **Build**: `crates/eth-trie/src/state_root.rs` — StateRoot<S> with compute(). Heart of MerkleStage.
- [ ] Test: reconstruct block 1 mainnet state root from genesis + block 1 changes.
- [ ] Commit + log

**Saturday — `StorageRoot` + tag**
- [ ] **Build**: `crates/eth-trie/src/storage_root.rs`.
- [ ] Pass simplest Ethereum trie test vectors end-to-end.
- [ ] Tag `eth-trie v0.2.0`.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 45 min during ritual: pick ONE TIP (recommended: TIP-1020 since signature verification ties to your W19 ECRECOVER work). Read end-to-end. Note in tempo_orientation.md how it would plug into exec-vm precompile registry.
- [ ] Update North Star M5 metrics.

---

## Month 6: MPT Understanding + First Maintainer Interactions

### Week 21 — `eth-rlp` extension + maintainer engagement

**Monday — `eth-rlp` extension: trie-friendly encoding**
- [ ] Re-read RLP spec sections relevant to trie nodes.
- [ ] **Build**: `crates/eth-rlp/src/trie.rs` — encode_branch_node, encode_extension_node, encode_leaf_node.
- [ ] **Build**: refactor EipTransactionRlp helper to use eth-rlp helpers consistently.
- [ ] Commit + log

**Tuesday — Reth RLP usage patterns + `eth-rlp` derive enhancements**
- [ ] Read reth's RLP usage patterns + alloy-rlp source freshly.
- [ ] **Extend**: eth-rlp-derive to support `#[rlp(trailing)]`.
- [ ] Tag `eth-rlp v0.2.0`.
- [ ] Commit + log

**Wednesday — Fourth revm PR**
- [ ] Pick + implement.
- [ ] Commit + log

**Thursday — Second Foundry PR**
- [ ] Pick + implement.
- [ ] Commit + log

**Friday — Maintainer engagement**
- [ ] Identify maintainers per area (alloy-eips: gakonst/yash; revm: rakita; reth-trie: rakita/mattsse).
- [ ] Engage thoughtfully in an issue discussion.
- [ ] [Tempo] 15 min at end of day: identify which Tempo maintainers overlap with your Reth tracker. Update Tempo maintainer tracker (cross-references for gakonst, rakita, joshieDo). No outreach yet.
- [ ] Commit notes.

**Saturday — Consolidation**
- [ ] Review all open PRs. Close out review comments.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 22 — Staged sync architecture + `eth-stage` trait skeleton

**Crate created**: `crates/eth-stage/`.

**Monday — Erigon staged sync (deeper this time)**
- [ ] Re-read Erigon staged sync doc with implementation eye.
- [ ] Stage concept, unwind, checkpoints.
- [ ] Commit notes.

**Tuesday — Reth stages source dive**
- [ ] Browse reth/crates/stages.
- [ ] **Build**: `crates/eth-stage/src/lib.rs` — Stage trait (id, execute, unwind) matching reth shape.
- [ ] **Build**: Pipeline runner with checkpoint persistence via eth-storage-cache::StateCache.
- [ ] Commit + log

**Wednesday — Stage dependency map**
- [ ] Diagram: headers → bodies → senders → execution → hashing → merkle.
- [ ] **Build**: `crates/eth-stage/src/stages/headers.rs` — skeleton HeaderStage.
- [ ] Commit + log

**Thursday — More revm or Alloy PRs**
- [ ] Keep PR velocity.
- [ ] Commit + log

**Friday — Reth Telegram + Discord**
- [ ] Join reth main Telegram. Observe discussion style for 4 weeks before posting.
- [ ] [Tempo] 15 min: also join Tempo's public community channel (Discord/Telegram per `tempoxyz/tempo` CONTRIBUTING.md). Observe, post nothing for 4 weeks.
- [ ] Commit notes.

**Saturday — `eth-stage` consolidation + tag**
- [ ] Skeleton stages for senders, execution, hashing, merkle.
- [ ] Tag `eth-stage v0.0.1`.
- [ ] [Tempo] 45 min: read TIP-20 (stablecoin token standard) end-to-end. Note differences from ERC-20. Add to tempo_diff.md — fee-token semantics, policy registry hook (TIP-403).
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 23 — Ready up for Phase 3 (`storage-trie` scaffold pre-wiring)

**Monday — Reth storage crate survey + `storage-trie` workspace setup**
- [ ] Browse reth/crates/storage (db, provider, codecs, api).
- [ ] **Build**: `crates/storage-trie/Cargo.toml` workspace member. Empty lib.rs.
- [ ] Confirm cargo build --workspace succeeds.
- [ ] Commit notes + scaffold.

**Tuesday — MDBX first look + `Database` trait sketch**
- [ ] Read libmdbx high-level README.
- [ ] **Sketch**: in storage-trie/src/lib.rs, define Database trait shape.
- [ ] [Tempo] 30 min at end of day: read TIP-1031 (consensus context in block header). Matters for Phase 5 consensus-engine — Tempo's header has extra fields your engine_newPayload handler needs to carry through (gated behind feature flag).
- [ ] Commit notes.

**Wednesday — More Alloy/revm PRs**
- [ ] Keep contribution streak.
- [ ] Commit + log

**Thursday — Conference research**
- [ ] EthCC Paris 2027 + Devcon 2027 dates. Start budgeting.
- [ ] Commit notes.

**Friday — Relationship review**
- [ ] Update maintainer tracker. Identify target mentor.
- [ ] Commit notes.

**Saturday — Month 6 consolidation**
- [ ] Review all PRs. Check target: 5+ Alloy, 3+ revm, 2+ Foundry.
- [ ] [Tempo] 1 hr: read tempoxyz/tempo's storage-adjacent crates. Note divergence from upstream Reth — payment-lane-aware indexing if any. Add to tempo_diff.md.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 24 — Phase 2 close + Phase 3 prep

**Monday — Mastering Ethereum consensus + `consensus-engine` placeholder crate**
- [ ] ME consensus chapter. The Merge high level.
- [ ] **Build**: `crates/consensus-engine/Cargo.toml` workspace member. Empty lib.rs.
- [ ] cargo build --workspace green with all 12 crates.
- [ ] Commit notes.

**Tuesday — Reth architecture talk/video**
- [ ] Watch gakonst reth architecture talk + Paradigm Frontiers talk.
- [ ] Map every component to one of YOUR workspace crates.
- [ ] [Tempo] 20 min: `git clone https://github.com/tempoxyz/tempo /tmp/tempo` and `cargo build --release` (runs in background). Confirms toolchain works against their pinned reth revision. If build fails, file failure mode in `notes/tempo_build_blockers.md` for later. Do NOT debug.
- [ ] Commit notes.

**Wednesday — Phase 3 scope + outline**
- [ ] Read Phase 3 section.
- [ ] Outline approach for Month 7.
- [ ] Commit notes.

**Thursday — Phase 3 scaffolding + CI**
- [ ] CI in `.github/workflows/ci.yml` running fmt --check, clippy, nextest, miri (weekly).
- [ ] README at workspace root with dependency graph.
- [ ] Commit + log

**Friday — Final Phase 2 PRs**
- [ ] Wrap outstanding.
- [ ] Commit + log

**Saturday — Phase 2 review**
- [ ] Full assessment.
- [ ] Verify shipped crates: eth-primitives v0.2, eth-rlp v0.2, eth-storage-cache v0.1, eth-network-codec v0.2, eth-consensus v0.4, eth-eips v0.2, eth-rpc-types v0.1, eth-trie v0.2, eth-stage v0.0.1, exec-vm v0.1, eth-primitives-derive v0.1, storage-trie scaffold, consensus-engine scaffold.
- [ ] Update progress.md.
- [ ] [Tempo] End-of-phase Tempo metrics check: orientation depth target 1 (should hit), TIPs read target 1 — at 3 (TIP-1020 W20, TIP-20 W22, TIP-1031 W23). PRs: 0 (correct).
- [ ] Commit + log

**Sunday — End Phase 2 + Phase 3 prep**
- [ ] Full rest.
- [ ] Phase 3 starts tomorrow.

---

# PHASE 3: STORAGE + TRIE DEEP DIVE (Month 7-12)

**Deliverable**: `storage-trie` v1.0 — MDBX-backed persistent state DB.

> Tempo Phase 3 budget: 2-3 hrs/wk. Reading + bookmarking PR candidates + Sunday release skims. First Tempo PR scheduled W60-62 (Phase 4).

storage-trie consumes seed crates from Phase 1-2: provides MDBX-backed Database implementing eth-storage-cache::StateCache, TrieStorage impl replacing MemoryStorage (W10), MerkleStage impl plugging into eth-stage::Stage (W22).

---

## Month 7: MDBX Foundation + First Reth Storage PRs

### Week 25 — MDBX documentation deep

**Monday — MDBX overview**
- [ ] Read libmdbx.dqdkfa.ru full overview. mmap-based design.
- [ ] Commit notes

**Tuesday — MDBX internals: B-tree**
- [ ] B-tree structure section. Compare with B+tree.
- [ ] Commit notes

**Wednesday — MDBX internals: MVCC**
- [ ] MVCC section. Read tx during write tx.
- [ ] Commit notes

**Thursday — MDBX internals: Durability**
- [ ] WAL / sync modes. Crash recovery.
- [ ] Commit notes

**Friday — MDBX cursor semantics**
- [ ] Cursor documentation. Range scan.
- [ ] Commit notes

**Saturday — libmdbx-rs source**
- [ ] Clone and read libmdbx-rs.
- [ ] [Tempo] 1 hr at end of day: re-read tempoxyz/tempo's storage-adjacent crates with MDBX knowledge fresh. Note divergence from upstream Reth — payment-lane-aware indexing if any. Update tempo_diff.md.
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 26 — Research reth storage architecture

**Monday — Reth storage survey**
- [ ] Browse every crate in reth/crates/storage/. Map relationships.
- [ ] Commit notes

**Tuesday — reth-db deep read part 1**
- [ ] Read reth-db/src/lib.rs. Table definitions.
- [ ] Commit notes

**Wednesday — reth-db deep read part 2**
- [ ] Transaction impl. Cursor wrappers.
- [ ] Commit notes

**Thursday — reth-provider read**
- [ ] reth-provider crate. Abstraction over db.
- [ ] Commit notes

**Friday — First reth storage PR hunt**
- [ ] Browse reth issues tagged storage.
- [ ] Find good-first-issue or docs issue. Claim.
- [ ] [Tempo] 15 min after Reth issue claimed: scan `tempoxyz/tempo` issues filtered by storage, db, state-root labels. Bookmark 2-3 candidate "future second-PR" issues. Do NOT claim. Reth comes first.
- [ ] Commit notes

**Saturday — First reth storage PR work**
- [ ] Implement. Submit PR.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 27 — `storage-trie::mdbx`: mmap scaffold

**Monday — Research mmap in Rust**
- [ ] memmap2 crate docs. Rust + mmap safety.
- [ ] Commit notes

**Tuesday — mmap B-tree research (decide: thin wrapper vs from-scratch)**
- [ ] B-tree on mmap techniques.
- [ ] **Decision**: thin wrapper over libmdbx-rs vs from-scratch. Default wrapper unless explicit re-implementation milestone. Record in notes/.
- [ ] Commit notes

**Wednesday — Crate structure (extending the W23 scaffold)**
- [ ] Lay out storage-trie/src/{mdbx, tables, mpt, state_root, merkle_stage, lib.rs}. mpt and state_root re-export from eth-trie.
- [ ] Sketch Tx / Cursor traits matching reth-db-api.
- [ ] Commit + log

**Thursday — Page provider over mmap**
- [ ] Implement MmapPageProvider returning eth_storage_cache::Page views.
- [ ] Free-list allocation.
- [ ] Commit + log

**Friday — mmap wrapper + growth**
- [ ] mmap-backed file wrapper with safe remap on growth.
- [ ] Commit + log

**Saturday — Respond to reth PR review + continue crate**
- [ ] Address reth PR feedback.
- [ ] Continue crate work.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min: skim Tempo releases page (tempoxyz/tempo/releases). Identify which TIPs are landing this week. Add to notes/tempo_roadmap.md. (Weekly ritual from now on.)

---

### Week 28 — `storage-trie` crate: B-tree core

**Monday — B-tree node design**
- [ ] Design leaf vs internal node layout.
- [ ] Commit + log

**Tuesday — B-tree insert**
- [ ] Insert with node splitting. Unit tests.
- [ ] Commit + log

**Wednesday — B-tree get**
- [ ] Lookup by key. Range iteration.
- [ ] Commit + log

**Thursday — B-tree delete**
- [ ] Delete with node merging.
- [ ] Commit + log

**Friday — Second reth storage PR**
- [ ] Pick next issue. Implement.
- [ ] Commit + log

**Saturday — Crate polish**
- [ ] Document public API. Benchmark setup.
- [ ] Commit + log

**Sunday — Rest + End Month 7 review**
- [ ] Update North Star M7 metrics.
- [ ] [Tempo] 20 min during ritual: Tempo releases skim.

---

## Month 8: MVCC + Reth Storage Contribution Velocity

### Week 29 — MVCC in `storage-trie`

**Monday — MVCC design**
- [ ] Design MVCC (version chain vs copy-on-write).
- [ ] Commit notes

**Tuesday — Read transaction**
- [ ] Implement read tx with snapshot.
- [ ] Commit + log

**Wednesday — Write transaction**
- [ ] Implement write tx with copy-on-write.
- [ ] Commit + log

**Thursday — Concurrent read during write**
- [ ] Test read tx during write tx. Verify snapshot isolation.
- [ ] Commit + log

**Friday — Third reth storage PR**
- [ ] Medium-difficulty issue.
- [ ] Commit + log

**Saturday — Crate: durability**
- [ ] fsync strategies. Crash recovery.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min: Tempo releases skim.

---

### Week 30 — Reth trie crate reading

**Monday — reth-trie overview**
- [ ] Browse reth/crates/trie. Top-level lib.rs.
- [ ] Commit notes

**Tuesday — reth-trie node types**
- [ ] Node definitions: extension, branch, leaf.
- [ ] Commit notes

**Wednesday — reth-trie state root**
- [ ] State root computation. Incremental.
- [ ] Commit notes

**Thursday — reth-trie hashed state**
- [ ] Hashed state abstraction.
- [ ] Commit notes

**Friday — First reth trie PR**
- [ ] Find trie-related issue. Implement.
- [ ] Commit + log

**Saturday — Crate: benchmarks**
- [ ] criterion benchmarks for B-tree ops. Baseline vs sled, redb.
- [ ] [Tempo] 1 hr at end of day: while reth-trie is fresh, browse Tempo's trie integration. Note: Tempo uses Reth's trie wholesale; divergence is in state schema (TIP-20 token balances first-class). 2-paragraph note to tempo_diff.md.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 31 — Persistent MPT in `storage-trie::mpt`

**Inheritance**: Nibbles, Node, HashBuilder, TrieStorage, StateRoot, ProofRetainer all in eth-trie. This week adds **persistent** backing — MdbxTrieStorage implementing eth-trie::TrieStorage against W27-29 MDBX. Do NOT reimplement.

**Monday — `MdbxTrieStorage` design**
- [ ] Design table layout (state nodes by hash, intermediate nodes by Nibbles path).
- [ ] Implement `eth_trie::TrieStorage for MdbxTrieStorage` skeleton.
- [ ] Commit + log

**Tuesday — Wire `eth-trie::Node` to the table layout**
- [ ] Cursor-based read path + dirty-set write path against MDBX.
- [ ] Commit + log

**Wednesday — Persistent insert via existing HashBuilder**
- [ ] Drive eth_trie::HashBuilder with MdbxTrieStorage as read source.
- [ ] Test: round-trip small trie through MDBX; assert root matches W10/W20 in-memory.
- [ ] Commit + log

**Thursday — Persistent get via existing walker**
- [ ] Drive eth_trie::TrieWalker against MdbxTrieStorage.
- [ ] Range scans via MDBX cursor.
- [ ] Commit + log

**Friday — Root hash regression suite against `eth-trie` v0.2 fixtures**
- [ ] Re-run W20's Ethereum test vectors with persistent backing — assert byte-identical roots.
- [ ] Commit + log

**Saturday — Reth trie second PR**
- [ ] Continue trie contribution.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 30 min during ritual: read one Tempo TIP from queue. Storage-relevant if available. Aim for "one TIP per ritual" through Phase 3.

---

### Week 32 — MPT proofs + more reth PRs

**Monday — MPT proof generation**
- [ ] Implement Merkle proof generation. Unit tests.
- [ ] Commit + log

**Tuesday — MPT proof verification**
- [ ] Standalone proof verification.
- [ ] Commit + log

**Wednesday — MPT delete**
- [ ] MPT delete with rebalancing.
- [ ] Commit + log

**Thursday — Ethereum test vectors**
- [ ] Integrate official trie test vectors.
- [ ] Commit + log

**Friday — Reth PR volume**
- [ ] Another reth PR (storage or trie).
- [ ] Commit + log

**Saturday — Crate docs**
- [ ] Comprehensive docs for all public APIs. Examples in docs.
- [ ] Commit + log

**Sunday — Rest + End Month 8 review**
- [ ] [Tempo] 20 min: Tempo releases skim.

---

## Month 9: Trie Depth + Staged Sync Understanding

### Week 33 — Advanced trie: path compression

**Monday — Path compression theory**
- [ ] Research path compression. Ethereum's approach.
- [ ] Commit notes

**Tuesday — Implement path compression**
- [ ] Add to crate MPT. Verify correctness.
- [ ] Commit + log

**Wednesday — Benchmark path compression**
- [ ] Benchmark with/without. Document.
- [ ] Commit + log

**Thursday — Reth staged sync survey**
- [ ] Browse reth/crates/stages deeply.
- [ ] Commit notes

**Friday — Stage dependencies diagram**
- [ ] Detailed flow diagram. Unwind paths.
- [ ] Commit notes

**Saturday — Reth PR day**
- [ ] Another reth PR.
- [ ] [Tempo] 30 min at end of day: read 2 of the most storage-relevant TIPs end-to-end from W19 list. Add 1-page "TIP storage impact" summary to notes/.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 34 — Pruning strategies

**Monday — Pruning research**
- [ ] Ethereum pruning modes: full, archive, pruned.
- [ ] Commit notes

**Tuesday — Reth pruning code**
- [ ] Read reth pruner crate.
- [ ] Commit notes

**Wednesday — Crate: pruning design**
- [ ] Design pruning strategy trait. Plan MPT integration.
- [ ] Commit + log

**Thursday — Implement full pruning**
- [ ] "full" retention (prune history beyond N blocks).
- [ ] Commit + log

**Friday — Implement archive mode**
- [ ] Keep everything mode.
- [ ] Commit + log

**Saturday — Reth PR + integration testing**
- [ ] Reth PR in pruning area if possible.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min: Tempo releases skim.

---

### Week 35 — State commitment deep

**Monday — State commitment theory**
- [ ] State commitment schemes. MPT vs Verkle tradeoffs.
- [ ] Commit notes

**Tuesday — Verkle Trees reading**
- [ ] Verkle Trees research (Vitalik, EF).
- [ ] Commit notes

**Wednesday — Crate: incremental root**
- [ ] Design incremental state root computation.
- [ ] Commit + log

**Thursday — Benchmark incremental vs full**
- [ ] Benchmark root computation.
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Continue velocity.
- [ ] Commit + log

**Saturday — Crate polish**
- [ ] Clean up APIs. Update docs.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min: Tempo releases skim.

---

### Week 36 — Snapshot sync research

**Monday — Snapshot sync theory**
- [ ] Ethereum snapshot sync.
- [ ] Commit notes

**Tuesday — Erigon snapshots**
- [ ] Erigon's snapshot strategy. File format.
- [ ] Commit notes

**Wednesday — Reth snapshots**
- [ ] reth's snapshot approach.
- [ ] Commit notes

**Thursday — Crate: snapshot export**
- [ ] Design export format. Basic export.
- [ ] Commit + log

**Friday — Crate: snapshot import**
- [ ] Snapshot import.
- [ ] Commit + log

**Saturday — End Month 9 PR push**
- [ ] 1-2 more reth PRs.
- [ ] [Tempo] 45 min at end of day: read tempoxyz/tidx README (Tempo's PostgreSQL + ClickHouse chain indexer). Note: analytics path separate from node snapshots. Document architectural split in tempo_diff.md. NOT something to build — interview context.
- [ ] Commit + log

**Sunday — Rest + End Month 9 review**
- [ ] Check: 15+ reth PRs, 10+ in storage/trie.

---

## Month 10: Cross-Subsystem Storage PRs + Integration

### Week 37 — Medium-sized reth PRs

**Monday — Identify meaningful PR target**
- [ ] Enhancement issues. 1 medium PR candidate. Design.
- [ ] Commit notes

**Tuesday — Medium PR: implement**
- [ ] Start implementation.
- [ ] Commit + log

**Wednesday — Medium PR: tests**
- [ ] Comprehensive testing.
- [ ] Commit + log

**Thursday — Medium PR: benchmark**
- [ ] Perf measurements if relevant.
- [ ] Commit + log

**Friday — Medium PR: submit**
- [ ] Submit.
- [ ] Commit + log

**Saturday — Crate work**
- [ ] Continue storage-trie enhancements.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min: Tempo releases skim.

---

### Week 38 — Codec / compression deep

**Monday — Reth codecs**
- [ ] Read reth codecs crate.
- [ ] Commit notes

**Tuesday — Zstd compression in reth**
- [ ] How reth uses compression.
- [ ] Commit notes

**Wednesday — Crate: codec support**
- [ ] Add compact encoding to crate.
- [ ] Commit + log

**Thursday — Crate: compression**
- [ ] Optional compression layer.
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Codec-related PR ideally.
- [ ] Commit + log

**Saturday — Crate benchmarks**
- [ ] Bench compression tradeoffs.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min: Tempo releases skim.

---

### Week 39 — Reth storage architecture contributions

**Monday — Read storage discussions**
- [ ] All recent GitHub discussions on storage.
- [ ] Commit notes

**Tuesday — Substantive comment**
- [ ] Find appropriate discussion. Substantive technical comment.
- [ ] Commit notes

**Wednesday — More reth PR**
- [ ] Continue velocity.
- [ ] Commit + log

**Thursday — Crate: composition test**
- [ ] Integration test: B-tree + MPT + transaction combined.
- [ ] Commit + log

**Friday — Crate: example**
- [ ] Example showing typical usage.
- [ ] Commit + log

**Saturday — Consolidation**
- [ ] Review everything.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min: Tempo releases skim + 1-sentence summary of "what shipped in Tempo this month" to tempo_roadmap.md.

---

### Week 40 — Medium reth feature development

**Monday — Feature proposal**
- [ ] Storage improvement opportunity. Draft proposal comment on GitHub.
- [ ] Commit notes

**Tuesday — Discuss with maintainers**
- [ ] If feedback, iterate. If approved, start design.
- [ ] Commit notes

**Wednesday — Feature implementation part 1**
- [ ] Start coding.
- [ ] Commit + log

**Thursday — Feature implementation part 2**
- [ ] Continue.
- [ ] Commit + log

**Friday — Feature: tests**
- [ ] Comprehensive tests.
- [ ] Commit + log

**Saturday — Feature: submit**
- [ ] Submit PR.
- [ ] Commit + log

**Sunday — Rest + End Month 10 review**
- [ ] [Tempo] 30 min during ritual: identify by name which 2 Tempo maintainers have been most active in the last month's PR/release activity (likely klkvr, legion2002, or 0xrusowsky). Update Tempo maintainer tracker.

---

## Month 11: Feature Shipping + Crate v1.0

### Week 41 — Ship reth feature

**Monday — Address feature PR reviews**
- [ ] Iterate on reviews.
- [ ] Commit + log

**Tuesday — More iteration**
- [ ] Address remaining feedback.
- [ ] Commit + log

**Wednesday — Feature merged ideally**
- [ ] If merged, celebrate + blog draft.
- [ ] Commit + log

**Thursday — Crate: performance pass**
- [ ] Profile storage-trie. Hot paths.
- [ ] Commit + log

**Friday — Crate: optimizations**
- [ ] Implement optimizations.
- [ ] Commit + log

**Saturday — Another reth PR**
- [ ] Keep velocity.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 42 — Geth comparison study

**Monday — Geth core/state package**
- [ ] Read Geth's Go implementation.
- [ ] Commit notes

**Tuesday — Geth core/trie**
- [ ] Read Geth trie implementation.
- [ ] Commit notes

**Wednesday — Write comparison doc**
- [ ] Internal doc: reth vs Geth storage decisions.
- [ ] Commit + log

**Thursday — Reth PR**
- [ ] Continue.
- [ ] Commit + log

**Friday — Crate: fuzz targets**
- [ ] cargo-fuzz targets.
- [ ] Commit + log

**Saturday — Crate: property tests**
- [ ] proptest for MPT invariants.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 43 — Crate v1.0 preparation

**Monday — API review**
- [ ] Review all public APIs. Stabilize.
- [ ] Commit + log

**Tuesday — Documentation pass**
- [ ] Every public item has docs.
- [ ] Commit + log

**Wednesday — Examples expansion**
- [ ] Multiple examples in examples/.
- [ ] Commit + log

**Thursday — CI hardening**
- [ ] All CI checks pass. Coverage. MSRV.
- [ ] Commit + log

**Friday — README + design doc**
- [ ] Comprehensive README. DESIGN.md.
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] More PR activity.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min: Tempo releases skim.

---

### Week 44 — Crate v1.0 ship

**Monday — Final benchmarks**
- [ ] Comprehensive bench suite. Compare vs reth, sled, redb.
- [ ] Commit + log

**Tuesday — Security review self-audit**
- [ ] Review unsafe blocks. Error handling.
- [ ] Commit + log

**Wednesday — Crate v1.0 tag**
- [ ] Tag release.
- [ ] Commit + log

**Thursday — Blog: crate intro**
- [ ] If writing mood, draft "Building storage-trie" post. No deadline.
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Continue.
- [ ] Commit + log

**Saturday — Month 11 review**
- [ ] Assess crate quality. PR portfolio.
- [ ] Commit + log

**Sunday — Rest + End Month 11 review**

---

## Month 12: Phase 3 Close + Phase 4 Prep

### Week 45 — Final reth storage feature

**Monday — Identify second feature**
- [ ] Another meaningful opportunity. Design.
- [ ] Commit notes

**Tuesday — Implement**
- [ ] Code.
- [ ] Commit + log

**Wednesday — Continue**
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Submit**
- [ ] Submit PR.
- [ ] Commit + log

**Saturday — Iterate on reviews**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min: Tempo releases skim.

---

### Week 46 — Recognition signals

**Monday — Review PRs of others**
- [ ] Review others' storage PRs substantively.
- [ ] Commit notes

**Tuesday — Help newcomers**
- [ ] Answer questions in Telegram.
- [ ] Commit notes

**Wednesday — More PR reviews**
- [ ] Build reviewing muscle.
- [ ] Commit notes

**Thursday — Maintainer relationship check**
- [ ] Which maintainers engaged. Update tracker.
- [ ] Commit notes

**Friday — Active issue engagement**
- [ ] Participate in design discussions.
- [ ] Commit notes

**Saturday — Another small reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 47 — revm preview for Phase 4

**Monday — revm architecture refresher**
- [ ] Re-read revm with Phase 3 eyes.
- [ ] Commit notes

**Tuesday — Identify revm learning gaps**
- [ ] Map what needs deep understanding in Phase 4.
- [ ] Commit notes

**Wednesday — Reth evm crate**
- [ ] Read reth/crates/evm.
- [ ] Commit notes

**Thursday — More reth PR**
- [ ] Commit + log

**Friday — Crate maintenance**
- [ ] Any bug fixes on storage-trie.
- [ ] Commit + log

**Saturday — Phase 4 prep (exec-vm already scaffolded — review state)**
- [ ] exec-vm seeded W9 + extended W17. Re-read README + opcode coverage matrix. Gap to Phase 4 v1.0. Phase 4 outline in notes/.
- [ ] [Tempo] 1 hr at end of day: re-read TIP-1020 (signature verification precompile) with exec-vm precompile registry in mind. Sketch in `notes/tempo_evm_ext_design.md` the 3-4 traits and types tempo-evm-ext will need so it can be a downstream crate of exec-vm without forking.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 48 — Phase 3 close

**Monday — Phase 3 reflection**
- [ ] Full assessment vs exit criteria.
- [ ] Commit notes

**Tuesday — Metrics update**
- [ ] Update all North Star metrics. Target: 30 storage PRs, 1+ feature.
- [ ] [Tempo] Update M12 Tempo metrics: orientation depth (target 3 — should be there), TIPs read (target 5 — count carefully), PRs merged (target 3 — flag if zero; first Tempo PR scheduled W60-62 so 0 here is acceptable). Do NOT panic-claim if zero.
- [ ] Commit notes

**Wednesday — Blog if in mood**
- [ ] Phase 3 retrospective in disruptor style. No pressure.
- [ ] Commit + log

**Thursday — Relationship stock-take**
- [ ] Update maintainer tracker. Identify mentor candidate.
- [ ] Commit notes

**Friday — Final Phase 3 PRs**
- [ ] Wrap outstanding.
- [ ] Commit + log

**Saturday — Clean transition prep**
- [ ] Mental prep for Phase 4. Storage maintenance minimum during Phase 4.
- [ ] Commit notes

**Sunday — End Phase 3 rest**
- [ ] Full rest. Phase 4 starts tomorrow.

---

# PHASE 4: EXECUTION DEEP DIVE (Month 13-18)

**Deliverable**: `exec-vm` v1.0 — full revm-equivalent EVM.

> Tempo Phase 4 budget: 4-5 hrs/wk. First Tempo PR W60 Sat. `tempo-evm-ext` scaffolded W54 Sat. `tempo-tx-envelope` v0.1.0 shipped W66 Fri.

## Month 13: Revm Full Codebase + First revm Perf PRs

### Week 49 — Revm architecture deep

**Monday — Revm top-level**
- [ ] Re-read revm from top. Map all crates.
- [ ] Commit notes

**Tuesday — Revm interpreter core**
- [ ] Read revm-interpreter in full. Main execution loop.
- [ ] Commit notes

**Wednesday — Revm Host trait**
- [ ] Read Host trait and impls.
- [ ] Commit notes

**Thursday — Revm Database trait**
- [ ] Read Database trait. How it integrates with any storage.
- [ ] Commit notes

**Friday — Revm precompiles**
- [ ] Read revm-precompiles crate. Each precompile.
- [ ] [Tempo] 1 hr: read revm-precompiles AND tempoxyz/tempo precompile extensions side-by-side. Tempo adds TIP-1020 (P256/WebAuthn/secp256k1 verify) as stateful precompile reusing tx-signature verification. Map: where in exec-vm does this plug in? (Same dispatch point as W19's ECRECOVER, but signature scheme dispatcher must be generic.) Update tempo_evm_ext_design.md.
- [ ] Commit notes

**Saturday — First revm perf-oriented PR**
- [ ] Find performance issue. Implement.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 50 — Revm journaling

**Monday — Journaling design**
- [ ] Read revm-interpreter journal module. Revert semantics.
- [ ] Commit notes

**Tuesday — Nested checkpoints**
- [ ] Study nested call handling.
- [ ] Commit notes

**Wednesday — State access patterns**
- [ ] Read state management in revm.
- [ ] Commit notes

**Thursday — Second revm PR**
- [ ] Another contribution.
- [ ] Commit + log

**Friday — `exec-vm`: align traits with revm `Database`/`Host`**
- [ ] Refactor signatures so any `impl Database for T` from revm Just Works as Host for exec-vm. Goal: swap revm in/out with one type alias change.
- [ ] [Tempo] 30 min: while refactoring, verify trait shapes are compatible with TempoEvm's extension pattern. Skim tempoxyz/tempo's evm crate to confirm tempo-evm-ext can be downstream consumer without trait-incompatible changes.
- [ ] Commit + log

**Saturday — Interpreter loop refactor**
- [ ] Consolidate match-based dispatch from W9 + W17 into `interpreter/dispatch.rs`. Set up W58 jump-table swap.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 51 — Opcode coverage gap-fill

**Inheritance**: most basic opcodes DONE (W9 + W17). This week fills missing arithmetic/bitwise and Cancun-specific opcodes.

**Monday — Missing arithmetic: SDIV, SMOD, ADDMOD, MULMOD, EXP, SIGNEXTEND**
- [ ] Implement. Unit tests against revm for edge cases.
- [ ] Commit + log

**Tuesday — Missing bitwise: BYTE, SHL, SHR, SAR**
- [ ] Implement against revm fixtures.
- [ ] Commit + log

**Wednesday — KECCAK256 + missing call-frame envs**
- [ ] KECCAK256. CALLDATALOAD, CALLDATASIZE, CALLDATACOPY, CODESIZE, CODECOPY, RETURNDATASIZE, RETURNDATACOPY, GASPRICE, ORIGIN, CALLER, CALLVALUE.
- [ ] Commit + log

**Thursday — PREVRANDAO + DIFFICULTY post-Merge handling**
- [ ] Same opcode byte (0x44), different semantics. Fork-aware via CfgEnv::spec_id.
- [ ] PC, MSIZE, GAS, JUMPDEST coverage check.
- [ ] Commit + log

**Friday — TLOAD/TSTORE (EIP-1153, Cancun)**
- [ ] Transient storage scoped to call frame. Adds `transient: HashMap` to call-frame state.
- [ ] Commit + log

**Saturday — MCOPY (EIP-5656, Cancun) + opcode-coverage matrix audit**
- [ ] MCOPY copies memory regions.
- [ ] Diff opcode coverage table against revm's instruction table.
- [ ] [Tempo] 30 min: cross-check opcode coverage against Tempo's EVM. They use upstream revm opcodes plus stateful precompiles — no opcode divergence. Note in tempo_evm_ext_design.md: "tempo-evm-ext adds precompiles + tx handler, not opcodes."
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 52 — `exec-vm` control flow + gas

**Monday — Gas accounting**
- [ ] Implement gas tracking. London schedule.
- [ ] Commit + log

**Tuesday — Memory expansion gas**
- [ ] Quadratic memory gas cost.
- [ ] Commit + log

**Wednesday — Gas schedule per fork**
- [ ] Shanghai, Cancun schedules.
- [ ] Commit + log

**Thursday — LOG opcodes**
- [ ] LOG0-LOG4.
- [ ] Commit + log

**Friday — CREATE/CALL family**
- [ ] CREATE, CREATE2. CALL, CALLCODE, DELEGATECALL, STATICCALL.
- [ ] Commit + log

**Saturday — Another revm PR + reth storage PR**
- [ ] Maintain storage PR velocity. revm contribution.
- [ ] Commit + log

**Sunday — Rest + End Month 13 review**
- [ ] [Tempo] 20 min: Tempo releases skim. If release touches EVM crate, read diff and note implications for tempo-evm-ext.

---

## Month 14: Full Opcode Coverage + Precompiles

### Week 53 — Complete opcode set

**Monday — RETURN, REVERT, INVALID**
- [ ] Terminal opcodes.
- [ ] Commit + log

**Tuesday — SELFDESTRUCT**
- [ ] Implement.
- [ ] Commit + log

**Wednesday — EIP-1153 transient storage**
- [ ] TLOAD/TSTORE if not done.
- [ ] Commit + log

**Thursday — Test vector integration**
- [ ] Integrate Ethereum execution test vectors.
- [ ] Commit + log

**Friday — revm PR**
- [ ] Contribution.
- [ ] Commit + log

**Saturday — Reth evm PR**
- [ ] Find reth evm crate issue. Implement.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 54 — Precompiles in `exec-vm`

**Monday — ecrecover**
- [ ] Implement ecrecover precompile. Test vectors.
- [ ] Commit + log

**Tuesday — sha256, ripemd160, identity**
- [ ] Implement all three.
- [ ] Commit + log

**Wednesday — modexp**
- [ ] Implement (using num-bigint).
- [ ] Commit + log

**Thursday — BN256 operations**
- [ ] BN256Add, BN256ScalarMul, BN256Pairing.
- [ ] Commit + log

**Friday — blake2f**
- [ ] Implement Blake2 F compression.
- [ ] [Tempo] 1 hr at end of day: design exec-vm's precompile dispatch so a downstream tempo-evm-ext can register P256 + WebAuthn verify precompiles without forking. Registry must accept new precompile addresses via registration call (use `Box<dyn Precompile>` over `HashMap<Address, Box<dyn Precompile>>` — W19 skeleton supports this). Add test that registers a dummy "always-return-zero" precompile at address 0x100 to prove extensibility.
- [ ] Commit + log

**Saturday — KZG precompile**
- [ ] Point evaluation precompile (EIP-4844).
- [ ] [Tempo] 45 min: **`tempo-evm-ext` scaffold** — create `crates/tempo-evm-ext/` workspace member. `Cargo.toml` depends on exec-vm, eth-primitives, eth-consensus. Empty `lib.rs` with single `register_tempo_precompiles(registry: &mut PrecompileRegistry)` stub function. cargo build --workspace green. No real code yet; lands W66+.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 55 — Journaling in `exec-vm`

**Monday — Journal design**
- [ ] Design journal structure. Mirror revm's approach.
- [ ] Commit + log

**Tuesday — Account journal**
- [ ] Track account changes with undo log.
- [ ] Commit + log

**Wednesday — Storage journal**
- [ ] Track storage changes.
- [ ] Commit + log

**Thursday — Nested checkpoints**
- [ ] Support nested call checkpoint/commit.
- [ ] Commit + log

**Friday — Revert semantics tests**
- [ ] Test revert properly undoes all changes.
- [ ] Commit + log

**Saturday — revm PR**
- [ ] Contribution.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 56 — Test vector push

**Monday — Ethereum tests repo**
- [ ] Integrate comprehensive test vectors.
- [ ] Commit + log

**Tuesday — General state tests**
- [ ] Run general state tests. Fix failures.
- [ ] Commit + log

**Wednesday — More failure fixing**
- [ ] Continue.
- [ ] Commit + log

**Thursday — Validator tests**
- [ ] Run validator test suite.
- [ ] Commit + log

**Friday — reth PR**
- [ ] Storage or evm.
- [ ] Commit + log

**Saturday — revm PR**
- [ ] Contribution.
- [ ] Commit + log

**Sunday — Rest + End Month 14 review**
- [ ] Crate passing majority of test vectors.
- [ ] [Tempo] 20 min: Tempo releases skim. Check Tempo PR queue for any open PRs that look like good-first-issue material. Bookmark for W60.

---

## Month 15: Dispatch Strategies + EthCC Prep

### Week 57 — EthCC Paris trip

**Monday-Friday — Conference attendance**
- [ ] Attend EthCC sessions.
- [ ] Target: 1-on-1 with 3 reth core contributors. Arrange via Twitter DM in advance.
- [ ] Side events (hacker houses, dinners).
- [ ] [Tempo] If any Tempo team members or design partners (Stripe, Visa blockchain) at EthCC, request 1-on-1. Same priority as Reth core 1-on-1s.
- [ ] Take notes on talks.

**Saturday — Travel home**
- [ ] Rest.

**Sunday — Post-conference ritual**
- [ ] Update maintainer tracker (Reth + Tempo) with new connections.
- [ ] Follow-up emails/DMs.

---

### Week 58 — Back to work: dispatch strategies

**Monday — Match dispatch (baseline)**
- [ ] Baseline benchmark.
- [ ] Commit + log

**Tuesday — Jump table research**
- [ ] Function pointer jump tables.
- [ ] Commit notes

**Wednesday — Implement jump table dispatch**
- [ ] In exec-vm. Feature-flagged.
- [ ] [Tempo] 15 min check: ensure tempo-evm-ext can plug into both match-dispatch and jump-table-dispatch code paths. No code today — just a comment in tempo_evm_ext_design.md.
- [ ] Commit + log

**Thursday — Computed goto research**
- [ ] Unsafe computed goto via asm. Portability tradeoffs.
- [ ] Commit notes

**Friday — Benchmark match vs jump table**
- [ ] Measure instruction-level differences.
- [ ] Commit + log

**Saturday — Dispatch strategy docs**
- [ ] Document findings.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 59 — evmone comparison

**Monday — evmone overview**
- [ ] Read evmone README deeply.
- [ ] Commit notes

**Tuesday — evmone basic interpreter**
- [ ] Basic mode.
- [ ] Commit notes

**Wednesday — evmone advanced mode**
- [ ] Advanced interpreter with caching.
- [ ] Commit notes

**Thursday — Apply learnings to `exec-vm`**
- [ ] Implement applicable optimizations.
- [ ] Commit + log

**Friday — Benchmark exec-vm vs revm**
- [ ] Comprehensive benchmark. Identify gaps.
- [ ] [Tempo] 20 min: note whether tempo-evm-ext's extra precompile dispatch overhead is measurable. (Likely 0 if feature-flagged and not registered.)
- [ ] Commit + log

**Saturday — revm PR**
- [ ] Another contribution.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 60 — Hot path optimization

**Monday — Profile `exec-vm`**
- [ ] Profile with perf or similar.
- [ ] Commit notes

**Tuesday — Stack optimization + SIMD primer**
- [ ] 60-90 min reading: `std::simd` (portable SIMD) + `std::arch::x86_64` (target-specific intrinsics). Focus on `__m256i` load/store and the autovectorization patterns rustc already does. Skim the `wide` crate as the stable ecosystem option.
- [ ] Inline stack ops. While you're in the stack code, evaluate whether U256 push/pop or copy paths can benefit from 256-bit SIMD loads (likely yes for batched memory ops, marginal for single-value push). Note one candidate hot spot in `EXEC_VM_PERF_BACKLOG.md` for SIMD experimentation in W64.
- [ ] Commit + log

**Wednesday — Memory access**
- [ ] Optimize memory reads/writes. MCOPY (EIP-5656) and CODECOPY are the obvious SIMD candidates from yesterday's primer — try `std::arch::x86_64::_mm256_loadu_si256` / `_mm256_storeu_si256` for aligned 32-byte block copies and bench against the naive loop. Feature-flag the SIMD path so non-x86_64 targets fall back cleanly.
- [ ] Commit + log

**Thursday — Gas calculation**
- [ ] Optimize gas tracking in hot path.
- [ ] Commit + log

**Friday — Benchmark improvements**
- [ ] Measure gains.
- [ ] Commit + log

**Saturday — More reth PRs**
- [ ] Keep reth velocity (10+ execution PRs target M18).
- [ ] [Tempo] 2 hrs: **First Tempo PR claim**. You now have 12+ months of Reth/revm context AND tempo-evm-ext is scaffolded. Browse `tempoxyz/tempo` issues filtered by `good-first-issue` or `help-wanted`. Prefer issues touching TempoEvm or transaction-parsing surfaces. Pick ONE. Comment claiming. Begin implementation.
- [ ] Commit + log

**Sunday — Rest + End Month 15 review**

---

## Month 16: EOF + Integration with storage-trie

### Week 61 — EOF implementation

**Monday — EOF EIP deep re-read**
- [ ] Re-read EIP-3540, 3670. EOF container format.
- [ ] Commit notes

**Tuesday — EOF validation**
- [ ] Stack validation per EIP-3670.
- [ ] Commit + log

**Wednesday — Static relative jumps**
- [ ] Implement EIP-4200 opcodes.
- [ ] Commit + log

**Thursday — Functions (EIP-4750)**
- [ ] Implement CALLF, RETF, JUMPF.
- [ ] Commit + log

**Friday — EOF tests**
- [ ] Integrate EOF test vectors.
- [ ] [Tempo] 1.5 hrs at end of day: **Tempo PR #1 progress**. Continue implementation. EOF knowledge transfers. Working draft by EOD.
- [ ] Commit + log

**Saturday — revm EOF PR**
- [ ] If revm has EOF issues, contribute.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 62 — `exec-vm` + `storage-trie` integration

**Monday — Integration design**
- [ ] Design how exec-vm uses storage-trie via Database trait.
- [ ] Commit + log

**Tuesday — Implement integration**
- [ ] Wire up the two crates.
- [ ] Commit + log

**Wednesday — Integration tests**
- [ ] End-to-end execution with real storage.
- [ ] Commit + log

**Thursday — Benchmark integrated stack**
- [ ] Performance vs revm + reth storage.
- [ ] Commit + log

**Friday — reth evm PR**
- [ ] Reth-side contribution.
- [ ] [Tempo] 1 hr at end of day: **Tempo PR #1 submit**. Finish, run their CI locally, open the PR with clear motivation + test plan.
- [ ] Commit + log

**Saturday — Crate maintenance**
- [ ] storage-trie fixes if needed. exec-vm polish.
- [ ] [Tempo] 30 min: respond to any Tempo PR #1 review feedback. Don't let it sit.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 63 — Fuzz targets

**Monday — Fuzz setup**
- [ ] Setup cargo-fuzz. First target on opcode sequences.
- [ ] Commit + log

**Tuesday — Run fuzz, fix findings**
- [ ] Run fuzzer. Address crashes.
- [ ] Commit + log

**Wednesday — More fuzz targets**
- [ ] Fuzz gas metering. Fuzz call operations.
- [ ] Commit + log

**Thursday — Differential fuzzing**
- [ ] Fuzz exec-vm vs revm for consistency.
- [ ] Commit + log

**Friday — reth or revm PR**
- [ ] Contribution.
- [ ] Commit + log

**Saturday — Docs pass**
- [ ] exec-vm documentation.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min during ritual: review Tempo PR #1 status. If merged, log it and pick next candidate from bookmarked list. If still in review, address feedback.

---

### Week 64 — Revm performance PR push

**Monday — Identify revm perf opportunity**
- [ ] Deep profile revm in common scenarios.
- [ ] Re-open `EXEC_VM_PERF_BACKLOG.md` — specifically the SIMD candidate noted W60 Tue. If profiling confirms it's a hot spot in revm too, the optimization plus benchmark becomes a strong revm PR candidate this week.
- [ ] Commit notes

**Tuesday — Design optimization**
- [ ] Plan approach.
- [ ] Commit notes

**Wednesday — Implement**
- [ ] Code optimization.
- [ ] Commit + log

**Thursday — Benchmark**
- [ ] Measure improvement.
- [ ] Commit + log

**Friday — Submit revm PR**
- [ ] Clean PR.
- [ ] Commit + log

**Saturday — Respond to reviews**
- [ ] Iterate.
- [ ] [Tempo] 30 min: address Tempo PR #1 feedback. If merged, claim Tempo PR #2 candidate from bookmarks.
- [ ] Commit + log

**Sunday — Rest + End Month 16 review**

---

## Month 17: Architectural Discussions + Reth evm Features

### Week 65 — Architectural engagement

**Monday — GitHub discussions**
- [ ] Browse ongoing execution-layer architecture discussions.
- [ ] Commit notes

**Tuesday — Substantive comment**
- [ ] Write substantive architectural comment.
- [ ] Commit notes

**Wednesday — Proposal draft**
- [ ] Draft small design proposal for reth evm.
- [ ] Commit notes

**Thursday — Submit proposal**
- [ ] Post as GitHub discussion.
- [ ] Commit notes

**Friday — Engage discussion**
- [ ] Respond to feedback.
- [ ] [Tempo] 30 min: scan Tempo discussions tab on GitHub. Pick one substantive thread to read fully (not to comment). Note in `notes/tempo_discussions.md` who's driving the design conversation and what the open questions are. Reconnaissance, not engagement.
- [ ] Commit notes

**Saturday — Reth PR**
- [ ] Storage or evm.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 66 — Reth evm feature + `tempo-tx-envelope` v0.1.0

**Monday — Feature identification**
- [ ] Find meaningful reth evm improvement. Design.
- [ ] Commit notes

**Tuesday — Implementation**
- [ ] Start coding.
- [ ] Commit + log

**Wednesday — Continue**
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Submit + `tempo-tx-envelope` v0.1.0 ship**
- [ ] Reth feature PR ready.
- [ ] [Tempo] 3 hrs split across the day: **`tempo-tx-envelope` v0.1.0 build**. Mirror `tempoxyz/tempo`'s primitives crate. Define `TempoTransaction` (EIP-2718 type 0x76) struct with fields: chain_id, nonce, max_fee_per_gas, max_priority_fee_per_gas, gas, calls: Vec<Call>, fee_token: Address, valid_before: Option<NonZeroU64>, valid_after: Option<NonZeroU64>, auth: Authorization. Use eth-rlp derive (W5). Reuse eth-primitives types.
- [ ] [Tempo] Test: encode transaction with hard-coded fields, assert bytes match fixture pulled from tempoxyz/tempo's test data.
- [ ] [Tempo] Tag `tempo-tx-envelope v0.1.0` if tests pass.
- [ ] Commit + log

**Saturday — Another storage PR (maintain velocity)**
- [ ] Commit + log
- [ ] [Tempo] 1 hr: test `tempo-tx-envelope` end-to-end. Use `tempo-foundry`'s `cast` to send transaction to Tempo testnet (stablecoins from docs.tempo.xyz faucet). Assert acceptance. If fails, debug — likely an RLP edge case.

**Sunday — Rest + Weekly Ritual**

---

### Week 67 — exec-vm v1.0 prep

**Monday — API stabilization**
- [ ] Review all public APIs. Freeze signatures.
- [ ] Commit + log

**Tuesday — Docs pass**
- [ ] Every item documented. Examples.
- [ ] Commit + log

**Wednesday — Final benchmarks**
- [ ] Comprehensive suite.
- [ ] Commit + log

**Thursday — DESIGN.md**
- [ ] Document architectural decisions.
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Continue.
- [ ] [Tempo] 30 min: Tempo PR #2 progress check. By now you should have 2 Tempo PRs merged or 1 merged + 1 in review.
- [ ] Commit + log

**Saturday — Crate polish**
- [ ] Final cleanup.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 68 — exec-vm v1.0 ship

**Monday — Tag v1.0**
- [ ] Tag release.
- [ ] Commit + log

**Tuesday — Blog if ready**
- [ ] Consider writing exec-vm intro blog. No pressure.
- [ ] [Tempo] Update M18 Tempo metrics row: PRs merged (target 10 — flag if below 6), TIPs read (target 10), crates (target 2 — tempo-tx-envelope ✓ + tempo-evm-ext scaffold ✓), maintainer relationships (target 2 — anyone who reviewed Tempo PRs).
- [ ] Commit + log

**Wednesday — Reth feature iteration**
- [ ] Address reviews on feature PR.
- [ ] Commit + log

**Thursday — More reth**
- [ ] Continue velocity.
- [ ] Commit + log

**Friday — Reviews given**
- [ ] Review 3 others' Reth PRs substantively.
- [ ] [Tempo] Review 2 others' Tempo PRs substantively. Even one substantive Tempo review is a relationship-warming signal worth more than three Reth reviews at this stage. Note who you reviewed; goes in Tempo maintainer tracker.
- [ ] Commit notes

**Saturday — Month 17 close**
- [ ] Commit + log

**Sunday — Rest + End Month 17 review**

---

## Month 18: Phase 4 Close + Consensus Prep

### Week 69 — Final execution PRs

**Monday — Final feature push**
- [ ] Last medium-sized feature.
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
- [ ] Read Ethereum consensus layer intro. PoS high level.
- [ ] Commit notes

**Tuesday — Engine API spec preview**
- [ ] Read Engine API specification at high level.
- [ ] Commit notes

**Wednesday — Lighthouse survey**
- [ ] Browse Lighthouse code at high level.
- [ ] Commit notes

**Thursday — Reth engine crate preview**
- [ ] Browse reth/crates/engine.
- [ ] Commit notes

**Friday — Reth consensus crate preview**
- [ ] Browse reth/crates/consensus.
- [ ] Commit notes

**Saturday — Phase 5 prep (consensus-engine already scaffolded W24)**
- [ ] Re-read consensus-engine empty lib.rs. Sketch module layout in notes/ (engine_api, fork_choice, payload_builder, jwt, builder_api, state_root_validator).
- [ ] Identify which eth-* crates each module imports. Confirm dependency graph builds.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 71 — Phase 4 reflection

**Monday — Full Phase 4 assessment**
- [ ] Check exit criteria.
- [ ] Commit notes

**Tuesday — Metrics**
- [ ] Update North Star M18. Check 20+ execution PRs.
- [ ] Commit notes

**Wednesday — Relationship update**
- [ ] Which maintainers engaged. Depth.
- [ ] Commit notes

**Thursday — Blog consideration**
- [ ] Phase 4 retrospective. No deadline.
- [ ] Commit notes

**Friday — Wrap**
- [ ] Close outstanding PRs.
- [ ] Commit + log

**Saturday — Rest prep**
- [ ] Light day.
- [ ] Commit notes

**Sunday — Rest**

---

### Week 72 — Transition week

**Monday — Mental prep Phase 5**
- [ ] Read Phase 5 section. Outline Month 19.
- [ ] Commit notes

**Tuesday — Reading list for consensus**
- [ ] Compile reading list.
- [ ] Commit notes

**Wednesday — Reach out to Lighthouse folks**
- [ ] If any connections, warm up.
- [ ] Commit notes

**Thursday — Maintenance on previous crates**
- [ ] storage-trie, exec-vm bug fixes.
- [ ] Commit + log

**Friday — Final exec-vm polish**
- [ ] Any remaining items.
- [ ] Commit + log

**Saturday — Month 18 close**
- [ ] Final PRs.
- [ ] Commit + log

**Sunday — Rest**
- [ ] Phase 5 starts tomorrow.

---

# PHASE 5: CONSENSUS + ENGINE API (Month 19-24)

**Deliverable**: `consensus-engine` v1.0 + end-to-end integration capable of syncing Sepolia.

> Tempo Phase 5 budget: 5-7 hrs/wk. Payment lane design W82 Fri. `tempo-payment-lane` scaffold W83 Wed. Both Tempo crates v0.1.0 W91 Thu. Path D added to W96 decision.

Three-crate integration target (W85 Sepolia sync): consensus-engine orchestrates eth-network-codec → block ingestion → eth-stage::Pipeline (driving exec-vm + storage-trie + eth-trie::StateRoot) → engine_api for CL coordination.

## Month 19: Engine API Deep Dive

### Week 73 — Engine API specification

**Monday — Engine API full read part 1**
- [ ] Read Engine API spec sections 1-3.
- [ ] Commit notes

**Tuesday — Engine API full read part 2**
- [ ] Read sections 4-6.
- [ ] Commit notes

**Wednesday — newPayload deep**
- [ ] Study newPayload V1, V2, V3, V4.
- [ ] Commit notes

**Thursday — forkchoiceUpdated deep**
- [ ] Study fcU variants.
- [ ] Commit notes

**Friday — getPayload deep**
- [ ] Study getPayload variants.
- [ ] Commit notes

**Saturday — JWT auth**
- [ ] Study JWT auth used by Engine API.
- [ ] [Tempo] 30 min: while JWT is fresh, read how Tempo handles engine API auth. Tempo's CL-EL split is different (validator set is permissioned). Note implications in `notes/tempo_engine_diff.md`.
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 74 — Reth engine crate

**Monday — reth-engine structure**
- [ ] Browse reth/crates/engine. Map files.
- [ ] Commit notes

**Tuesday — Engine tree**
- [ ] Read engine tree implementation. Block tree for forks.
- [ ] Commit notes

**Wednesday — Payload builder**
- [ ] Read reth payload builder.
- [ ] Commit notes

**Thursday — First engine PR**
- [ ] Find docs or small fix.
- [ ] [Tempo] 30 min: scan Tempo's engine-API-adjacent PRs to see what kinds of issues are open there. Bookmark candidates.
- [ ] Commit + log

**Friday — `consensus-engine::engine_api` module skeleton**
- [ ] Create `consensus-engine/src/engine_api/{mod.rs, server.rs, types.rs}`. Define EngineApi trait with V1-V4 method signatures. Wire eth-network-codec::Codec for JSON-RPC framing.
- [ ] Re-export eth-rpc-types request/response types.
- [ ] Commit + log

**Saturday — JWT auth in `consensus-engine::engine_api::jwt`**
- [ ] Implement HS256 JWT auth middleware. Test against fixture token from Lighthouse deployment.
- [ ] [Tempo] 30 min: confirm JWT module is agnostic enough to work for both Ethereum-style engine API and Tempo's variant. If not, parameterize.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 75 — `consensus-engine`: core methods

**Monday — newPayload implementation**
- [ ] Implement newPayload V3 handler.
- [ ] Commit + log

**Tuesday — Payload validation**
- [ ] Block header validation.
- [ ] Commit + log

**Wednesday — forkchoiceUpdated**
- [ ] Implement fcU handler.
- [ ] Commit + log

**Thursday — getPayload**
- [ ] Implement getPayload.
- [ ] Commit + log

**Friday — Storage + engine integration**
- [ ] Wire up with storage-trie.
- [ ] Commit + log

**Saturday — Engine + exec-vm integration**
- [ ] Execute payload using exec-vm.
- [ ] [Tempo] 1 hr: **TIP-1031 reads-side wiring**. Confirm consensus-engine engine_newPayload handler can carry Tempo-style consensus-context field without breaking upstream Ethereum path. Cargo features (`tempo-consensus-context`) guard the field. Groundwork for W82-83 payment lane work.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 76 — Lighthouse CL perspective

**Monday — Lighthouse code survey**
- [ ] Browse Lighthouse execution interaction layer.
- [ ] Commit notes

**Tuesday — Lighthouse Engine API client**
- [ ] Read Lighthouse's side of Engine API.
- [ ] Commit notes

**Wednesday — Prysm perspective**
- [ ] Read Prysm equivalent (less depth).
- [ ] Commit notes

**Thursday — CL/EL lifecycle**
- [ ] Map full CL/EL communication flow.
- [ ] Commit notes

**Friday — Another reth engine PR**
- [ ] Continue velocity.
- [ ] [Tempo] 30 min: pick Tempo PR candidate from W74 bookmarks. If good one available, claim and begin. Otherwise push to W78.
- [ ] Commit + log

**Saturday — Crate: connection handling**
- [ ] Websocket/HTTP Engine API transport.
- [ ] Commit + log

**Sunday — Rest + End Month 19 review**

---

## Month 20: Full Engine API + State Transition Validation

### Week 77 — State transition validation

**Monday — STF theory**
- [ ] Read state transition function theory.
- [ ] Commit notes

**Tuesday — Consensus rules in execution**
- [ ] What execution layer validates per consensus rules.
- [ ] Commit notes

**Wednesday — Block validation**
- [ ] Implement block validation in crate.
- [ ] Commit + log

**Thursday — Receipt validation**
- [ ] Receipt consistency checks.
- [ ] Commit + log

**Friday — Gas limit validation**
- [ ] Block gas limit checks.
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Engine or consensus area.
- [ ] [Tempo] 1 hr: continue Tempo PR or claim new one. Aim for Tempo PR #3-4 merged by end of M20.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 78 — Fork choice integration

**Monday — Fork choice theory**
- [ ] Read fork choice rule (LMD-GHOST, Casper FFG).
- [ ] Commit notes

**Tuesday — Reth fork choice code**
- [ ] Read reth's fork choice handling.
- [ ] Commit notes

**Wednesday — Crate: fork choice**
- [ ] Implement fork choice processing.
- [ ] Commit + log

**Thursday — Safe/finalized tracking**
- [ ] Track safe, finalized, head blocks.
- [ ] Commit + log

**Friday — Reorg detection**
- [ ] Detect reorgs from fork choice updates.
- [ ] Commit + log

**Saturday — More reth PRs**
- [ ] Consensus or engine.
- [ ] [Tempo] 1 hr: Tempo PR work.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] [Tempo] 20 min: Tempo releases skim. T3 hardfork or successor is likely in flight — read the meta-TIP.

---

### Week 79 — Reorg handling

**Monday — Reorg theory**
- [ ] Deep understand reorg handling in execution.
- [ ] Commit notes

**Tuesday — State rollback**
- [ ] Implement state rollback on reorg. Leverage storage-trie snapshots.
- [ ] Commit + log

**Wednesday — Receipt reindexing**
- [ ] Handle receipt/log reindexing.
- [ ] Commit + log

**Thursday — Transaction re-pool**
- [ ] Handle moving txs back to mempool on reorg.
- [ ] Commit + log

**Friday — Reorg integration tests**
- [ ] Test various reorg scenarios.
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] [Tempo] 1 hr: Tempo PR.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 80 — Multi-branch state

**Monday — Multi-branch theory**
- [ ] Maintaining state across forks.
- [ ] Commit notes

**Tuesday — Branch state design**
- [ ] Design multi-branch state in crate.
- [ ] Commit + log

**Wednesday — Implement**
- [ ] Code branch state management.
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Integration with exec-vm**
- [ ] Speculative execution across branches.
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] [Tempo] 1 hr: Tempo PR. Tempo PR count should be 6-8 merged; flag if below 4.
- [ ] Commit + log

**Sunday — Rest + End Month 20 review**

---

## Month 21: PBS + Builder API + Invalid Payload Handling

### Week 81 — Invalid payload handling

**Monday — Invalid payload scenarios**
- [ ] Catalog all invalid payload cases from spec.
- [ ] Commit notes

**Tuesday — Invalid header**
- [ ] Handle invalid headers.
- [ ] Commit + log

**Wednesday — Invalid transactions**
- [ ] Handle invalid tx in payload.
- [ ] Commit + log

**Thursday — Invalid state root**
- [ ] Handle state root mismatch.
- [ ] Commit + log

**Friday — Latest valid hash logic**
- [ ] Implement LVH tracking.
- [ ] Commit + log

**Saturday — Reth PR + Tempo payment-lane prior-art read**
- [ ] Reth PR work.
- [ ] [Tempo] 2 hrs at end of day: **payment-lane prior-art read** (no code). Read tempoxyz/tempo's payment-lane / payload-builder implementation end-to-end. Locate the lane reservation logic — likely under `consensus/`, `payload-builder/`, or `block-builder/`. For each non-obvious choice (priority queue shape, fairness rule, unused-reservation handling, tip20-detection mechanism), one sentence in `notes/payment_lane_prior_art.md` capturing what they did and your first guess at why. This is the reading scaffold for W82 Fri's design sketch — do NOT design your own yet.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

**Monday — PBS theory**
- [ ] Read PBS (Proposer-Builder Separation) spec.
- [ ] Commit notes

**Tuesday — MEV-Boost architecture**
- [ ] Read MEV-Boost architecture.
- [ ] Commit notes

**Wednesday — Builder API spec**
- [ ] Read Builder API specification.
- [ ] Commit notes

**Thursday — Builder API in reth**
- [ ] Check reth's builder API support.
- [ ] Commit notes

**Friday — Crate: Builder API compat + Tempo payment lane design**
- [ ] Design builder API support in consensus-engine.
- [ ] [Tempo] 2 hrs: **Tempo design-partner-facing feature design start**. Open `notes/payment_lane_prior_art.md` from W81 Sat as reference. Design "payment lane" support in payload builder. Rule: configurable percentage of block gas (default 30%) reserved for TIP-20 transfers. If TIP-20 demand below reservation, rest is general. If above, TIP-20 wins. Sketch algorithm in `notes/payment_lane_design.md`: priority queue per category, fairness, what happens when reservation fully unused (give to general or burn slot). For each design choice, note whether it matches upstream Tempo or intentionally diverges (with reason).
- [ ] Commit + log

**Saturday — Implementation start**
- [ ] Begin builder API endpoints.
- [ ] [Tempo] 30 min: review payment lane sketch from yesterday. Identify the 2-3 hardest design choices. Note them; don't solve yet.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 83 — Builder API implementation + `tempo-payment-lane` scaffold

**Monday — Header submissions**
- [ ] Implement header submission flow.
- [ ] Commit + log

**Tuesday — Block submissions**
- [ ] Implement block submission flow.
- [ ] Commit + log

**Wednesday — Builder client + `tempo-payment-lane` scaffold**
- [ ] Implement builder client perspective.
- [ ] [Tempo] 2 hrs: **`tempo-payment-lane` scaffold**. Create `crates/tempo-payment-lane/` workspace member depending on consensus-engine. Define `LaneStrategy` trait: `fn select_transactions(&self, pool: &[PoolTx], gas_limit: u64) -> Vec<PoolTx>`. Empty default impl + `TempoLaneStrategy { tip20_reservation_pct: u8 }` skeleton.
- [ ] Commit + log

**Thursday — Builder integration tests + lane strategy impl**
- [ ] Existing builder integration tests.
- [ ] [Tempo] 1 hr: implement `TempoLaneStrategy::select_transactions` for simple case: split pool into tip20 vs general buckets (check fee_token field from tempo-tx-envelope), fill reservation from tip20 first, then general. Test against synthetic pool of 100 txs.
- [ ] Commit + log

**Friday — Reth PR + lane edge case**
- [ ] Reth PR.
- [ ] [Tempo] 1 hr: continue tempo-payment-lane. Handle edge case where tip20 reservation is unused — give to general.
- [ ] Commit + log

**Saturday — Flashbots docs + diff against upstream**
- [ ] Study Flashbots additional docs.
- [ ] [Tempo] 1.5 hrs: **Diff your prototype against upstream**. Read Tempo's payment-lane implementation in `tempoxyz/tempo` (likely under consensus/, payload-builder/, or block-builder/). Compare to your tempo-payment-lane prototype. Note 3 design choices that differ. For each, decide: port upstream, keep yours, or document trade. Add to payment_lane_design.md.
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 84 — Test harness

**Monday — Test harness design**
- [ ] Design CL/EL test harness.
- [ ] Commit notes

**Tuesday — Deterministic CL**
- [ ] Implement mock CL for testing.
- [ ] Commit + log

**Wednesday — Scenario DSL**
- [ ] Define DSL for test scenarios.
- [ ] Commit + log

**Thursday — Reorg scenarios**
- [ ] Reorg simulation tests.
- [ ] Commit + log

**Friday — Engine API conformance**
- [ ] Run crate against spec conformance tests.
- [ ] [Tempo] 30 min: also run stack against any public Tempo conformance suite if one exists. If not, note as gap.
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + End Month 21 review**

---

## Month 22: Cross-Subsystem Features + Integration Push

### Week 85 — Three-crate integration push

**Monday — Integration architecture**
- [ ] Design toy execution client using all 3 crates.
- [ ] Commit + log

**Tuesday — Boot sequence**
- [ ] Implement node startup.
- [ ] Commit + log

**Wednesday — Engine API → execution → storage flow**
- [ ] End-to-end flow.
- [ ] Commit + log

**Thursday — Sync from testnet**
- [ ] Attempt Sepolia sync using own stack (PRIMARY GOAL — THE Phase 5 deliverable).
- [ ] [Tempo] **Secondary stretch goal** (cap at 4 hrs total this week, not just today): attempt Tempo testnet sync using your stack with tempo-tx-envelope + tempo-evm-ext + tempo-payment-lane plugged in. Document blockers in `notes/tempo_sync_blockers.md`. Do NOT let this eat Sepolia time. If Sepolia isn't working, Tempo gets zero time.
- [ ] Commit + log

**Friday — Debug failures**
- [ ] Fix Sepolia sync issues.
- [ ] [Tempo] 30 min only if Sepolia is green: continue Tempo sync attempt.
- [ ] Commit + log

**Saturday — More debugging**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 86 — Cross-subsystem reth feature

**Monday — Feature identification**
- [ ] Find reth feature touching engine + storage.
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
- [ ] [Tempo] 30 min: Tempo PR work. Target Tempo PR count by end of M22: 12-15 merged.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 87 — PR reviews velocity

**Monday — Review 2 PRs substantively**
- [ ] [Tempo] 1 Reth + 1 Tempo PR. Mix sources from now on.
- [ ] Commit notes

**Tuesday — Review 2 more**
- [ ] [Tempo] 1 Reth + 1 Tempo PR.
- [ ] Commit notes

**Wednesday — Review discussion comments**
- [ ] Engage design discussions.
- [ ] Commit notes

**Thursday — Reth PR**
- [ ] Commit + log

**Friday — Review 2 more**
- [ ] [Tempo] 1 Reth + 1 Tempo PR.
- [ ] Commit notes

**Saturday — Crate maintenance**
- [ ] All three reth crates + Tempo crates.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 88 — Devcon attendance

**Monday-Friday — Devcon (dates vary)**
- [ ] Attend Devcon. Meet maintainers in person. Side events.
- [ ] [Tempo] If any Tempo team members or design partners (Stripe folks, Visa blockchain folks) at Devcon, request 1-on-1 in advance. Same priority as Reth core 1-on-1s. Update Tempo maintainer tracker after.
- [ ] Notes.

**Saturday — Travel home**

**Sunday — Post-conference ritual**
- [ ] Update Reth + Tempo trackers.
- [ ] Follow-ups.

---

## Month 23: Mentorship + RFC Work

### Week 89 — RFC consideration

**Monday — Identify RFC opportunity**
- [ ] Find area needing design doc.
- [ ] Commit notes

**Tuesday — Draft RFC + Tempo TIP decision**
- [ ] Write initial draft.
- [ ] [Tempo] Decide RFC target this week: Reth proposal OR Tempo TIP. Tempo TIPs are numbered like EIPs; bar is high but process is open. If your tempo-payment-lane prototype surfaced a real design issue (W83 Sat diff), a Tempo TIP draft is the right vehicle. Otherwise Reth RFC. Commit to one — don't do both.
- [ ] Commit notes

**Wednesday — Refine RFC**
- [ ] Iterate.
- [ ] Commit notes

**Thursday — Post RFC**
- [ ] Post as GitHub discussion (Reth OR Tempo per Tuesday's decision).
- [ ] Commit notes

**Friday — Respond to feedback**
- [ ] Engage commenters.
- [ ] Commit notes

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 90 — Mentorship practice

**Monday — Identify newcomer**
- [ ] Find newer contributor in Telegram. Offer help on their first PR.
- [ ] [Tempo] 15 min: identify any newcomer on Tempo side. Same offer.
- [ ] Commit notes

**Tuesday — Help them**
- [ ] Pair review.
- [ ] Commit notes

**Wednesday — Another mentee**
- [ ] Help another newcomer.
- [ ] Commit notes

**Thursday — Crate PR**
- [ ] Reth contribution.
- [ ] Commit + log

**Friday — Consensus-engine v1.0 prep**
- [ ] API review.
- [ ] Commit + log

**Saturday — Docs pass**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 91 — consensus-engine v1.0 ship + Tempo crates v0.1.0

**Monday — Final benchmarks**
- [ ] Commit + log

**Tuesday — DESIGN.md**
- [ ] Commit + log

**Wednesday — Release tag**
- [ ] `consensus-engine v1.0` tag (PRIMARY).
- [ ] Commit + log

**Thursday — Tempo crates ship**
- [ ] Integration example: full 3-crate Reth example.
- [ ] [Tempo] 1 hr: **Tag `tempo-evm-ext v0.1.0`**. Finalize at least TIP-1020 P256/WebAuthn precompile impls registered against exec-vm's registry. Test that registration works without forking exec-vm.
- [ ] [Tempo] 30 min: **Tag `tempo-payment-lane v0.1.0`**. Finalize lane reservation strategy from W83. Update README documenting strategy and trade-offs vs upstream Tempo.
- [ ] [Tempo] Update workspace root README to document all 3 Tempo crates alongside the 13 Reth crates.
- [ ] Commit + log

**Friday — Blog consideration**
- [ ] If ready, draft consensus-engine post.
- [ ] [Tempo] If writing Phase 5 blog, decide framing: "consensus-engine + Tempo payment lanes" reads more distinctive than just "consensus-engine." Lean into Tempo angle for distribution reach.
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 92 — Recognition push

**Monday — Engage major discussions**
- [ ] Architecture-level discussion contributions.
- [ ] Commit notes

**Tuesday — More PR reviews**
- [ ] 5+ substantive reviews (mixed Reth + Tempo).
- [ ] Commit notes

**Wednesday — Second RFC**
- [ ] If applicable, another design proposal.
- [ ] Commit notes

**Thursday — Reth PR**
- [ ] Commit + log

**Friday — Maintainer touch points**
- [ ] Engage each target Reth maintainer at least once.
- [ ] [Tempo] Tempo maintainer touch points: engage each of 2-4 Tempo maintainers you've built relationship with. Reference your tempo-payment-lane prototype. Ask for design feedback on one specific point, not generic input.
- [ ] Commit notes

**Saturday — End Month 23**
- [ ] Commit + log

**Sunday — Rest + End Month 23 review**

---

## Month 24: Phase 5 Close + Reassessment

### Week 93 — Final feature push

**Monday — Feature identification**
- [ ] Last major reth feature for Phase 5.
- [ ] Commit notes

**Tuesday — Implementation**
- [ ] Commit + log

**Wednesday — Continue**
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Submit**
- [ ] Submit final Reth feature PR.
- [ ] [Tempo] 30 min: Tempo PR final push if below 15 merged.
- [ ] Commit + log

**Saturday — Reviews**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 94 — Final PR push

**Monday — PR volume**
- [ ] Multiple smaller Reth PRs.
- [ ] [Tempo] 1 hr: any final Tempo PRs you can ship before reassessment.
- [ ] Commit + log

**Tuesday — Continue**
- [ ] Commit + log

**Wednesday — Reviews given**
- [ ] 5+ reviews across Reth + Tempo, mixed.
- [ ] Commit notes

**Thursday — Continue**
- [ ] Commit + log

**Friday — Final PRs**
- [ ] Commit + log

**Saturday — Wrap up**
- [ ] All outstanding items.
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 95 — Reassessment preparation

**Monday — Data collection**
- [ ] Count all Reth PRs merged. List features shipped. Update maintainer tracker.
- [ ] [Tempo] Count Tempo PRs merged. Target: 25. Acceptable: 15+. Below 10 = flag the gap honestly.
- [ ] [Tempo] Count Tempo crates shipped. Target: 3 (all live). Acceptable: 2.
- [ ] [Tempo] Update Tempo maintainer tracker depth scores.
- [ ] Commit notes

**Tuesday — Signal assessment**
- [ ] Any approaches from Reth-adjacent companies? Mentions by maintainers?
- [ ] [Tempo] Any approaches from Tempo team? From Tempo design partners (Stripe, Visa, Shopify, Deutsche Bank crypto teams)? Has anyone from upstream Tempo engaged substantively with tempo-payment-lane?
- [ ] Commit notes

**Wednesday — Three crates assessment**
- [ ] Quality of each (Reth: storage-trie, exec-vm, consensus-engine; Tempo: tempo-tx-envelope, tempo-evm-ext, tempo-payment-lane).
- [ ] Commit notes

**Thursday — Energy assessment**
- [ ] Sustainability check.
- [ ] Commit notes

**Friday — Two pulls to evaluate honestly**
- [ ] **Post-Reth systems pull**: Chronicle Queue / matching engine / Aeron urges?
- [ ] [Tempo] **Tempo gravity check**: have you been pulled into Tempo strongly enough that "Tempo full-time" is now a credible path? Specifically: ≥15 Tempo PRs merged AND ≥2 direct Tempo maintainer relationships AND has your tempo-payment-lane been substantively engaged by upstream? If yes to all three, Path D is real.
- [ ] Commit notes

**Saturday — Market state**
- [ ] Crypto cycle. Rust infra hiring climate. Stablecoin payments market state.
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 96 — Month 24 Decision

**Monday — Path A analysis**
- [ ] Extend Reth core for 6-12 months. What does this look like? Likelihood you accept a Reth-adjacent Tier A offer?
- [ ] Commit notes

**Tuesday — Path B analysis**
- [ ] Pivot to post-Reth systems (Chronicle Queue, Aeron, matching engine).
- [ ] Commit notes

**Wednesday — Path C analysis**
- [ ] Catch-up if Phase 5 slipped.
- [ ] Commit notes

**Thursday — Path D analysis (Tempo pivot, added)**
- [ ] [Tempo] **Path D**: pivot to Tempo full-time. Apply to Tempo directly, OR to a Tempo design partner with Tempo on their build surface (Stripe crypto, Visa blockchain, Shopify Payments). Real option ONLY if W95 Friday's three-condition test passed. Otherwise, Path D drops back into Tier A/B as generic "stablecoin payments infra" track.
- [ ] [Tempo] If Path D is real: what does next 6-12 months look like? Direct application, or build-in-public to attract inbound? Vietnam-remote compatibility check (Tempo is US/EU-distributed; check if remote roles are open).
- [ ] Commit notes

**Friday — Decision**
- [ ] Pick one path. Don't pick "do both." The plan was always one-track; pick one for next 6-12 months.
- [ ] Write decision in `progress.md` with supporting evidence.
- [ ] Commit notes

**Saturday — Phase 5 close + Tempo addendum close**
- [ ] Full Phase 5 review.
- [ ] Final Tempo metrics tally. Update North Star.
- [ ] [Tempo] Note in progress.md: what the Tempo extension was worth. Honest assessment: did Tempo pay off as optionality, or did it cost Reth velocity without proportional return?
- [ ] Commit + log

**Sunday — End 24-month plan**
- [ ] Full rest. Celebrate milestone.
- [ ] Next chapter prep starts following Monday based on Friday's path decision.

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

### Reth track (primary)

| Metric | M6 | M12 | M18 | M24 |
|--------|----|----|----|----|
| Paradigm ecosystem PRs merged | 10 | 25 | 50 | 80 |
| Reth PRs merged | 0 | 15 | 35 | 60 |
| Storage/Trie PRs | 0 | 10 | 20 | 30 |
| Execution PRs (revm + reth evm) | 0 | 3 | 10 | 20 |
| Consensus/Engine PRs | 0 | 0 | 3 | 10 |
| PR reviews given (substantive) | 0 | 10 | 40 | 100 |
| Features led end-to-end | 0 | 0 | 1 | 3 |
| Reth-side production crates shipped | 0 | 1 | 2 | 3 |
| Direct relationships with Reth maintainers | 1 | 3 | 5 | 8 |
| Conferences attended | 0 | 0 | 0 | 1 |

### Tempo track (additive)

| Metric | M6 | M12 | M18 | M24 |
|--------|----|-----|-----|-----|
| Tempo repo orientation depth (0-5) | 1 | 3 | 4 | 5 |
| Tempo PRs merged | 0 | 0 | 10 | 25 |
| TIP specs read end-to-end | 1 | 5 | 10 | all current |
| TIP discussions participated in (substantive) | 0 | 1 | 3 | 8 |
| Tempo-flavored workspace crates shipped | 0 | 0 | 2 | 3 |
| Direct relationships with Tempo maintainers | 0 | 1 | 2 | 4 |
| Tempo design-partner-facing feature shipped | 0 | 0 | 0 | 1 |

**Not goals**: "core maintainer of X" or "lead of all current maintainers." Status is the OUTPUT of shipped code, reviews, and design engagement — not a directly addressable target. Setting status as the goal corrodes focus on the work that produces status.

---

## Open Questions

*Running list. Close what resolves, carry what doesn't. If survives 2 weeks, dedicated slot.*

- [ ]
- [ ]

---

## Reth Maintainer Relationship Tracker

| Name | Role | First interaction | Last interaction | Depth 0-5 | Notes |
|------|------|------------------|-----------------|-----------|-------|
| Matthias Seitz (mattsse_) | Core reth | — | — | 0 | Target primary mentor |
| Georgios Konstantopoulos (gakonst) | CTO Paradigm | — | — | 0 | Ecosystem leader; also on Tempo |
| Dan Cline | Core reth | — | — | 0 | Storage/trie area |
| Oliver Nordbjerg | Core reth | — | — | 0 | — |
| Roman Krasiuk | Core reth | — | — | 0 | — |
| Dragan Rakita | Core reth / revm author | — | — | 0 | Critical Phase 4; also on Tempo |
| joshieDo | Core reth | — | — | 0 | Cross-project (also Tempo) |

Depth: 0=none, 1=reviewed PR, 2=back-and-forth, 3=tags you for area reviews, 4=DM relationship, 5=co-design

---

## Tempo Maintainer Relationship Tracker

| Name | Role | First interaction | Last interaction | Depth 0-5 | Notes |
|------|------|------------------|------------------|-----------|-------|
| Matt Huang | CEO Tempo, Paradigm co-founder | — | — | 0 | Strategic; not a code reviewer |
| Georgios Konstantopoulos (gakonst) | Paradigm CTO, active on Tempo PRs | (cross-ref Reth tracker) | — | 0 | Same person; track depth separately per project |
| Dragan Rakita | revm author, evm-relevant on Tempo | (cross-ref Reth tracker) | — | 0 | Same person |
| joshieDo | Reth + Tempo | (cross-ref) | — | 0 | Cross-project — high leverage |
| klkvr | Tempo PR reviewer | — | — | 0 | Engine/consensus area |
| legion2002 | Tempo contributor | — | — | 0 | Tx-envelope area |
| 0xrusowsky | Tempo contributor | — | — | 0 | Reth-revision-bumps area |
| figtracer | Tempo contributor | — | — | 0 | — |
| stevencartavia | Tempo contributor | — | — | 0 | — |
| SuperFluffy | Tempo consensus | — | — | 0 | Dynamic validators |
| danrobinson | Tempo docs / account abstraction | — | — | 0 | — |

Same depth scale. Update monthly.

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
| Tempo closes-sources or becomes Paradigm-internal | 25% | Reth contributions remain floor; Tempo time falls back to upstream revm/reth | — |
| Stripe walks; Tempo becomes orphan | 15% | Same as above; Tempo skills still transfer to other Reth-SDK chains | — |
| Tempo PRs crowd out Reth PRs | 50% | Hour cap enforced; Reth velocity primary metric reviewed monthly | — |
| You like Tempo and abandon Reth core | 30% | 70/30 Reth/Tempo ratio enforced through M18; revisit at M22 only | — |
| Tempo TIPs evolve faster than you can track | 60% | Weekly Sunday ritual skim; don't aim to know all TIPs, just storage- and execution-touching ones | — |
| Tempo's compliance / KYC features pull you toward business work you don't want | 30% | Reject contribution paths requiring KYC implementation; stay in execution + consensus + storage | — |

---

## Principles

1. Deliverables over hours. 5h target, 4h floor. Done at 3h → rest. Stuck at 6h → diagnose.
2. Three Reth production crates are the real output. Phase 1-2 exercises are not throwaways but they are not the deliverable either.
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
14. **Tempo is leverage on the Reth bet, not a parallel bet.** Treat it as such in budgeting, framing, and CV positioning.
15. **Path D at M24 is conditional, not aspirational.** Unlocks only if PRs, relationships, and design engagement are real. Otherwise, Path A/B/C as originally planned.
16. **The deliverable is shipped code on a chain that processes real payment volume.** Not "core maintainer status." Status is downstream of shipped code.

---

*Plan is a skeleton. Adjust weekly. Review monthly. Recalibrate quarterly. Reassess at Month 24. No re-plans before W95 (M22) outside Sunday rituals.*