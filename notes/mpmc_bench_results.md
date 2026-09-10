# MPMC ring — bench results

Machine: aarch64-apple-darwin (Apple M2). Threads not pinned; read the shape, not
the third significant figure. Run `cargo bench -p concurrent --bench mpmc`.

## BEFORE false-sharing padding (head/tail adjacent, unpadded)

Quick run (--warm-up-time 0.5 --measurement-time 2), 2026-09-11:

| scenario | ns/item (median) |
|---|---|
| uncontended (1 thread, control) | ~7.3 |
| spsc (1P1C)  | ~50.9 |
| mpmc (2P2C)  | ~97.9 |

> Re-run with default (full) measurement time before committing final numbers.

## AFTER false-sharing padding (head / tail each in `CachePadded`, separate lines)

Quick run (--warm-up-time 0.5 --measurement-time 2), 2026-09-11:

| scenario | ns/item (median) | Δ vs before |
|---|---|---|
| uncontended (control) | ~7.0 | ~unchanged (7.3 → 7.0, noise) — validates the experiment |
| spsc | ~8.8 | **~5.8× faster** (50.9 → 8.8); near the ~7.0 single-thread floor |
| mpmc (2P2C) | ~62.3 | ~1.6× faster (97.9 → 62.3); residual cost is CAS contention, not false sharing |

Conclusion: padding `head`/`tail` apart is the whole win. spsc lands ~1.8 ns above the
uncontended floor — too small a gap to justify per-`Cell` (seq/payload) padding and its
4x memory cost for small payloads. Trục-2 (adjacent-cell seq false sharing) NOT worth it here.

> Re-run with default (full) measurement time before committing final numbers.
