# `reclaim` — the teaching plan for C4 (when a round runs, and who pays for it)

> **Covers**: edit 7 of `plan/reclaim_ladder.md`, C4. Planned 15 h + ≈ 4 h modeling.
> **Before**: C3 closed (2026-11-07): `Registry<P>`, `Record<P>`, `claim` / `release` / `iter`.
> **The learner types all code, tests and doc comments** unless he says "you write this".
> **Lessons continue the numbering**: C3 ended at L24, so C4 is L25–L34.
> **Source rows**: `notes/smr_coverage.md` rows 342, 347, 387, 392, 397, 459, 474, 479, 484,
> 539, 596, 601, 711, 878, 1126, 1148, 1168, 1173. RCU's `synchronize_rcu` survives here as
> `cleanup()` (row 86).

| Stage | Lessons | Output | Est |
|---|---|---|---:|
| Map | L25 | the map retold | 0.5 h |
| Model | L26–L27 | `notes/reclaim_model.md` §11, every C4 signature | 4 h |
| Core | L28 | the trigger state type; a test scheme that frees on every round | 1 h |
| Behaviour | L29–L34 | count trigger, time trigger, flattener, offload, `cleanup`, teardown + `global` + warnings | 13.5 h |
| | | **total** | **19 h** |

---

# The map

## What C4 adds

C2 built the pile and C3 the registry, but nothing yet decides **when** a round runs or
**which thread pays** for it. Both schemes (hazard pointers in C5, epoch in C6) need the
same answers, so C4 builds them once, in `Domain`. C4 frees nothing on its own: the round
itself is the scheme's (C5 writes hazard pointers' round). C4's tests use a test scheme
whose round frees everything, which is correct only because those tests hold no guards.

## Components

| Component | Kind | New? | One sentence |
|---|---|---|---|
| `Domain<R>` | struct | C1 | The handle a structure holds. Gains the trigger, `cleanup()`, teardown on drop, and `global()`. |
| scheme `R` | `Reclaimer` impl | C1 | Owns its pile and registry. Gains one method: run a round. |
| the **count** | signed atomic in the domain | C4 | Retired objects not yet reclaimed. Goes up on `retire`, down after a round. May go below zero for a moment. |
| the **threshold** | number | C4 | The count at which a round starts: `max(1000, 2 × records)`. |
| the **due time** | atomic in the domain | C4 | When a round must run even if the count is low (every 2 s). |
| the **round owner** | rule | C4 | The thread whose compare-exchange sets the count to 0 runs the round. Exactly one. |
| the **executor** | thread + channel | C4 | Optional. Runs rounds on its own thread, so a retiring thread never pays for one. |
| the **flattener** | `thread_local` queue | C4 | When a round inside a round would start, it is queued and run after, not nested. |
| `cleanup()` | method on `Domain` | C4 | Returns only when everything reclaimable right now is freed, including batches another thread's round is still holding. |
| teardown | `Drop` for the domain's inner part | C4 | No readers can exist, so free every retired object without asking the scheme. |
| `global()` | function | C4 | One process-wide default domain. |
| warnings | counters | C4 | Pile above 100,000 objects; executor more than 10 rounds behind. |

## Who holds whom

```
Domain<R> ── Arc ──> Inner
                      ├── scheme R ── pile (C2), registry (C3), round()
                      ├── count  (signed)        ┐
                      ├── due time               ├ the trigger
                      ├── executor (optional) ───┘ thread + channel
                      └── roots (C1, debug only)
thread_local: flattener queue (one per thread, shared by all domains)
```

## One lifecycle: a node retired by your Treiber stack's `pop`

1. **T2** pops node `A` and calls `domain.retire(A)`. The scheme pushes `A` on its pile; the
   domain adds 1 to the count. The count is now 1,000, the threshold.
2. **T2** tries a compare-exchange on the count, 1,000 → 0, and wins: T2 owns the round.
   **T3**, retiring at the same moment, sees the count already at 0 and does nothing.
3. **Offload on**: T2 sends "run a round" to the executor and returns at once. **Offload
   off**: T2 runs the round itself, inline.
4. The round (the scheme's code) frees `A`. `A`'s `reclaim` retires `A`'s child `B` —
   another retire, which can cross the threshold and start a round *inside* this round.
   The flattener queues it instead; the outer round runs it when it finishes.
5. While the round runs, other threads keep retiring (count +5). The round subtracts what
   it took (−1,000 + what it pushed back). The count can dip below zero for a moment; it
   is signed so that this is a small negative number, not a huge unsigned one.
6. **T4** retires 10 objects in total and then stops. The count never reaches 1,000. At
   T4's next retire after 2 s, the due time has passed, so T4 starts a round anyway.
7. A test ends with `domain.cleanup()`: it runs rounds and waits for any round still in
   progress, then returns with nothing reclaimable left.
8. The last `Domain` handle is dropped. Teardown frees every retired object without a scan,
   because no reader can exist any more.

## Glossary

- **round** — one pass of the scheme: take the pile, free what no reader protects, keep the rest.
- **count** — retired objects not yet freed, as the domain tracks them; signed.
- **threshold** — the count that starts a round.
- **due time** — the moment a round runs even below the threshold.
- **owner** (of a round) — the one thread whose compare-exchange zeroed the count.
- **inline** — the owner runs the round on its own thread.
- **offload** — the owner hands the round to the executor thread.
- **executor** — a thread that runs rounds handed to it, in order.
- **flattener** — a per-thread queue that turns a round inside a round into a round after it.
- **cleanup** — run and wait until everything reclaimable now is freed.
- **teardown** — freeing everything when the domain dies.
- **backlog** — rounds handed to the executor and not yet run.

## folly technique on this component (notes/folly_gap_analysis.md)

The folly gap analysis maps nothing new to C4 beyond the coverage rows above: the trigger,
offload, flattener, `cleanup` and teardown are all folly `hazptr_domain` mechanisms already
in the inventory. Its executor findings (sojourn time, saturation, wake throttling) are about
general task executors, not this one-purpose reclaim thread; the backlog warning (row 601)
is the one piece of that axis C4 carries.

---

# Lessons (cards are filled in as each stage starts)

| Lesson | Stage | Idea | Est |
|---|---|---|---:|
| L25 | Map | the map above, retold | 0.5 h |
| L26 | Model | when a round runs: count, threshold, owner, signed count, due time | 2.5 h |
| L27 | Model | who pays: inline vs offload, the flattener, `cleanup`, teardown, `global` | 1.5 h |
| L28 | Core | trigger state in `Inner`; the `Reclaimer` round method; a test scheme | 1 h |
| L29 | Behaviour | count trigger + owner (loom: exactly one owner); signed count test | 2 h |
| L30 | Behaviour | time trigger (a slow retirer is still freed) | 1.5 h |
| L31 | Behaviour | flattener (nested retire overflows the stack without it, passes with it) | 2 h |
| L32 | Behaviour | offload executor (same frees, different thread pays; backlog warning) | 3 h |
| L33 | Behaviour | `cleanup()` with in-progress rounds counted | 2.5 h |
| L34 | Behaviour | teardown, `global()`, list-too-large warning; journal | 2.5 h |
| | | **total** | **19 h** |
