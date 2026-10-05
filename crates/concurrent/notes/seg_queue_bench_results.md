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
- Re-run this whole file at each step of the reclamation ladder (R1 refcount,
  R2 hazard pointers, R3 epoch). The harness now has backoff on both sides, so
  those numbers are comparable to Round 2 — **not** to Round 1.
- Keep `uncontended` as the tripwire: it must not move when only concurrency
  behaviour changes.

---

## Round 3 — sealed prediction, BEFORE measuring

`pop` refactored from **claim-then-wait** to **check-then-claim**: read
`state[c]`; if not WRITTEN return `None` *without* committing `consumed`; only CAS
`consumed` once WRITTEN is observed. Removes the in-`pop` wait entirely (and with
it `SlotWaiter`/`Backoff` inside the queue).

Sealed by me before running anything:

| Scenario | prior measurement | **my prediction** |
|---|---|---|
| uncontended | 12.7–14.8 | **70 ns** |
| spsc | 12.5–13.8 | **35 ns** |
| 2P/2C | 41.3–42.5 | **45 ns** |
| 4P/4C | 46.2–66.8 (unstable) | **45 ns** |

Reasoning: `pop` is deterministic now — nothing waits inside it — so the thread
count should stop mattering; 2P/2C, 4P/4C and anything beyond should converge to
roughly the same per-item cost.

### Round 3 — measured, vs the sealed prediction

| Scenario | prior | prediction | **measured** | verdict |
|---|---|---|---|---|
| uncontended | 12.7–14.8 | 70 | **17.3** | miss 4x |
| spsc | 12.5–13.8 | 35 | **13.8** | miss 2.5x |
| 2P/2C | 41.3–42.5 | 45 | **44.5** | **HIT, <1%** |
| 4P/4C | 46.2–66.8 | 45 | **71.9** | miss 1.6x, **wrong direction** |

**The prediction's thesis was falsified.** It said: `pop` is deterministic now, so
the thread count stops mattering and every config converges to ~45. Measured:
2P/2C = 44.5 but 4P/4C = 71.9 — a 1.6x spread. Thread count still dominates.

**Why the reasoning was wrong, which is the whole value of having sealed it.**
Removing the wait from inside `pop` bounds `pop`'s *instruction count*. It does not
remove waiting from the *system* — it relocates it, and the relocation is
expensive:

| | claim-then-wait (before) | check-then-claim (after) |
|---|---|---|
| consumer meets an unwritten slot | already CAS'd, **owns** the slot → spins on **one** `state` line it already holds | **bails out entirely** → caller waits → **re-enters from the top** |
| contended lines touched per wait iteration | ~1 (already owned) | `head` + `consumed` + `claimed` + `state` = **4** |

Coherence traffic is the dominant cost term (~10.7 ns per contended line), so
converting a *local* wait into a *global retry* multiplies the dominant term. At
4P/4C with 8 threads doing it, that is worst. "Deterministic ⇒ thread count stops
mattering" is false precisely because thread count matters through coherence
traffic, and this change increased traffic per unit of waiting.

Kept anyway: `pop` is now bounded, so its tail latency no longer depends on
another thread's scheduler. That is the trade — **bounded latency bought with
worse throughput under contention** — and it is usually the right trade for a
latency-critical consumer. Stated, not hidden.

---

## Round 4 — drain policy as an explicit variable

Hypothesis under test: *the 4P/4C deficit is the caller's fault; a smarter
consumer would fix it.* So the drain policy became a swept dimension.

| Scenario | SegQueue | `Mutex<VecDeque>` |
|---|---|---|
| uncontended (no policy applies) | 16.12 | 15.53 |
| spsc **spin** | 23.42 | 26.87 |
| spsc **backoff** | 13.19 | 27.56 |
| spsc **yield** | **12.52** | 27.48 |
| 2P/2C backoff | **43.21** | 54.95 |
| 4P/4C **spin** | 83.35 | 54.27 |
| 4P/4C **backoff** | **75.39** | 54.13 |
| 4P/4C **yield** | 84.91 | 54.63 |

### Findings

8. **Policy swings SegQueue 1.9x and leaves the mutex flat.** spsc: SegQueue
   23.4 / 13.2 / 12.5 across spin/backoff/yield; the mutex sits at 26.9 / 27.6 /
   27.5. Cleanest possible confirmation of finding 6: taking the lock to discover
   emptiness *is* the mutex's backoff, so policy cannot move it. A lock-free queue
   has no built-in throttle, so **the policy IS the throttle**.

9. **Hot spinning is the WORST policy in spsc** (23.4 vs 12.5 for yield), which
   contradicts the reflex "spin for latency". With only 2 threads there is no CPU
   oversubscription — the spinner starves the producer through *coherence traffic*,
   not through CPU. Flooding the shared lines makes the producer's own atomics
   slower.

10. **No single policy wins everywhere.** Yield is best at spsc (12.5) and worst
    at 4P/4C (84.9); backoff is best at 4P/4C (75.4) and near-best at spsc (13.2).
    That spread is the empirical justification for an adaptive ladder rather than a
    fixed choice.

11. **Hypothesis FALSIFIED: the 4P/4C deficit is not the caller's fault.**
    SegQueue loses to the mutex under *all three* policies (75–85 vs ~54), and the
    mutex is rock stable (54.1–54.6) while SegQueue swings 75–85. At 8 contending
    threads on this machine the lock genuinely wins, whatever the consumer does.
    Plausible mechanism: serialisation is the *right* strategy once oversubscribed.
    The mutex converts contention into queueing (one at a time, parked); the
    lock-free structure converts contention into coherence traffic (everyone
    hammering the same lines). Making the policy an explicit variable cost one
    bench run and killed the hypothesis cheaply — which is the point.

12. **uncontended regressed slightly**: 12.7–14.8 → 16.1, now marginally behind
    the mutex (15.5). Part drift, part the extra `state.load(Acquire)` now sitting
    *before* the CAS on the success path rather than after it.

### Honest R0 verdict

- spsc with a sane policy: SegQueue wins **~−55%** (12.5 vs 27.5)
- 2P/2C: SegQueue wins **~−21%** (43.2 vs 55.0)
- 4P/4C: **mutex wins under every policy** (54 vs 75–85)
- uncontended: a tie, slightly behind

A fourth policy — park, woken by the producer — is still untested because it needs
the notification layer (`Parker`/`WaitList` → `channel`, a separate node). Do not
expect it to rescue 4P/4C: it reduces coherence traffic while waiting but adds
µs-scale wake latency per item, and finding 11 says the deficit is structural
rather than policy-shaped.

---

## Round 5 — R1 refcount: sealed prediction, BEFORE measuring

R1 adds, per guard: one `acquire_ref` (`fetch_add`, Relaxed) + one `release_ref`
(`fetch_sub`, Release) on a `ref_count` that sits on **its own cache line** and is
written by producers *and* consumers — i.e. true sharing, which padding cannot help.
Plus a `try_reclaim` walk once per `SEG_LEN` pops: a contended `swap` on the
`reclaiming` flag plus up to N `free()` calls.

Sealed before running anything:

| scenario | R0 baseline (backoff arms) | **prediction** | implied delta |
|---|---:|---:|---:|
| uncontended (1 thread) | 16.12 | **17** | +0.9 |
| spsc (2 threads) | 13.19 | **30** | +16.8 |
| 2P/2C (4 threads) | 43.21 | **53** | +9.8 |
| 4P/4C (8 threads) | 75.39 | **85** | +9.6 |

Implied shape: a large jump at spsc, then a roughly **constant** ~+10 ns at higher
thread counts — i.e. a fixed per-op tax rather than a penalty that widens with core
count. (Note: the 12.52 quoted when sealing was the spsc *yield* arm; the backoff arm,
used here for consistency with the 2P/2C and 4P/4C baselines, was 13.19.)

### Round 5 — measured, and the prediction's mechanism confirmed despite wrong numbers

| scenario | R0 | prediction | **measured** | delta vs R0 |
|---|---:|---:|---:|---:|
| uncontended | 16.12 | 17 | **10.79** | **−5.3** *(faster)* |
| spsc backoff | 13.19 | 30 | **7.53** | **−5.7** *(faster)* |
| 2P/2C backoff | 43.21 | 53 | **66.25** | **+23.0** |
| 4P/4C backoff | 75.39 | 85 | **111.60** | **+36.2** |
| 4P/4C spin | 83.35 | — | 102.75 | +19.4 |
| 4P/4C yield | 84.91 | — | 96.59 | +11.7 |
| spsc spin | 23.42 | — | 35.84 *(range 20.4–54.2!)* | +12.4 |

`mutex_vecdeque` unchanged throughout (~27 spsc, ~56 2P/2C, ~55 4P/4C) — a clean control.

**13. Adding two atomic RMWs per guard made 1–2 threads FASTER. R1 changed two
variables, not one.** R1 did not only add a refcount: it started *freeing memory*. In
R0 every 32 pushes allocated an ~896 B segment that was never reused — roughly 300k
segments ≈ **280 MB** of cold, never-recycled memory per 300 ms measurement, each one
costing allocator growth, a first-touch fault, and cold misses over ~7 cache lines. R1
recycles a handful of segments: `malloc` hits a hot free-list entry and the memory is
usually still in L1/L2. That win exceeds the ~4 ns of extra atomics.

**So R0's leak was never only a memory problem — it was silently inflating every number
we measured, and R1's figures are the first honest baseline for this structure.** The
earlier padding conclusions are unaffected, since those compared R0 variants against
each other under the same leak.

Consequence: **the refcount's isolated cost is not readable from this comparison.** Two
variables moved. To isolate it you would need an R0 run with a bounded workload (so the
leak cannot grow) or an allocation-count instrument.

**14. The shape question was still answered, and understated.** Deltas in thread-count
order: **−5.3, −5.7, +23.0, +36.2** — monotonically widening. That is the true-sharing
fingerprint for a line written by every thread on every operation, and every one of
those figures already has the allocator win pulling it *down*, so the raw contention
grows more steeply than shown.

**15. The verdict flipped at 2P/2C.**

| | R0 | R1 |
|---|---|---|
| 2P/2C vs mutex | SegQueue **won** 43 vs 55 | **loses** 66 vs 56 |
| 4P/4C vs mutex | lost 75 vs 54 | **loses worse** 112 vs 55 |

**16. R1 widened the policy spread enormously.** `spsc_spin` became wildly unstable —
median 35.8, range **20.4–54.2** — against `spsc_backoff`'s tight 7.53. A shared
contended counter amplifies whatever the drain policy does wrong.

### What R2/R3 should recover, and what they will not

Of the +23 / +36 at 4 and 8 threads, the parts that are **R1-specific by construction**:

- the shared `ref_count` line, written by every thread on every op → R2 and R3 have no
  shared counter at all; the signal moves to per-thread state
- the `reclaiming` flag serialisation → epoch uses per-thread garbage bags, no global flag

The part that is **permanent and keeps helping**: freeing memory at all, worth ~−5 ns.

So R3 should land near R1's 1–2 thread figures and well below its 4–8 thread ones. What
is *not* promised is beating a parking mutex at 8 threads on an 8-thread laptop with
heterogeneous P/E cores — that configuration favours a lock that converts contention
into queueing. The number that would actually justify a lock-free queue is **p99
latency**, which this harness has never measured.

---

## Round 6 — the right shape, against the right opponents

**Two methodology errors, both caught by the user:** every previous round benchmarked
**NPNC** (1P1C, 2P2C, 4P4C), which is a shape *neither real consumer of this queue
produces*; and the only opponent was `Mutex<VecDeque>`, which is not what anyone would
actually reach for.

Both real uses are **N producers → exactly 1 consumer**, unbounded because the backlog
is a function of something outside their control:

- `wal/group_commit.rs` — N request threads into the sole `fsync` caller. A bounded ring
  means request threads block on the disk.
- `runtime/cross_shard.rs` — N shards into one shard's inbox. A bounded ring means one
  stalled shard blocks the others.

NP1C also has a materially different contention profile: `consumed`, the `head` CAS and
half the refcount traffic are **uncontended** with a single consumer. Benchmarking NPNC
was measuring pressure neither use case creates.

Opponents now are the ones actually specialised for MPSC:

| config | **`seg_queue` (this crate, R1)** | **crossbeam `SegQueue`** | `Mutex<VecDeque>` | `std::sync::mpsc` |
|---|---:|---:|---:|---:|
| 1P1C | **7.66** | 13.78 | 28.23 | 14.01 |
| 2P1C | 77.85 | **14.43** | 76.29 | 40.44 |
| 4P1C | 105.91 | **15.40** | 48.28 | 78.10 |
| 8P1C | 104.94 | **16.74** | 39.20 | 169.32 |
| 2P2C *(control — unused shape)* | 62.95 | **14.08** | 53.56 | — |
| 4P4C *(control — unused shape)* | 103.46 | **18.61** | 55.05 | — |

### Findings

**17. The algorithm is decisively worth building — and the proof is crossbeam, not us.**
Crossbeam's `SegQueue` is essentially **flat** from 1 to 8 producers (13.8 → 16.7), and at
8 producers it is **2.3× faster than a mutex** and **10× faster than `std::sync::mpsc`**.
So "why not just use a mutex" has a measured answer at the shape that matters. What was
in doubt was never the structure; it was this crate's implementation of it.

**18. `seg_queue` is 6–7× off the reference, and it is a cliff, not a slope.**

```
seg_queue:  7.7  →  77.8  →  105.9  →  104.9      (1 → 2 → 4 → 8 producers)
crossbeam: 13.8  →  14.4  →   15.4  →   16.7
```

A **10× jump from one producer to two.** Not "contention exists" — crossbeam has
contended counters too and barely moves.

**19. The prime suspect is the shared refcount, which R2/R3 delete by construction.**
Crossbeam's SegQueue has **no refcount at all**; it reclaims via per-slot
WRITE/READ/DESTROY bits. `seg_queue` adds a counter written by every thread on every operation.
At 2P1C that is 3 threads × 2 RMWs per item on one line ≈ 4 contended transfers ≈ 43 ns,
plus contended `claimed` — most of the observed +70.

**20. The strongest evidence the core mechanics are sound: at 1P1C `seg_queue` BEATS crossbeam**
(7.66 vs 13.78). push/pop/boundary is not the problem. The problem is specifically the
reclamation scheme bolted on top — the part that is temporary by design.

**21. `std::sync::mpsc` scales badly**: 14 → 169 ns from 1 to 8 producers, *worse than a
mutex* at 8. The standard library's purpose-built MPSC loses to a lock under load.

**22. The mutex gets FASTER with more producers** (76 → 48 → 39). Almost certainly
because at low producer counts the single consumer starves and burns backoff on an empty
queue, while more producers keep it fed. Which means the low-N figures for *every* arm
are partly measuring consumer starvation rather than the queue — a confound to remember
when reading 1P1C and 2P1C.

### Not measured, and deliberately deferred

Every number here is a **median throughput**. The argument that actually motivates
lock-free is **p99 latency** — a mutex's tail includes "the holder was descheduled while
holding the lock", which is unbounded. That was going to be the deciding measurement if
throughput had been inconclusive; finding 17 settled it on throughput alone, so p99 waits
for `latency-lab`/HdrHistogram. Note also (finding 10i) that the reclamation spike is
invisible to a median by construction.
