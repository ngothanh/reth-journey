# Smart Pointers: Single vs Multi-Threaded Shared Mutability

Reference notes on the `Rc<RefCell<T>>` ↔ `Arc<RwLock<T>>` substitution, grounded
in the actual diff between `eth-storage-cache::LocalAccountCache` and
`SharedAccountCache`.

## The substitution table

| Local (single-threaded) | Shared (multi-threaded) |
|---|---|
| `std::rc::Rc<T>` | `alloc::sync::Arc<T>` |
| `std::cell::RefCell<T>` | `parking_lot::RwLock<T>` |
| `.borrow()` | `.read()` |
| `.borrow_mut()` | `.write()` |
| `Rc::clone(&x)` or `x.clone()` | `Arc::clone(&x)` |
| `Rc::try_unwrap(rc)` | `Arc::try_unwrap(arc)` |
| `RefCell::into_inner` | `RwLock::into_inner` (parking_lot: returns T directly) |
| Double-`borrow_mut` → **runtime panic** | Double-`write` → **blocks** (deadlocks on same thread) |
| Zero atomic overhead | Atomic ref-count + lock primitives |
| Cache miss-call overhead: 1 alloc (Rc + RefCell merged) | Cache miss-call overhead: 1 alloc + atomic write to refcount |

## Why each substitution

### `Rc` → `Arc`

`Rc<T>` uses a non-atomic `usize` reference count. Cheap. Cannot cross thread
boundaries — clone/drop would race on the counter, causing memory corruption.
`Rc<T>` is `!Send` and `!Sync`, so the compiler refuses to share it across
threads.

`Arc<T>` uses an `AtomicUsize` for the reference count. Clones and drops use
atomic increment/decrement (`fetch_add`/`fetch_sub`). Atomic ops cost ~5–20
cycles on x86 (cache-line locking when contended). Use `Arc<T>` when **any**
thread might clone or drop the shared value.

### `RefCell` → `parking_lot::RwLock`

`RefCell<T>` tracks borrows in a runtime field (essentially a `Cell<isize>`).
On `borrow()` / `borrow_mut()`, it checks/updates the counter and panics on
violation (multiple writers, or writer + readers). `RefCell<T>` is `!Sync`, so
it cannot cross threads either.

`RwLock<T>` is a real synchronization primitive backed by OS futexes (Linux) /
WAIT_ON_ADDRESS (Windows). On contention it can block the thread, suspending
execution until the lock is available. It's `Sync`, so multiple threads can
hold readers / one writer simultaneously.

#### Why `parking_lot::RwLock` instead of `std::sync::RwLock`

1. **No poisoning.** `std::sync::RwLock::write()` returns
   `LockResult<RwLockWriteGuard>` because the lock can be "poisoned" if a thread
   panicked while holding it. parking_lot drops poisoning entirely —
   `.read()` and `.write()` return guards directly, no `.unwrap()`.

2. **Faster fast paths.** parking_lot's uncontended `read()` / `write()` is
   ~2–3x faster than `std::sync::RwLock`. Better wait-list management on
   contention too.

3. **Reth's choice.** reth uses parking_lot pervasively, so types compose
   cleanly with the rest of the ecosystem. Mixing `std::sync::Mutex` and
   `parking_lot::Mutex` works but loses some optimization opportunities.

#### Different failure modes

| Misuse | `RefCell` behavior | `parking_lot::RwLock` behavior |
|---|---|---|
| Two `.borrow_mut()` / `.write()` on same thread | Panics immediately ("already borrowed") | **Deadlocks** — second `.write()` blocks waiting for the first, which the same thread holds |
| `.borrow()` while `.borrow_mut()` outstanding | Panics | Blocks |
| `.borrow_mut()` while `.borrow()` outstanding | Panics | Blocks |

The deadlock case is the dangerous one. `RefCell` fails loudly with a stack
trace. `RwLock` hangs silently — your program just stops making progress.
**Mitigation**: use `.try_read()` / `.try_write()` (return `Option<Guard>`) at
critical points, or enable parking_lot's `deadlock_detection` feature in dev
builds.

The `second_write_blocks_when_first_held` test in `shared_cache.rs` uses
`.try_write()` to observe the blocking without actually hanging.

## What's identical between the two caches

Looking at the diff between `local_cache.rs` and `shared_cache.rs`:

- Both wrap `HashMap<Address, _>` with the same `get_or_load` / `commit` shape.
- Both implement `Default` via `new()`.
- Both run `Rc::try_unwrap` / `Arc::try_unwrap` in `commit()` to extract owned
  `Account`s. Both panic if an outstanding handle exists.
- Both use the loader-closure pattern (`F: FnOnce(Address) -> Account`) so the
  caller controls how cache misses are resolved.

The shape — staging area with explicit commit, sharable-but-mutable entries,
loader-driven misses — is the same. Only the concurrency primitives differ.

## Cost comparison

On a modern x86 with uncontended access:

| Operation | `Rc<RefCell<T>>` | `Arc<parking_lot::RwLock<T>>` |
|---|---|---|
| Clone the pointer | ~1 cycle (non-atomic inc) | ~3-5 cycles (atomic inc) |
| Drop the pointer | ~1 cycle | ~3-5 cycles |
| `.borrow()` / `.read()` | ~2 cycles (counter check) | ~5-10 cycles (atomic CAS) |
| `.borrow_mut()` / `.write()` | ~2 cycles | ~5-10 cycles (atomic CAS) |
| Release the borrow / guard | ~1 cycle | ~5 cycles |

So `Arc<RwLock>` is roughly 3-5x slower per operation under no contention.
Under contention, `RwLock` can block, which trades latency for fairness and
correctness. None of this matters for the EVM's per-account cost — accounts
are touched once per opcode, ~1000s per tx, well under the per-CPU-cycle budget
either way. The atomic cost shows up at the cache-line bouncing level when
many threads contend on the same accounts (rare in practice).

## When to use which

| Situation | Pick |
|---|---|
| Single-threaded code (one transaction, one call frame) | `Rc<RefCell<T>>` |
| Concurrent reads, occasional writes | `Arc<RwLock<T>>` |
| Equal read/write or short critical sections | `Arc<Mutex<T>>` (often faster than `RwLock` because no reader/writer counter) |
| Shared, mostly read, atomic swap of whole value | `arc_swap::ArcSwap<T>` |
| Many writers to different keys | `DashMap<K, V>` (sharded internally) |

For your reth journey, this maps to:

- **Phase 1 (Week 2)**: `LocalAccountCache` (single-tx execution; `Rc<RefCell>`),
  `SharedAccountCache` (cross-tx within a block; `Arc<RwLock>`).
- **Phase 4 (revm)**: `Rc<RefCell<JournaledState>>` is what revm uses
  internally for nested call frames — single-threaded execution.
- **Phase 5 (consensus engine)**: `Arc<RwLock<BlockTree>>` shared between
  engine API handlers, payload builders, validators.
- **Reth-wide**: `Arc<Mutex<Database>>` for storage; `arc_swap` for "current
  chain head" pointer updates.

## What this doesn't cover

- **Lock-free shared mutable**: `AtomicUsize`, `AtomicPtr`, `crossbeam`
  channels, `arc-swap`. These avoid locks entirely via atomic operations on
  hardware primitives. Faster but require careful reasoning about memory
  ordering. Covered in W4 (atomics) and W2 Friday (arc-swap, dashmap docs).

- **Async-aware locks**: `tokio::sync::Mutex`, `tokio::sync::RwLock`. These
  yield to the runtime instead of blocking the OS thread, important for async
  code where blocking a thread blocks many tasks. Used in W3+ async work.

- **Cell types**: `Cell<T>` (Copy types only), `OnceCell<T>` (single-write
  cell), `LazyLock<T>` (lazy static initialization). Special-purpose cousins
  of `RefCell` with stricter or different rules.

- **The `Pin` / self-referential story**: covered in W3 with `MessageStream`.

## TL;DR

> Replace `Rc<RefCell<T>>` with `Arc<parking_lot::RwLock<T>>` when you cross
> thread boundaries. The API is nearly identical; the cost is atomic operations
> for refcount + real locks instead of a non-atomic counter + a `bool` flag.
> `RefCell` panics on misuse; `RwLock` deadlocks. Use parking_lot over std for
> no poisoning, faster fast paths, and reth ecosystem fit.
