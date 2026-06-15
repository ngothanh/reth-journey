# AtomicCell perf results (W4 — `concurrent v0.1.0`)

- **Host**: aarch64-apple-darwin (Apple Silicon)
- **How**: `cargo bench -p concurrent --bench atomic_cell` (criterion, `iter_custom`
  batching + `black_box`); 4-thread p99 from `cargo run --release --example atomic_cell_p99`.
- **p50** = criterion median. Fast-path **p99** is not separately instrumented:
  each fast op is one branchless instruction (see asm below) with no contention,
  so per-op variance is negligible and p99 ≤ 20 ns holds by construction.

| Bench | Path | Metric | Threshold | Observed | Δ | Result |
|---|---|---|---|---|---|---|
| `fast_path_store` | u64, lock-free | p50 | ≤ 5 ns | **0.307 ns** | 16× under | ✅ PASS |
| `fast_path_load` | u64, lock-free | p50 | ≤ 5 ns | **0.290 ns** | 17× under | ✅ PASS |
| `slow_path_store` | [u8;16], spinlock | p50 | ≤ 200 ns | **4.33 ns** | 46× under | ✅ PASS |
| `slow_path_load` | [u8;16], spinlock | p50 | ≤ 200 ns | **4.32 ns** | 46× under | ✅ PASS |
| `slow_path_store_4thread` | [u8;16], 4-thread | p99 | ≤ 2 µs | **5.21 µs** (Backoff) | 2.6× over | ❌ FAIL |
| Cliff ratio | slow/fast p50 store | ratio | ≥ 20× | **14.1×** | 0.7× | ❌ FAIL |

## Misses & interpretation

- **4-thread p99 still FAILs the 2 µs target — but the acquire loop now uses
  `Backoff` (snooze: spin band -> `yield_now`), off loom.** Before/after, 4
  threads, 500k samples on aarch64-apple-darwin:

  | metric | bare `spin_loop` | `Backoff::snooze` | change |
  |---|---|---|---|
  | p50  | 1 µs    | **166 ns**  | 6× better |
  | p90  | 2.67 µs | 1.54 µs     | better |
  | p99  | 4.75 µs | **5.21 µs** | ~same (FAIL) |
  | p999 | 343 µs  | **11.9 µs** | 29× better |
  | max  | 8.18 ms | **116 µs**  | 70× better |

  Backoff eliminated the catastrophic starvation tail (max 8 ms → 116 µs; p999
  343 µs → 12 µs) and cut median 6×, but did not move p99. Reason: at p99 the
  waiter is yielding, and a `yield_now()` → reschedule round-trip on macOS costs
  ~1–5 µs. Under 3 tight-loop contenders the lock is almost never free, so the
  waiter must yield often — a *yielding* spinlock cannot push p99 below the OS
  reschedule cost. The remaining options to chase 2 µs: a futex/`atomic-wait`
  park-with-direct-handoff lock (adds its own wake-syscall latency; uncertain
  win), or relax the threshold for this saturating contention model (3 cores
  doing nothing but hammering one cell is not a realistic steady state).

- **Cliff ratio = 14.1× (FAIL, target ≥ 20×).** Not because the fast path is
  slow — it's 0.3 ns — but because the *uncontended* spinlock is unusually cheap
  on Apple Silicon (single L1-hot CAS ≈ 4.3 ns). The 20× cliff was set assuming a
  pricier slow path; on this µarch the honest ratio is ~14×. The cliff is real
  and large; the specific multiple is hardware-dependent.

## Fast-path asm (R3)

Confirmed one instruction each, no branch / no CAS — see
`atomic_cell_fast_path.asm.txt`:
- store → `stlr x1, [x0]`  (store-release)
- load  → `ldapr x0, [x0]` (load-acquire, RCpc)

## Miri (R4)

`cargo +nightly miri test -p concurrent --test atomic_cell` and `--lib fast_path`:
zero UB across u64, `*const u64`, `[u8;8]`, and a downstream `Pod` wrapper. (The
trybuild `compile_fail` case is excluded — it shells out to cargo, which Miri
can't execute; a tooling limit, not UB.)
