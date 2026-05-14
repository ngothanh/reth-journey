# Locks: `parking_lot` vs `std::sync`

Notes from the `MutexCache` / `RwLockCache` / `BatchedCache` exercises.
Companion to `05_smart_pointers.md`, which covered `parking_lot::RwLock` vs
`std::sync::RwLock`. The story for `Mutex` is almost identical — this file
captures the Mutex-specific points and the "keep parking_lot" conclusion.

## `parking_lot::Mutex` vs `std::sync::Mutex`

| Property | `std::sync::Mutex<T>` | `parking_lot::Mutex<T>` |
|---|---|---|
| Poisoning on panic | Yes — `.lock()` returns `LockResult<MutexGuard>` | **No** — `.lock()` returns the guard directly |
| Size of the lock | 1 word + state | **1 byte** of state (1 byte total when `T` is `()`) |
| Uncontended fast path | ~1 atomic + branch | ~1 atomic + branch, but no syscall fallback path baggage; benchmarks at roughly 2x |
| Contended wait | OS syscall via futex/`pthread_mutex` | Own parking lot — adaptive spinning, then park |
| Fairness | Not guaranteed | Not guaranteed by default; **`fair_unlock`** opt-in for hand-off |
| `const fn new` | Yes (stable) | Yes |
| Guard return | `MutexGuard<'_, T>` — wrapped in `Result` | `MutexGuard<'_, T>` — direct |
| `try_lock` | Returns `TryLockResult` (Result + Option) | Returns `Option<MutexGuard>` |

### What "no poisoning" actually buys you

`std::sync::Mutex` poisons the lock if a thread panics while holding it. Every
later `.lock()` returns `Err(PoisonError)`, forcing callers to either
`.unwrap()` (so the panic propagates) or call `.into_inner()` to opt back in.

In practice almost everyone writes `.lock().unwrap()` and moves on. The
poisoning bit just adds noise. parking_lot drops it entirely — the cost is
that you can read partially-updated state from a previously-panicked thread,
which is a fair trade if you've designed your invariants to be re-entrant or
panic-safe (which you usually have, since you write your panic recovery
around the data, not the lock).

### Footprint

`std::sync::Mutex<T>` on Linux x86-64 is roughly 40 bytes of overhead (it
embeds a `pthread_mutex_t` plus poison flag). `parking_lot::Mutex<T>` is **1
byte** of overhead — the parking happens out-of-band in a global hash table
keyed by the lock's address. For a struct like `MutexCacheInner` that holds
four large `HashMap`s, this doesn't matter. For a per-entry lock pattern
(`HashMap<K, Arc<Mutex<V>>>`) with millions of entries, it adds up to MBs.

### Failure modes (same as the RwLock table in 05)

Double-`.lock()` on the same thread **deadlocks** with parking_lot, doesn't
panic. Same mitigation: `.try_lock()` returns `Option<MutexGuard>`, and the
`deadlock_detection` feature can be enabled in dev builds.

## Why reth uses parking_lot

`Cargo.toml` in reth has `parking_lot` as a workspace dependency; it shows up
across `crates/storage/`, `crates/network/`, `crates/transaction-pool/`,
basically anywhere there's shared state. revm uses it too. Three reasons:

1. **Consistency.** Mixing `std::sync` and `parking_lot` in one type tree
   means callers have to remember which `.lock()` returns a Result. One
   flavor everywhere = no `.unwrap()` litter.
2. **Footprint.** Reth has lots of fine-grained per-entry locks (e.g. the
   transaction pool's per-account state). 40 bytes vs 1 byte per lock
   compounds at scale.
3. **Throughput.** Contended paths are common in block-execution hot loops;
   parking_lot's parking-lot algorithm wins the benchmarks reth's
   maintainers cited when they migrated off `std`.

## Decision for this project

**Keep `parking_lot::Mutex` and `parking_lot::RwLock`** everywhere.
- No `.unwrap()` on `.lock()`.
- Aligned with reth.
- Footprint matters for the per-entry pattern in `SharedAccountCache`.

## When `std::sync::Mutex` would be the right call

- **No extra dependency allowed.** The std lock is stdlib-only.
- **You want poisoning.** It's an actual feature if your invariants can't
  survive a mid-mutation panic and you'd rather propagate the failure than
  observe partial state.
- **`const` context with stability concerns on MSRV.** Stable `const fn new`
  has been available on both for a while, but if you're stuck on an older
  toolchain, std got there first.

None of those apply here, so: parking_lot it is.

## TL;DR

`parking_lot::Mutex` is `std::sync::Mutex` minus poisoning, minus syscall
overhead on the fast path, and minus ~39 bytes of footprint. Cost: the same
"misuse hangs instead of panics" trade you accept for `parking_lot::RwLock`.
Reth uses it pervasively, so we use it pervasively too.
