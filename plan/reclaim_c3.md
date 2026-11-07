# `reclaim` — the teaching plan for C3 (the registry)

> **Covers**: edit 6 of `plan/reclaim_ladder.md`, C3. Planned 4 h + ≈ 2 h modeling (was 4.5 h; see the note under the table).
> **Before**: C2 is closed (2026-11-05): `RetireLink`, `RetiredList`, `ShardedList`, `Chain`.
> **The learner types all code, tests and doc comments** unless he says "you write this".
> **Lessons continue the C2 numbering**: C2 ended at L19, so C3 is L20–L24.

| Stage | Lessons | Output | Est |
|---|---|---|---:|
| Map | L20 | the map retold in your own words | 0.5 h |
| Model | L21 | `notes/reclaim_model.md` section on the registry, ending in every C3 signature | 1.5 h |
| Core | L22 | `registry.rs`: `Record<P>`, `Registry<P>`, `Drop`, layout tests | 1 h |
| Behaviour | L23–L24 | `claim` / `release` / `iter`, loom, the 64-thread reuse test | 3 h |
| | | **total** | **6 h** |

*2026-11-06: thread-exit hand-back moved to C6 (L21 Q3), so L24 went 1.5 h → 1 h and C6
carries the 0.5 h. The `reclaim` total does not change.*

---

# The map

## What C3 adds, and why it exists at all

A round (C2) can free an object only if no reader is using it. A reader says "I am using
`A`" by writing `A`'s address somewhere. The **registry** is the list of those places, so a
round can find every one of them. C3 builds that list. It does not decide what is written
in it — that is C5 (hazard pointers) and C6 (epoch).

## Components

| Component | Kind | New? | One sentence |
|---|---|---|---|
| `Record<P>` | struct | C3 | One place a reader writes to. Holds a `payload: P`, an `in_use` flag, and `next` (the record after it in the list). Padded to its own cache line. |
| `Registry<P>` | struct | C3 | A list of `Record<P>`s. Records are added, never removed, and freed only when the registry is dropped. |
| claim / release | functions on `Registry` | C3 | `claim` gives a thread a record nobody else holds (reusing a free one first). `release` clears the payload and marks the record free. |
| `P` (the payload) | type parameter | C3 | What a reader writes. Hazard pointers: one address (`AtomicPtr<()>`). Epoch: a number (`AtomicU64`). |
| `Domain<R>` | struct | C1 | Unchanged. Holds the scheme `R`. |
| scheme `R` | `Reclaimer` impl | C5/C6 | Will own one `Registry<P>` and one `ShardedList`. `Leak` needs neither. |
| guard | `R::Guard` | C1 | For hazard pointers, holds one claimed record and writes into it. |
| `ShardedList` | struct | C2 | Unchanged. The pile of retired objects. |

## Who holds whom

```
Domain<R> ── Arc ──> scheme R
                       ├── ShardedList          (C2: retired objects)
                       └── Registry<P>          (C3)
                             head ──> Record ──> Record ──> Record ──> null
                                       ▲ in_use    in_use     free
                                       │
                         guard on T1 ──┘  (holds the claim, writes payload)
```

## One lifecycle: a record, followed through your Treiber stack's `pop`

Assume a hazard-pointer scheme exists (C5). `P = AtomicPtr<()>`.

1. **T1** calls `domain.guard()`. The scheme calls `registry.claim()`. No record is free, so
   `claim` allocates record **R1** and adds it at the head of the list. `R1.in_use = true`.
2. **T1** calls `guard.protect(&head_root)` and gets node **A**. The guard writes `A`'s
   address into `R1.payload`.
3. **T2** pops `A` and calls `domain.retire(A)`. `A` goes on the pile (C2).
4. **T2** runs a round: `take_all` the pile, then walk the registry and read every payload:
   `{A}`. `A` is protected, so it goes back on the pile. Everything else is freed with
   `free_all`.
5. **T1** drops its guard. `release(R1)`: payload set to null, *then* `in_use = false`. `R1`
   stays in the list.
6. **T3** calls `domain.guard()`. `claim` finds `R1` free and takes it. No allocation.
7. The domain is dropped. The registry is dropped with it and frees `R1`. This is the only
   moment a record is ever freed.

## Glossary

- **record** — one `Record<P>`: the place one reader writes what it is using.
- **payload** — the value in a record that a reader writes and a round reads.
- **claim** — take a free record for yourself (or make a new one).
- **release** — clear the payload and mark the record free again.
- **registry** — the list of all records of one scheme in one domain.
- **immortal** — a record is never freed while the registry exists, only reused.
- **scan** — the part of a round that walks the registry and reads every payload.
- **stale payload** — a payload left behind by a reader that is gone.

## folly technique on this component (notes/folly_gap_analysis.md)

A registry that adds one record per thread and never frees makes every scan cost
O(records ever made). Reuse (step 6) bounds it at the most readers that were ever active at
the same time. folly also bounds hoarding with thread-cache decay, and `AccessSpreader`
would key records by CPU instead of thread, bounding the scan at the CPU count. C3 builds
reuse. Decay arrives with the thread cache (deferred); CPU keying is recorded, not built.

---

# Decided in L21 (2026-11-06)

- **Q1 — records are immortal.** Freeing a record while a scan reads it is a use-after-free;
  making records safe to free would need a reclamation scheme for the reclamation scheme.
- **Q2 — release clears the payload, then marks the record free.** The other order lets the
  old owner wipe the next owner's payload; a round then frees what the new owner reads.
- **Q3 — the registry does not decide who holds a claim.** Hazard pointers: a guard, from
  `domain.guard()` to the guard's drop. Epoch: a thread, from its first `pin()` to thread
  exit, because a claim per pin costs a shared compare-exchange (≈ 125 ns at 8 threads)
  where a pin should cost one store to its own record. So **thread-exit hand-back moves to
  C6**, where its only user is. C3 keeps the 64-thread reuse test, with each thread
  claiming and releasing before it exits.
- **Q4 — a linked list first, the array only if C5 measures a reason.** The scan runs once
  per round, not once per retired object. List: H dependent misses (≈ 100 ns each); array:
  the same misses, overlapped. Prediction (to check in C5's R × H scan bench): H = 64 →
  list ≈ 6.4 µs per round ≈ 6.4 ns per retired object at 1,000 per round, above a chained
  push (≈ 1 ns), so the array likely pays there; H = 8 → ≈ 0.8 ns, it does not. Switch
  trigger: scan cost per retired object above push cost at the H the clients really reach.

# Lessons

### L20 — The map  ✅ 2026-11-06
- **Where on the map**: all of it
- **Input**: C2 done; `domain.rs`, `retire.rs`, `tests/stack.rs`
- **Teacher presents**: the map above, in the session
- **Output**: the lifecycle story retold in your own words, steps 1–7
- **Gate**: you can say which component acts at each step without looking
- **Est**: 0.5 h

### L21 — Why the registry is shaped like this  ✅ 2026-11-06 (Output written by the teacher, on request)
- **Where on the map**: `Record<P>`, `Registry<P>`, claim / release
- **Input**: L20; the C2 pile (`push` with a compare-exchange); `CachePadded` from L18
- **Teacher presents**: a sketch of the two types and the three functions (marked sketch)
- **Questions**:
  1. Why is a record never freed while the registry lives? (Exhibit: free `R1` at step 5
     while T2 is at step 4.)
  2. In `release`, why clear the payload *before* `in_use = false`? (Exhibit: the other order,
     with T3 claiming in between.)
  3. Who holds a claim: a guard or a thread? Answer it once for hazard pointers and once for
     epoch. What does that say about where thread exit belongs?
  4. Linked list or grow-only array? What does the scan cost in each, and what does each
     cost to grow?
  5. Two threads call `claim` on the same free record. What makes only one of them win?
  6. Why padded? Which two writers would share a line without it?
- **Output**: a registry section in `notes/reclaim_model.md`, with the exhibits and every
  C3 signature
- **Gate**: you can state both rules (immortal; clear before free) with the trace that breaks each
- **Est**: 1.5 h

### L22 — The types  ✅ 2026-11-06
- **Where on the map**: `Record<P>`, `Registry<P>`
- **Input**: L21 signatures
- **Output**: `crates/reclaim/src/registry.rs` with `Payload` (+ its impl for `AtomicPtr<()>`),
  both types, `Registry::new`, `push_new` (allocate a record and add it at `head` — the half
  of `claim` that L23 reuses), `Drop` that frees every record, and tests: alignment is 128 on aarch64; a new registry is empty;
  Miri shows `Drop` leaks nothing
- **Gate**: you can say what `Drop` may assume about other threads, and why
- **Est**: 1 h

### L23 — Claim, release, scan  ✅ 2026-11-07
- **Where on the map**: claim / release / iter
- **Input**: L22
- **Output**: `claim`, `release`, an iterator over records; single-thread tests (claim twice
  gives two records; release then claim gives the same record; release clears the
  payload); a loom model with two claimers that asserts they never get the same record,
  and its negative control (claim with load-then-store instead of compare-exchange) going
  red; a loom model where a payload written before claiming is seen by a scanning thread
- **Gate**: you can name the ordering on each atomic access and the pair it belongs to
- **Est**: 2 h

### L24 — Reuse across threads  ✅ 2026-11-07
- **Where on the map**: claim / release, the owner from L21 Q3
- **Input**: L23
- **Output**: the test from the ladder: 64 threads run one after another, each claims,
  releases and exits, and the registry ends with **1** record, not 64 (negative control:
  `claim` that never reuses → 64); a journal entry. Thread-exit hand-back is C6 (L21 Q3).
- **Gate**: you can say what bounds the scan cost, and what does not
- **Est**: 1 h
