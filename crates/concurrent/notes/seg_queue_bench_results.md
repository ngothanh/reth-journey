# `SegQueue` R0 bench results — false sharing, backoff, and the lock-free premium

Machine: Apple Silicon (aarch64), `CachePadded` line policy = 128 B.
Criterion `iter_custom`, per-item transfer cost, median of 10 samples / 300 ms.
Threads not pinned — read the **shape**, not the third digit.

**Noise floor, measured:** across runs the *unchanged* `mutex_vecdeque` arms moved
by 3–5% with `p = 0.00`. Anything under ~5% here is drift, not a result.

---

## Round 1 — cache-line padding

### Variants

| | `head`/`tail` | `Segment::consumed` | `Segment::claimed` |
|---|---|---|---|
| **baseline** | — | — | — |
| **A** | `CachePadded` | — | — |
| **C** | `CachePadded` | `CachePadded` | `CachePadded` |
| **D** (kept) | `CachePadded` | `CachePadded` | — (lands on the next line anyway: `CachePadded` pads its size too) |

### Numbers (ns/item, median)

| Scenario | baseline | A | C | D | `Mutex<VecDeque>` |
|---|---|---|---|---|---|
| uncontended (1 thread) | **8.70** | 8.75 | 15.09 | 15.17 | 15.70 |
| spsc (1P/1C) | 45.8 | 43.6 | 31.2 | **30.16** | 28.19 |
| mpmc 2P/2C | 44.9 | 41.6 | 44.4 | **40.19** | 53.96 |
| mpmc 4P/4C | — | — | — | 61.83 (54.5–69.1) | 55.23 |

### Findings

1. **The obvious false-sharing pair was the wrong one.** `head`/`tail` are 8 B
   apart and logically disjoint — but they are only written when a segment
   boundary is crossed, once per 32 ops. Padding them (A) is free and worth
   roughly nothing.
2. **The hot pair is `Segment::{claimed, consumed}`** — same adjacency, written on
   *every* push and pop. Padding it: spsc **45.8 → 30.2 ns (−34%)**.
   "Adjacent + logically disjoint" is only half the test; the other half is write
   frequency.
3. **Padding is not free.** The uncontended control moved **8.70 → 15.17 ns
   (+74%)**: two cache lines per push+pop instead of one, plus a larger `Segment`
   to initialise every 32 items. Kept anyway — cross-core handoff is the workload
   an MPMC queue exists for — but the cost is real.

---

## Round 2 — backoff instead of bare spin

Two separate spin sites, and **only the first belongs to the queue**:

- **inside `pop`**, while a consumer holds a slot whose producer has not stored
  the value yet → now `Backoff::snooze` (exponential spin bursts, then
  `yield_now`) instead of a bare `spin_loop`.
- **in the caller's drain loop**, on `pop() == None` → the queue returns
  `Option`, so the empty-wait policy is deliberately the caller's. Changed in the
  bench harness for **both** arms so the comparison stays fair.

### Numbers (ns/item, median; two runs where variance mattered)

| Scenario | D, bare spin | + backoff in `pop` | + backoff in caller too | `Mutex<VecDeque>` |
|---|---|---|---|---|
| uncontended | 15.17 | 15.02 | 14.84 / 12.67 | 15.65 / 15.67 |
| spsc | 30.16 | 28.34 | **13.81 / 12.47** | 27.87 / 27.95 |
| mpmc 2P/2C | 40.19 | 23.07 (range 19–32) | 41.30 / 42.52 | 51.55 / 54.54 |
| mpmc 4P/4C | 61.83 | 59.67 | 46.19 / **66.78** ⚠ | 51.91 / 54.32 |

### Findings

4. **Backoff inside `pop` alone: noise-level.** −2% to −6%, within the measured
   drift. The in-pop wait is genuinely short, so bounding it is correctness
   hygiene (a descheduled producer no longer has its core stolen) rather than a
   throughput win.

5. **The caller's empty-wait policy dominated the whole benchmark.**
   spsc **28.3 → 12.5 ns (−56%)**, reproducible across runs. A consumer that
   hot-spins on `None` floods the shared atomics and *starves the producer it is
   waiting for*. Backing off hands the core back and the producer runs.

6. **The mutex baseline was accidentally providing backpressure.** Its consumer
   must take the lock to discover the queue is empty, which self-throttles it and
   leaves the producer room — which is why the mutex arm barely moved when the
   harness gained backoff. The lock's "flaw" (serialisation) was doing useful
   work. A lock-free queue has no such throttle and has to re-create it
   explicitly. This is the cost side of the `pop() -> Option<T>` contract: the
   queue refuses to own the empty-wait policy, so the caller can and will get it
   wrong.

7. **Reproducible verdict vs `Mutex<VecDeque>`** (with both sides backing off):
   - uncontended: SegQueue wins (12.7–14.8 vs 15.7)
   - spsc 1P/1C: SegQueue wins **~−55%** (12.5 vs 27.9) — stable across runs
   - 2P/2C: SegQueue wins **~−22%** (41–43 vs 52–55) — stable
   - **4P/4C: no claim.** SegQueue's median straddles the mutex across runs
     (46.2 then 66.8; in-run ranges 39–56 and 57–74) while the mutex is steady at
     ~54. At 8 threads on this machine the variance *is* the defect.

---

## Open items

- **4P/4C instability** is the one unresolved perf issue. Suspects, in order:
  Apple Silicon P/E-core asymmetry (the scheduler migrates threads between core
  types mid-run), remaining CAS contention on `claimed`/`consumed`, and
  oversubscription at 8 threads on an 8-thread host. Next step is to pin threads
  (or drop to 3P/3C) before drawing any conclusion — an unpinned 8-thread run on
  a heterogeneous-core laptop may simply not be a measurable configuration.
- Re-run this whole file at each rung of the reclamation ladder (R1 refcount,
  R2 hazard pointers, R3 epoch). The harness now has backoff on both sides, so
  those numbers are comparable to Round 2 — **not** to Round 1.
- Keep `uncontended` as the tripwire: it must not move when only concurrency
  behaviour changes.
