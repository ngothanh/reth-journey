# W004 — Facts & Distilled Knowledge

**Theme:** Layer-1 concurrency primitives + `eth-primitives` hardening, each a reusable
production component mirroring an upstream module. Verified *correct* (loom + miri) then
*fast* (criterion + `cargo asm`).

**Crates shipped (4):** `concurrent` (v0.1.0), `eth-primitives` (v0.2.0),
`eth-chain-state` (v0.1.0), `eth-primitives-derive`.

---

## 1. `CachePadded<T>` — `concurrent/src/cache_padded.rs`

**Mental model:** pad any `T` up to the target's cache-line size so two adjacent
`CachePadded<T>` never share a line → no MESI ping-pong on cross-thread writes to
neighbouring atoms (Vyukov head/tail, Disruptor cursor, matching-engine bid/ask).

- Cache line is a **target-dependent constant**: 128 B on aarch64/Apple (L2 prefetcher
  pulls 128-B pairs, so 64 B still false-shares) and powerpc64; 64 B on x86_64. Selected
  via stacked `#[cfg_attr(..., repr(align(N)))]` because `align(N)` needs a *literal*.
- **`repr(C)` is non-negotiable** on any struct holding `CachePadded` fields: `repr(Rust)`
  may reorder fields and shuffle the padding away → false sharing returns silently.
- Single-field wrapper whose only job is a layout property → expose via **`Deref`/`DerefMut`**
  so callers use `T` transparently.
- Layout invariants are **transitive**: `repr(C, align(128))` propagates; a type-system
  invariant exists ONLY when a const-assert / compile_fail proves it.
- **Mirror:** LMAX Disruptor `Sequence` (56 B `LhsPadding` + 8 B value + 56 B `RhsPadding`
  = 120 B); we get the same isolation via `repr(align(128))` with no manual padding fields.

**Measured (aarch64-apple-darwin), `benches/false_sharing.rs`:** 2-thread ping-pong,
`fetch_add(1, Relaxed)` on adjacent counters → **bare 12.75 ns/op vs padded 2.09 ns/op =
6.1× cliff** (≥3× threshold ✅; same order as LMAX's 4.9×). Bench MUST carry a layout
assertion (`offset_of!` bare<128, padded≥128) + counter end-state assert, or it can
silently measure nothing.

## 2. `OnceLock<B256>` lazy hash cache — `eth-primitives/src/atomic_hash.rs`

**Mental model:** a lazy, thread-safe memoization slot. First reader computes keccak256;
the rest read the cached value.

- A lazy cache on a `Sync` type must use a **`Sync` cell** (`OnceLock`, not `Cell`/`OnceCell`).
- **Don't reimplement once-init from raw atomics** — std/OS primitives already have the
  synchronization right.
- A memoization cell is **not part of identity** → exclude it from `Eq`/`Hash`/`Clone` reset.
- Memoize only **immutable** state; caching over mutable state is a correctness bug.

## 3. `ChainHead` SeqLock — `eth-chain-state/src/chain_head.rs`

**Mental model:** a sequence counter that's **even when stable, odd mid-write**. Readers
retry until they see a stable even seq bracketing an untorn payload.

- A SeqLock makes a multi-word read *appear* atomic via **retry, not locking**.
- Correctness depends on payload access being **provably fenced between two `Acquire`
  loads of the seq** (the load-bearing invariant loom checks).
- The **seq counter and payload must live on separate cache lines**.
- SeqLock removes *reader blocking*, not *writer exclusivity*; ideal for 1-writer /
  many-reader hot state like the canonical chain tip.

## 4. `Backoff` — `concurrent/src/backoff.rs`

**Mental model:** 3-stage waiting ladder — spin ~1–100 ns (hopeful CAS-retry), then
`yield_now` (let the holder run), then signal the caller to park. `Backoff` never blocks.

- `spin()` stays in the spin band (retry your own CAS); `snooze()` escalates spin→yield
  (waiting on *another* thread). `is_completed()` (step > YIELD_LIMIT) = "go park".
- `hint::spin_loop()` is the cheapest busy-wait line — always include it.
- Backoff state is **only a hypothesis about contention shape**; reset on success.
- **Yield ≠ sleep** — yield is "let someone else run a slot"; sleep needs park.
- Building Backoff is useless unless callers **drive it** (see AtomicCell §6 outcome).

## 5. `Sealed<T>` — `eth-primitives/src/sealed.rs`

**Mental model:** generic wrapper pairing any `T: Sealable` with its memoized hash, and
making the inner value **immutable** (no `&mut` / mutable escape hatch).

- Variance **leaks from fields to the struct** — one invariant field makes the whole
  wrapper invariant. (W004's variance lesson.)
- Cache is carried by `Clone`, never reset; cache is not identity (excluded from `Eq`/`Hash`).

## 6. `AtomicCell<T>` + Pod hardening — `concurrent/src/atomic_cell.rs`, `src/pod.rs`

**Mental model:** lock-free read/write for any `T` that fits an atomic word. 8 B/align-8 →
bit-cast to `AtomicU64` (fast path); everything else → `AtomicBool` spinlock + `UnsafeCell`.

- **Fast/slow dispatch must be a type-level fact, not a runtime branch** — gated by
  `const { size_of::<T>()==8 && align_of::<T>()==8 }` so the slow arm is dead code and the
  asm collapses. Verified: `store` → single `stlr`, `load` → single `ldapr` (no branch/CAS).
- A runtime size+align check is **necessary but not sufficient**: a `#[repr(C, align(8))]`
  struct with padding passes the gate but exposes uninit bytes to the `AtomicU64` view → UB
  (miri: "encountered uninitialized memory"). The type must **promise no padding**.
- Fix = **`pub unsafe trait Pod: Copy`** bound on the cell. `Pod` is *unsafe* because the
  compiler can't verify "no padding / every bit pattern valid" — the implementor swears to
  it (same shape as `Send`/`Sync`). The bound makes `AtomicCell<PaddedStruct>` a **compile
  error**; that compile-fail (trybuild) is the load-bearing artifact, not a passing test.
  Two sub-properties bundled: no-padding (store side) + any-bit-pattern-valid (load side) —
  bytemuck splits these as `NoUninit` + `AnyBitPattern`.
- Array impl is sound by derivation: `unsafe impl<T: Pod, const N: usize> Pod for [T; N]`
  (arrays have no inter-element padding).
- A spinlock beats a mutex for sub-µs critical sections (no syscall); its acquire/release
  are the *only* thing keeping the section atomic.

**Measured:** fast store 0.31 ns / load 0.29 ns; slow uncontended ~4.3 ns. **Cliff = 14.1×**
(target ≥20× — missed because the *uncontended spinlock is unusually cheap* on Apple
Silicon, not a bug). 4-thread contended store: wiring **`Backoff::snooze`** into the
acquire loop eliminated the starvation tail (max 8.18 ms → 116 µs, p999 343 µs → 12 µs,
p50 1 µs → 166 ns) but **p99 = 5.21 µs still misses 2 µs** — under saturating contention the
`yield`→reschedule cost (~µs) is the p99 floor; a *yielding* spinlock can't beat it.

## 7. `BytesMut` — `eth-primitives/src/bytes_mut.rs` (week's first `unsafe`)

**Mental model:** growable byte buffer by hand — `ptr`, `len`, `cap`; geometric growth.

- Every `dealloc`/`realloc` `Layout` must **byte-for-byte match** the `alloc` that made it.
- Zero-capacity buffer holds a **dangling sentinel pointer**, not a real allocation — never
  dealloc it.
- Capacity past `len` is **uninitialized — never read it**.
- Ownership transfer of a raw allocation ⇒ **exactly one Drop** runs for it.
- Any op that may reallocate takes **`&mut self`**. Growth is geometric (amortized O(1)).

## 8. `Parker` / `Unparker` — `concurrent/src/parker.rs`

**Mental model:** 3-state machine (EMPTY / PARKED / NOTIFIED) over a futex; fast path is
atomic-only, syscall is the cold branch. Defends against **lost wakeups**.

- The block primitive's first move is a CAS that **publishes "I'm about to block"**; the
  store of the wake signal must be **Release** so the parked thread sees published data.
- An OS wake can fire **spuriously** — always **re-check state in a loop** after waking.
- `futex_wait`'s expected-value arg must match the atomic at call time — it's the
  load-bearing safety net for the CAS-to-park vs unpark race.
- **Lost-wakeup is a state-machine bug, not (only) a memory-ordering bug**; loom catches
  both — but only if your atomics are `loom::sync::atomic` (else the model tests nothing).

## 9. `b256!` / `address!` const macros — `eth-primitives/src/macros.rs`

- "Compile-time constant" means **every op in the expansion is `const`-evaluable**.
- A fixed-width literal macro must **reject wrong-length input** with a clear message; in
  const eval, **`panic!` is the error mechanism** (no `Result`).
- "Is X const-stable?" is answered **empirically** (try it; read the compiler error).

## 10. `#[derive(SimpleEncode)]` — `eth-primitives-derive/` (syn + quote)

- A proc-macro crate is **macros-only**; the traits/types it references live in a normal crate.
- Generated code must name **every external item by absolute path** (`::core::…`) — bare
  names are invisible at the call site.
- Report unsupported input shapes as a clean `compile_error!`, not a panic.
- Every binding the macro introduces needs a **hygienic / non-colliding name**.

---

## Cross-cutting methodology

**Microbenchmarking (sub-10 ns ops):**
- Criterion's ~10 ns clock+dispatch overhead swamps the op → **batch with `iter_custom`**;
  criterion supplies `iters`, you do exactly that many and return total `Duration`.
- LLVM elides unobserved work → **`black_box` operands AND assert counter end-state**;
  multi-billion ops/sec is the elision tell.
- A bench whose two arms differ by one wrapper MUST assert the wrapper changed the layout.
- Every microbench gets a **physical-plausibility negative control** (fastest L1 atomic on
  aarch64 ≈ 3 ns/op).
- Perf thresholds are **hypotheses about hardware** — when the chip says no, adjust the
  threshold and *document*, don't gaslight the bench.

**`loom` model-checking:**
- loom only intercepts **`loom::sync::atomic`** (gate imports under `cfg(loom)`); a
  suspiciously-fast pass (<100 ms) means loom didn't intercept — it tested nothing.
- Models are **deliberately minimal** (2 threads × ~3 ops); correctness proven on the tiny
  model, not a realistic workload.
- loom uses `loom::thread::yield_now` as its scheduling point; it can't model
  `hint::spin_loop` or `std` yields — so `Backoff` is off-loom only; loom keeps `spin_hint`.
- **Gotcha caught this week:** `eth-chain-state` declared `loom` as a *dev-dependency* but
  used it in *library* code → the ChainHead model never actually compiled under loom. Must
  be a regular `[target.'cfg(loom)'.dependencies]` (mirror `concurrent`).

**`unsafe` marker traits (`Pod`/`Send`/`Sync`):** the compiler verifies nothing; the
contract lives in the `# Safety` doc-comment and the `unsafe impl` keyword is the forcing
function. The bound converts "any safe caller can hit UB" into "UB only via a deliberate,
auditable lie in one `unsafe impl`". A compiler-checked guarantee needs a **derive macro**
(bytemuck-style) that rejects padded structs, not the bare trait.

---

## Verification artifacts (live in the repo)
- `concurrent/notes/atomic_cell_fast_path.asm.txt` — single-instruction fast path.
- `concurrent/notes/atomic_cell_bench_results.md` — fast/slow/4-thread, PASS/FAIL + misses.
- `concurrent/notes/cache_padded_bench_results.md` — 6.1× false-sharing cliff.
- trybuild compile-fail: `AtomicCell<PaddedStruct>` rejected with `Padded: Pod` not satisfied.
- loom: ChainHead `no_torn_read`, Parker ×2, AtomicCell slow-path — all pass under `--cfg loom`.

## Honest misses (documented, not hidden)
- AtomicCell cliff ratio **14.1×** vs ≥20× target (Apple Silicon's uncontended spinlock is
  ~4.3 ns — cheap). 4-thread p99 **5.21 µs** vs ≤2 µs (yield-reschedule floor under
  saturating contention). Both are hardware reality, documented in the results notes; the
  spinlock now uses `Backoff` so the catastrophic ms-scale tail is gone.
