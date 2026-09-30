# `SegQueue` R0 bench results — false sharing and the lock-free premium

Machine: Apple Silicon (aarch64), `CachePadded` line policy = 128 B.
Criterion `iter_custom`, per-item transfer cost, median of 10 samples / 300 ms.
Threads not pinned — read the **shape**, not the third digit.

## Variants measured

| | `head`/`tail` | `Segment::consumed` | `Segment::claimed` |
|---|---|---|---|
| **baseline** | — | — | — |
| **A** | `CachePadded` | — | — |
| **C** | `CachePadded` | `CachePadded` | `CachePadded` |
| **D** (kept) | `CachePadded` | `CachePadded` | — (falls on the next line anyway, since `CachePadded` pads its size too) |

## Numbers (ns / item, median)

| Scenario | baseline | A | C | D | `Mutex<VecDeque>` |
|---|---|---|---|---|---|
| uncontended (1 thread) | **8.70** | 8.75 | 15.09 | 15.17 | 15.70 |
| spsc (1P/1C) | 45.8 | 43.6 | 31.2 | **30.16** | 28.19 |
| mpmc 2P/2C | 44.9 | 41.6 | 44.4 | **40.19** | 53.96 |
| mpmc 4P/4C | — | — | — | 61.83 (range 54.5–69.1) | 55.23 |

## Findings

1. **The obvious false-sharing pair was the wrong one.** `head`/`tail` are 8 B
   apart and logically disjoint, which is why they looked like the culprit — but
   they are only written when a segment boundary is crossed, i.e. once per 32 ops.
   Padding them (A) is free and worth roughly nothing (43.6 vs 45.8 spsc, inside
   the noise band).

2. **The hot pair is `Segment::{claimed, consumed}`** — same 8 B adjacency, but
   written on *every* push and *every* pop. Padding it moved spsc
   **45.8 → 30.2 ns (−34%)**. "Adjacent + logically disjoint" is only half the
   test; the other half is write frequency.

3. **Padding is not free.** The uncontended control moved **8.70 → 15.17 ns
   (+74%)**: separating the counters makes the single-threaded path touch two
   cache lines per push+pop instead of one, and every 32 items allocates and
   initialises a larger `Segment`. The trade bought is −34% cross-core for +74%
   same-core. For an MPMC queue the cross-core path is the workload that matters,
   so D is kept — but the cost is real and stated.

4. **The lock-free premium is narrow at these thread counts.** `Mutex<VecDeque>`
   is not a straw man:
   - 1P/1C: **mutex wins** (28.2 vs 30.2) — no real contention, so the lock is
     almost always free and the segment machinery is pure overhead.
   - 2P/2C: **SegQueue wins, −26%** (40.2 vs 54.0) — the lock starts serialising.
   - 4P/4C: **mutex wins again** (55.2 vs 61.8, and SegQueue's range blows out to
     54.5–69.1).

5. **The 4P/4C inversion is the most useful result, and it is not about padding.**
   `pop` *spins* — on `state != WRITTEN` and on empty — instead of parking. With
   8 threads on an 8-thread machine the spinners steal CPU from the very threads
   they are waiting on, and the variance explodes. The mutex version blocks, so
   it degrades gracefully under oversubscription.

## Next perf item (not more padding)

Backoff / parking in `pop`, using the crate's existing `Backoff` and `Parker`.
Expected shape: 4P/4C median drops and the range tightens; 1P/1C should be
unaffected (the spin almost never triggers there). Re-run this bench before and
after, and keep the uncontended control as the tripwire.
