# `reclaim` + SegQueue — implementation plan

> **Component**: SegQueue (unbounded lock-free MPMC) + `reclaim` (safe memory reclamation)
> **Crate(s) touched**: **[NEW]** `crates/reclaim/`; `concurrent` → `SegQueue` becomes generic over the scheme
> **Bar**: (c) — both are Layer-1 substrate, so the two-track rule applies: a sealed numeric prediction *before* every measurement, then the measured number, then reconcile the gap
> **Mirror target**: folly `hazptr` (Domain / holder / retire), `crossbeam-epoch` (Collector / LocalHandle / Guard / three-epoch cycle), `haphazard`, and `crossbeam_queue::SegQueue` (block cursor + per-slot DESTROY bit)
> **Feeds into**: `concurrent` skiplist · `bufpool` page reclaim · P4 price levels · P5 ledger + cross-shard queues
> **Current position**: refcount built, proven unsound, and benched. Reclamation is the open problem.
> **Remaining**: **≈ 164 h** across 6 steps — was 116 h before the inventory audits
> (+35.5 h from the not-ported audit, +12.5 h from the 192-row coverage sweep;
> full disposition of every inventory row: `notes/smr_coverage.md`)
> **Not counted here**: the blog. It is an output of finished work, not part of the build.
> **Source research**: `notes/smr_inventory.md` (192 mechanisms from folly / crossbeam-epoch / haphazard / the literature) · `notes/folly_gap_analysis.md`

---

## Where we are

Spent ≈ **15 h** on implementation (approximate — from `notes/seg_queue_build_log.md`):

| Done | What it bought |
|---|---|
| Queue on leak | push / pop, ticket cursors, boundary CAS, 9 std tests, 5 loom models, Miri clean with `-Zmiri-ignore-leaks`, bench vs `Mutex<VecDeque>` |
| `pop` check-then-claim | removed the unbounded internal spin, so pop's worst case no longer depends on another thread's scheduler |
| Refcount, **and its impossibility proof** | a counter inside the object cannot be made sound — the announcement *is* the dangerous access. Also produced three things that **survive into the final design**: the RAII guard, the three free conditions, and the ordered trailing `reclaim` cursor |
| Benches, six rounds | the false-sharing pair is `Segment::{claimed, consumed}` not head/tail · padding is a trade (spsc −34 %, uncontended +74 %) · drain policy dominated everything (−56 %) · the real shape is N→1, not N→N · the real opponents are `std::sync::mpsc` and `crossbeam`, not a mutex |

In the tree today: 9 passing tests, 5 loom models, the refcount and the ordered walk.
**Step 2 deletes the refcount** — its value is the proof, which is already banked.

---

## The six steps

| # | Step | Deliverable | Wall that forces the next step | Est |
|---|---|---|---|---:|
| **1** | **Define the problem** | the `Reclaim` trait and its safety documentation. No scheme, no registry. All signature questions are settled below — this step writes them down and proves they compile. | A contract with no implementor and no client is unfalsifiable | 6 h |
| **2** | **Integrate with SegQueue** | `Leak` (the trivial implementor) · `SegQueue<T, R: Reclaim>` · the protect-source restructure · a Treiber stack as a second client | Integrable and not SegQueue-shaped — but nothing yet reclaims anything | 14 h |
| **3** | **Implement the schemes** | the shared registry + `Domain` (28.5 h) · hazard pointers (18 h) · **link counting** (10 h) · **cohorts** (6 h) · epoch **including its native typed API** (38 h) · a **Harris linked set** as that API's acceptance test (6 h) | Two schemes exist behind one interface and have never been compared | 111 h |
| **4** | **Bench the reclamation axis** | `Leak` / `Hazard` / `Epoch` / `Mutex<VecDeque>` / real `crossbeam`, identical queue code — **throughput *and* the retire-call latency distribution** | Reclamation is now isolated. Any remaining gap to crossbeam is **layout** or **crossbeam's own scheme** — and neither has been built | 11 h |
| **5** | **The crossbeam approach** | layout A, a global index, reclamation held fixed (7 h) · crossbeam-exact: block cursor + per-slot `WRITE`/`READ`/`DESTROY` bits (11 h) | Everything is built; nothing has been compared head to head | 18 h |
| **6** | **Final bench — which wins, and why** | the full matrix, one variable per comparison, and the written argument for the winner | — endpoint | 4 h |

---

## The full sequence — 19 edits, in dependency order

The one table to work from — **19 edits**. `C*` are `reclaim`-crate edits, `Q*` are `seg_queue.rs` edits;
they interleave because `Q2` cannot land before the traits exist. Detail for each is in the
two sections after this one.

| Order | Edit | Depends on | Est | Cumulative |
|---:|---|---|---:|---:|
| 1 | **C1** traits + `Leak`, the safety docs, **filter / empty state / swap** | — | 6 h | 6 |
| 2 | **Q1** loss-detecting loom model, against today's code | — | 1.5 h | 7.5 |
| 3 | **Q2** trait integration under `Leak`; refcount deleted | C1 | 4 h | 11.5 |
| 4 | **Q3** protect-source restructure → root set `{head, tail}` | Q1, Q2 | 2.5 h | 14 |
| 5 | **C2** retired list — sharded, batch push, **double-retire detection** | C1 | 9 h | 23 |
| 6 | **C3** registry — immortal **padded** per-thread records | C2 | 4.5 h | 27.5 |
| 7 | **C4** `Domain<R>` — triggers, executor, flattener, **`cleanup`, teardown** | C3 | 15 h | 42.5 |
| 8 | **C5** hazard pointers — hashed guarded set, fence interface | C4 | 18.5 h | 61 |
| 9 | **C10** link counting — protect a child through its parent | C5 | 10 h | 71 |
| 10 | **C11** cohorts — per-structure retired lists, teardown | C4, C10 | 6 h | 77 |
| 11 | **C8** Treiber stack — second client | C5 | 5 h | 82 |
| 12 | **Q4** bench arms wired | Q3, C5 | 1 h | 83 |
| 13 | **C6** epoch core — two generations, then three, **+ `repin`/`flush`** | C4 | 23 h | 106 |
| 14 | **C7** epoch typed API — **+ `fetch_or` tag ops** | C6 | 19 h | 125 |
| 15 | **C9** Harris linked set — third client, and C10's real test | C7, C10 | 6 h | 131 |
| 16 | **step 4** bench the reclamation axis | C5, C6, Q4 | 11 h | 142 |
| 17 | **5a** layout A — global index | step 4 | 7 h | 149 |
| 18 | **5b** crossbeam-exact — DESTROY bit | 5a | 11 h | 160 |
| 19 | **step 6** final bench: which wins, and why | 5b | 4 h | **164** |

Three things this ordering buys that a different one would not:

- **C5 before C6.** Hazard pointers are the simpler scheme and the first real user of C3's
  registry and C4's threshold. If the registry's payload generics are wrong, discovering it
  inside a 14 h step beats discovering it inside a 38 h one.
- **C8 before C6.** The stack has to run against two schemes to prove anything about the
  trait, and `Leak` + `Hazard` is already two. So epoch gets written against a trait that
  already has two implementors and two clients, instead of being the thing that discovers the
  trait is wrong.
- **Q1 at position 2, not later.** Q3 rests on an argument, not a measurement; Q1 is what lets
  loom refute it before eleven more edits are stacked on top.

Optional early read: after edit 10 you can bench `Leak` vs `Hazard` alone. It is not step 4 —
epoch is missing — but it catches a hazard-pointer disaster 40 h before step 4 would.

---

## The `reclaim` crate, in edit order

Labelled **C1–C9** so nothing collides with the queue-side Q-edits or the old reclamation
labels. Each is its own commit with its own green run.

```
crates/reclaim/
  src/lib.rs        Reclaim · Guard · Retire · RetireLink        C1
  src/sync.rs       the loom shim (mirrors seg_queue's mod sync)  C1
  src/leak.rs       Leak                                          C1
  src/retire.rs     retired list: 8 shards, batch push, Retired   C2
  src/registry.rs   immortal per-thread records, claim/release    C3
  src/domain.rs     Domain<R> (Arc handle) + global()             C4
  src/trigger.rs    count trigger + 2 s time trigger, signed count C4
  src/reclaimer.rs  offload thread + inline recursion flattener   C4
  src/fence.rs      light/heavy asymmetric fence interface        C5
  src/hazard.rs     announce addresses                            C5
  src/guarded.rs    hashed guarded set, O(R + H) matching         C5
  src/linked.rs     link counting: packed {link|ref}, unlink      C10
  src/cohort.rs     per-structure retired list, teardown          C11
  src/epoch/
    mod.rs          the Reclaim impl                              C6
    epoch.rs        AtomicEpoch, the 3-generation rule            C6
    local.rs        per-thread pin counter + local epoch          C6
    bag.rs          garbage bags, Deferred                        C6
    atomic.rs       Atomic/Owned/Shared/Pointable + tagging       C7
crates/concurrent/
  src/stack.rs      Treiber stack  — 2nd client, on the trait     C8
  src/list.rs       Harris set     — 3rd client, on the typed API C9
```

Clients live in `concurrent`, not `reclaim`: `reclaim` stays pure substrate, and the Harris
set is the skiplist's precursor so it belongs next to it.

| # | Edit | Acceptance test | Est |
|---|---|---|---:|
| **C1** | Crate skeleton and **the three traits**, plus `Leak`. No concurrency anywhere. | `cargo build` · a doc-test using `Leak` · the safety docs from step 1 are written here, not later | 4 h |
| **C2** | **Intrusive retired list, sharded.** `RetireLink { next: AtomicPtr<()> }` on the object; `Retired` is the type-erased `(ptr, reclaim_fn)` pair that `Retire::reclaim` monomorphises into. **8 shard heads** chosen by hashing the object address with the low 8 bits discarded (allocator alignment makes them non-random), and a **batch push** that accumulates ~20 objects locally and pushes the run with one CAS plus one count add. | single-threaded: push N objects, drain, assert each object's **custom** `reclaim()` ran exactly once — a pooled object must go back to the pool, not through `Box` · shard distribution is even across a realistic allocation trace · **a contended retire bench showing the shard win**, because one list head is a CAS hot spot and that is the only reason the shards exist | 8 h |
| **C3** | **The registry.** Append-only immortal list of per-thread records, `claim`/`release`, hand-back on thread exit. **Generic over the record payload**, because C5 stores addresses in it and C6 stores an epoch. | a declaration is visible from another thread · release clears stale payload (a leftover would be a permanent false positive for whoever reuses the record) · 64 sequential threads do **not** create 64 records | 4 h |
| **C4** | **`Domain<R>`** — `Arc` handle, `global()`, owns the registry + retired list + reclamation threshold. Plus **two triggers, not one** — a count trigger and a **time trigger** — and the two things that decide *who pays* for a reclamation round: an **offload reclaimer** (a thread plus a channel, so the round is not charged to whichever thread happened to cross the threshold) and, for the inline fallback, a **recursion flattener** — a `thread_local` queue, because reclaiming objects can retire more objects, cross the threshold again, and recurse until the stack is gone. | two independent domains cannot see each other's records · a dropped domain asserts its retired list is empty · a loom model builds a `Domain` inside `loom::model` and never touches `global()` · **a nested-retire test that overflows the stack without the flattener and passes with it** · offload on and off produce identical reclamation, different latency owners · **a slow retirer — below the count threshold forever — still has its garbage freed, which only the time trigger delivers** · **the retired count is signed**: a round zeroes it then subtracts what it reclaimed while others add, so it legitimately goes negative, and an unsigned counter would wrap into a huge value and trigger runaway reclamation (a test drives the count negative) | 10.5 h |
| **C5** | **Hazard pointers.** In four landable pieces: announce + validate (loom first), then the **hashed guarded set**, then the scan, then batching against the threshold. The guarded set is not an optimisation — step 1's reason #2 for a scheme-owned retire list *is* O(R + H), and a linear scan per retired object delivers O(R × H), the complexity that argument rejected. Also the **light/heavy fence interface**: `light()`/`heavy()` as named operations, with `light() = full fence` on every platform without `membarrier`. Same codegen as a bare fence here; it names the seam and lets a Linux run show the real win. | loom on the announce/validate handoff — store/load, so loom is trustworthy here · **both negative controls**: always-protected stalls reclamation, never-protected produces a Miri UAF in a client · Miri clean · **a scan-cost measurement at R ∈ {100, 1000} × H ∈ {8, 64} showing O(R + H), not O(R × H)** | 18 h |
| **C6** | **Epoch core.** `AtomicEpoch` and the generation rule, then re-entrant `pin`/`unpin` with the pin counter, then bags, then `try_advance` + `collect`. | **build it with TWO generations first and let loom produce the counterexample**, then go to three — the whole point of the step is why two is not enough · loom on pin/unpin nesting · Miri | 20 h |
| **C7** | **Epoch's typed API.** `Atomic<T>` / `Owned<T>` / `Shared<'g, T>` / `Pointable`, pointer tagging, the full `compare_exchange` family. | `trybuild` compile-fail tests (already a dev-dependency) proving `Shared<'g, T>` cannot outlive its guard · tag round-trips at every alignment · `Pointable` for `[MaybeUninit<T>]` — **not** for the bags, which are `[Deferred; 64]`, a fixed array needing no `Pointable`; the real consumer is a variable-length allocation, i.e. the skiplist's tower, so this is the one part of C7 with no consumer until then | 18 h |
| **C8** | **Treiber stack** in `concurrent`, on the trait. Second *client*. | its own loom models · runs against `Leak`, `Hazard` and `Epoch` unchanged — that is the test of the trait, not of the stack | 5 h |
| **C9** | **Harris linked set** in `concurrent`, on the typed API. Third client, and C10's real acceptance test — Harris is the *uncertain removal* case by construction. | loom with **two guards held at once** (hand-over-hand on `pred`/`curr`) · a node marked but not yet physically unlinked must **not** be retired — the deferred-unlink case · runs against link counting as well as the root-set argument · Miri | 6 h |
| **C10** | **Link counting.** The mechanism that closes HP's actual limitation: *a hazard pointer on `A` does not protect `A->next`*. Two counters packed in one `AtomicU64` — link count (inbound from mutable paths) and ref count (inbound from immutable paths) — so "downgrade a mutable link to an immutable ref" is one CAS rather than a two-step window where the object looks unreferenced. `retire()` for certain removal, `unlink()` for uncertain. | a child reached through a protected parent is safe with **zero added reader cost** — that is the whole claim, so the reader path must be unchanged by a bench · **the `for_each_link` aliasing rule has its own Miri test**: each child pointer must be read *before* `f` is invoked on it and must never be touched after, because `f` may already have freed it · a chain of 10k immutable nodes frees in one pass, not 10k threshold rounds | 10 h |
| **C11** | **Cohorts.** A per-structure retired list instead of objects spread across per-thread lists and mixed with unrelated garbage. Carries the `active_` flag and `shutdown_and_reclaim()`. | objects retired *during* teardown are reclaimed, not pushed onto a list nobody will drain — the postcondition is `!active() && list.is_empty()` · a few stragglers must not delay reclamation of a large run of link-counted objects, which is the locality claim and needs C10 to be measurable | 6 h |

**C1 = step 1. C2–C4 = step 3's registry + `Domain` (10 h). C5 = hazard (14 h). C6 + C7 =
epoch (38 h). C8 = step 2's second client (5 h). C9 = step 3's Harris set (6 h).**

### Ordering constraints that are real

- **C2 before C3.** The registry's thread-exit path has to hand back a record *and* leave its
  payload clean; writing that before the retired list exists means guessing at what "clean"
  means.
- **C5 before C6.** Hazard pointers are the simpler scheme and they exercise C3's registry and
  C4's threshold first. If the registry's payload generics are wrong, finding out during a
  14 h step beats finding out during a 38 h one.
- **C8 after C5, before C6.** The stack must run against two schemes to prove the trait, and
  `Leak` + `Hazard` is enough for that. Doing it before epoch means epoch is written against a
  trait that already has two clients and two implementors.
- **C7 after C6.** The typed API's `Pointable` impl is what the bags allocate through, so the
  bags have to exist to know what it needs.
- **C9 after C10.** Harris is *uncertain removal* by construction, which is link counting's
  named case, so C9 is C10's acceptance test rather than an unrelated client. C9 is still the
  only client of C7 and so still cannot be written before it.
- **C10 after C5.** Link counting is an HP mechanism; it needs a working scan to be testable.
- **C11 after C10.** Cohorts' locality claim is *about* link-counted objects — "a few missing
  objects delaying the reclamation of large numbers of link-counted objects". Without C10 the
  claim cannot be measured, only asserted.

### What is ported, and what is not

**This section replaces an earlier one that was wrong.** It claimed the omissions were rows
`notes/smr_inventory.md` had marked `optional` or `skip`. Six of its seven items are marked
**IMPORTANT**, one mechanism it omitted is marked **CORE** and was not listed at all, and two
items it disclaimed are in fact built by this plan under different names. The audit below is
the real record. The test applied is not *is it affordable* but **does the forcing pressure
exist in this design** — and the hour cost of a yes is not an input.

| Mechanism | Inventory | Decision | Why |
|---|---|---|---|
| hashed guarded set (`F14FastSet`) | **CORE** 352 | **port**, C5 | Was absent from the plan *and* from the old not-ported list. Step 1's reason #2 for a scheme-owned retire list is O(R + H); a linear scan is O(R × H), the complexity that argument rejected. haphazard has this bug and flags it on itself (1116: `BTreeSet`, so every pass allocates) |
| **time trigger** (`kSyncTimePeriod` = 2 s) | **IMPORTANT** 459 | **port**, C4 | Second mechanism absent from the plan *and* from the old list, found the same way as the guarded set — by asking what batching leaves stuck rather than by reading the list of claimed omissions. The count threshold bounds garbage by **count**; a slow retirer sits below it forever and "in a queue the segments are never freed". Step 1's reason #1 claims the threshold machinery bounds exactly this, and the count half alone does not |
| sharded retired lists | **IMPORTANT** 454 | **port**, C2 | One list head is a CAS hot spot under many-thread retire; 8 shards cut it ~8×. Directly visible in step 4's 8-producer arm |
| offload executor | **IMPORTANT** 479 | **port**, C4 | "Charging it to a random unlucky retire-er produces a huge tail-latency spike." Re-derived from first principles in design discussion before the row was re-read — which is what exposed the old section |
| inline recursion flattener | OPTIONAL 596 | **port**, C4 | Near-mandatory, not optional: it exists *because* reclamation runs inline. Reclaim → retire → cross threshold → recurse → stack overflow. SegQueue triggers it, since freeing a segment drops its `T`s |
| asymmetric fence **interface** | **IMPORTANT** 444, 464 | **port the interface**, C5; skip the impl | "The biggest single performance idea in folly's hazptr." The old reason — unavailable on macOS/aarch64 — is true of the `membarrier` *implementation*, not of the light/heavy *interface*. Porting the interface costs nothing here and makes the win measurable on Linux |
| cohort batch push (`kThreshold = 20`) | **IMPORTANT** 509 | **port**, C2 | One shared-list CAS and one count add per 20 retires instead of per retire. Independent of the rest of cohorts |
| cohorts: locality, `active_`, teardown | **IMPORTANT** 504, 514 | **port**, C11 | Row 504 names "the UnboundedQueue Segment case" — literally this structure. Teardown is a correctness path: objects retired *while* a cohort shuts down must be reclaimed, not listed for a drainer that will never come |
| **link counting** (`hazptr_obj_linked`) | **IMPORTANT** 519, 529 | **port**, C10 | Reversed. This is not an optimisation — it is the mechanism that lifts hazard pointers' headline limitation, "a hazard pointer on `A` does not protect `A->next`", which is why HP applies to 3 of 18 structures in Singh's survey. Q3's root-set restructure substitutes for it **only where a root set exists**. `{head, tail}` is one for SegQueue; a skiplist traverses node pointers at every level and may have none. Deferring it would park the technique exactly where the committed roadmap needs it |
| thread cache (`hazptr_tc`) | **IMPORTANT** 489, 494 | **defer — with a measurement, not a guess** | The only defer whose reason is technical rather than budgetary. Its pressure is "every holder construction is a CAS on the shared `avail_` list", i.e. acquisition is hot. **Shape B makes acquisition ~1 per operation** (see *`protect` retargets a held guard*), so the pressure does not exist in this design. Step 4 measures guard-acquisition cost directly; if it is invisible there, this becomes a decided skip with evidence behind it, and if it is not, port it |
| `hazptr_local<M>` | OPTIONAL 616 | **skip** | The row argues this side: ~2 ns versus ~5 ns, bought with "it is unsafe for the current thread to construct any other holder-type objects while the current instance exists." Non-composability for 3 ns |
| tagged-list locking, `cleanup_cohort_tag` | OPTIONAL 591, 621, 626 | **conditional** | Only needed if synchronous per-tag cleanup is ported, which nothing currently asks for. Genuinely optional, and the inventory agrees |
| crossbeam `sync::list` | **IMPORTANT** 858 | **already built — it is C3** | The old section disclaimed it. "Intrusive lock-free participant registry" is C3's one-line description |
| crossbeam `sync::queue` | **IMPORTANT** 873 | **already built** | C4's retired list plus C6's bags. Also disclaimed in error |
| `load_consume` | OPTIONAL 920 | **skip** | Real on aarch64, where `Acquire` costs a `dmb ishld` and a dependent load is free — so the better reason than "optional" is that it is **disabled under Miri, loom and TSan**. An optimisation none of this project's verification tools can see is one this project cannot justify |
| `crossbeam_sanitize`, `no_std` | SKIP 942, 947 | **skip** | No algorithmic content |
| folly `mprotect` membarrier fallback | SKIP 658 | **skip** | TLB-shootdown trick for platforms without `membarrier`; the fence interface's `light() = full fence` fallback covers the same ground honestly |

**Net: +35.5 h** — C2 3 → 8, C4 3 → 10.5, C5 14 → 18, C10 +10, C11 +6, step 4 8 → 11.
One defer (the thread cache, ~5 h) with a measured trigger, and one conditional (tagged-list
locking, ~4 h). Nothing is parked for want of hours.

### The sweep that found what this table could not

The audit above started from the list of *claimed* omissions, so it could never surface a
mechanism nobody had claimed to omit — and two of those turned up anyway (the hashed guarded
set, by checking the plan's own O(R + H) argument; the time trigger, by asking what batching
leaves stuck). The terminating version is a sweep from the inventory side.

**`notes/smr_coverage.md` gives all 192 rows a disposition**: 85 ported, **25 gaps**, 1
plan/inventory contradiction, 28 reclamation families out of scope, 23 skips, 10 deliberate
divergences, 8 defers, 6 conditionals, 4 bench experiments, 2 done. The 25 gaps are **+12.5 h**,
distributed C1 +2, C2 +1, C3 +0.5, C4 +4.5, C5 +0.5, C6 +3, C7 +1, and the ones that matter
most cost nothing:

- **the ERA theorem** (116) — no scheme gets ease of integration, robustness *and* applicability;
  at most two. The one-paragraph answer to why each scheme has exactly one glaring weakness, and
  it belongs in step 1's docs.
- **the pointer-word filter** (414, CORE at 1059) — announcing a *tagged* word never matches the
  retired object's real address, so a marked node can be freed while protected. A soundness bug,
  and C9 would have hit it.
- **`swap(a, b)`** (CORE 1054) — and it **settles the deferred return-type question**: `&'a T`
  borrowed from `&'a mut self` cannot coexist with a swap, because the swap needs `&mut` while
  the reference is live. Raw pointer, or no hand-over-hand.
- **the empty state** (CORE 1049) — three states, not two: owning no record, owning an
  unassociated record, announcing. This is the open "what does `as_ref` do on a guard that
  announces nothing" question, and P1121 answers it.
- **`cleanup()` with bulk-reclaim quiescence** (474, 1126) — without it, "a dropped domain
  asserts its retired list is empty" cannot be written honestly.
- **`repin`** (828) — a long reader pins one epoch and blocks *all* reclamation in the domain.
  A queue consumer does not care; a skiplist traversal does.

---

## The SegQueue side, in edit order

Line numbers are `crates/concurrent/src/seg_queue.rs` as it stands today (9 tests, 5 loom
models, the refcount and the ordered walk). Each edit is its own commit and its own green
test run — there is no intermediate state where the queue is broken.

| # | Edit | What it touches | Verified by | Est |
|---|---|---|---|---:|
| **Q1** | **A loss-detecting loom model, written against the CURRENT code.** 2 producers, 2 consumers, enough items to cross a boundary at `SEG_LEN = 2`; assert the multiset of popped values equals the pushed set. | new `loom_tests` entry | passes today — that is the point. It becomes the regression harness for Q3 | 1.5 h |
| **Q2** | **Trait integration.** `SegGuard` (`:83`, `:118-139`) becomes the scheme's guard; `Segment::ref_count` (`:78`, `:104`) and `acquire_ref`/`release_ref` (`:109-115`) are deleted; `try_reclaim`'s refcount check (`:259`) becomes nothing and its `Box::from_raw` becomes `retire`. `SegQueue<T, R: Reclaim>` stores its `Domain`. Scheme = `Leak`. | the struct defs, `SegGuard`, `try_reclaim` | 9 std + 5 loom + Q1 green · Miri clean with `-Zmiri-ignore-leaks` · **a test asserting the chain grows** | 4 h |
| **Q3** | **The protect-source restructure.** `advance_tail`'s tail (`:215`) and `pop`'s advance (`:289`) stop protecting through `&cur.next` and protect through `&self.tail` / `&self.head`. Guards per thread 2 → 1. | `push`/`advance_tail`/`pop` only | Q1 must still pass — this is the FIFO question, and Q1 exists to answer it | 2.5 h |
| **Q4** | Bench arms for the new shape. `benches/seg_queue.rs` already has the `Queue<T>` trait and the N→1 scenarios; add one arm per scheme. | bench only | runs, numbers recorded against a sealed prediction | 1 h |

Q1–Q4 is **9 h**; the Treiber stack is the other 5 h of step 2.

**Why Q2 and Q3 can be separate.** Under `Leak` nothing is ever freed, so requirement (A) is
*vacuously* satisfied and protecting through `cur.next` is harmless. That is what lets the
trait integration land and be tested before the restructure, instead of one large change where
a failure could be either.

**Why Q1 comes first and is not optional.** Q3 rests on a claim: after `CAS head A→B`,
protecting through `&self.head` can return a *later* segment than `B` if another consumer has
already advanced, and that is safe because a consumer only advances past a drained segment.
That is an argument, not a measurement, and every later step builds on it. Q1 turns it into
something loom can refute.

---

## Step 1 — Define the problem

The deliverable is the safety documentation, because that is the load-bearing part and the
part most libraries get wrong.

### What the docs must state

An SMR scheme's safety argument has two halves, and a scheme supplies only one:

A **guard** is the thing a reader holds while it is using a pointer — the same role
`SegGuard` already plays in `seg_queue.rs:83`: acquire pins, `Drop` unpins, and while you
hold it the pointee is safe to read. `crossbeam-epoch` and `seize` both call it `Guard`.

- **(A) Unobtainability** — after `retire(p)`, no thread that does not already hold `p` may
  obtain it. **The data structure supplies this.**
- **(B) Grace** — threads that already hold `p` eventually let go. **The scheme supplies
  this.**

"Unlink before retire" is the usual phrasing and it is a special case. The precise statement
is about protect sources:

> **The set of atomics ever passed to `protect` is the root set. `retire(p)` is sound once
> `p` is unreachable from that root set.** A heap edge nobody protects through is not a root.

This is why hazard pointers apply to only 3 of 18 data structures in Singh's survey.

### The retire list is scheme-owned (decided)

`retire(ptr)` hands the object to the scheme, which batches and frees it — the folly /
crossbeam-epoch / haphazard / seize shape. The alternative considered was a `stamp` +
`can_free(ptr, stamp)` pair, letting SegQueue keep its chain as a free ordered retire list.
Rejected, strongest reason first:

1. **Bounded garbage becomes impossible.** Under `can_free`, the scheme can never make
   progress on its own — freeing happens only when a client walks its own list, and
   `try_reclaim` is called from `pop`, so a queue nobody pops never frees however long the
   grace period has elapsed. folly's threshold machinery exists precisely to bound this and
   can only live in the scheme.
2. **Hazard-pointer scans must be batched.** Build the guarded set once, match all retired
   objects against it: O(R + H). Per-object `can_free` is O(R × H) — 8× the work at 8
   threads, 64× at 64.
3. **One unsafe boundary instead of one per client**, and there will be four clients.
4. The zero-allocation advantage evaporates: production makes the retired list **intrusive**
   (folly's `hazptr_obj` carries its own `next_`), which costs one pointer per `Segment` and
   allocates nothing.

Consequence, stated plainly: the ordered walk stops being load-bearing *for safety*. The
`reclaim` cursor stays — it is still how you know `tail` has also passed — but the
front-to-back induction no longer carries the argument, because (A) now comes from the root
set being `{head, tail}` and (B) from the scheme. `Box::from_raw` becomes `retire`.

### Guards are growable, not capped (decided)

One pointer per guard; hold N guards for N pointers. With batching inside the scheme a
growable per-thread record is cheap — folly's thread cache is exactly this — and a cap would
lock the skiplist out of using the crate later. The acceptance test for growth is any
hand-over-hand traversal, which needs two guards at once.

### `protect` retargets a held guard; it does not mint one (decided)

Two shapes were on the table:

```rust
// A - the domain mints a guard, which IS the announcement
let g = domain.protect(&node.next);

// B - the guard is acquired once, then retargeted
let mut g = domain.guard();
let n = g.protect(&node.next);
```

**B.** The reason is not the acquisition count, it is that **A cannot express "stay protected
across a retry".** Under A, re-protecting means minting a second guard, and both ways of doing
that are wrong:

- drop the old guard then mint the new one, which leaves an **unprotected window**. Under
  epoch that is an `unpin`/`pin` pair with a gap, and anything carried across the boundary can
  be freed inside it;
- hold the old guard while minting the new one, which means **two live slots for one logical
  cursor** - exactly the 2 -> 1 reduction Q3 was for.

B has one slot and one pin held for the whole operation, with the announcement overwritten in
place. The acquisition counts follow from that rather than motivating it: `pop`'s retry loop
goes from N acquisitions to 1, and a Harris traversal of length N from N to 2. The *number of
announcements* - and so the number of `SeqCst` fences under HP - is identical in both shapes.
Only the registry bookkeeping differs.

The three things the schemes disagree about, which is what forces the trait to B:

| | a guard *is* | protects | acquiring one costs | protecting one pointer costs |
|---|---|---|---|---|
| Hazard | one slot you own in your record | **one** address | find a free slot, maybe grow | store + `SeqCst` fence + reload-and-compare, in a loop |
| Epoch | a pin on your record | **everything** reachable while pinned | a re-entrant counter bump | a plain load |
| Leak | nothing | everything, forever | nothing | a plain load |

What B costs, and is accepted: one extra state to document - a guard that owns a slot but
announces nothing, i.e. between `domain.guard()` and the first `protect`. `as_ref` on such a
guard has to be defined.

### `protect` returns a raw pointer (decided — the coverage sweep settled it)

This was parked for C9 to refute under `trybuild`. It did not need to be: `swap` is a **CORE**
row (inventory 1054) and it is incompatible with the alternative.

Raw `*mut T` leaves it to the caller not to retarget while a derived reference is live.
`&'a T` borrowed from `&'a mut self` makes the borrow checker forbid that — a real safety win,
and the thing the inventory singles out as haphazard's one advantage over folly and P1121
(1283: "its lifetime-based protection scoping statically prevents use-after-reset, which
neither folly nor P1121 can express"). But it breaks the standard hand-over-hand advance, which
does not re-protect `pred` (there is no atomic to re-protect it *from*) but **swaps the two
guards** and retargets the freed one:

```rust
core::mem::swap(&mut gp, &mut gc);   // needs &mut gc ...
let curr = gc.protect(&curr.next);   // ... but `curr` is still borrowed from gc
```

The swap needs `&mut gc` while `curr` is still borrowed from it. haphazard ties `protect` to
`&'l mut self` and inherits exactly this. So: **raw pointer, and `try_protect` becomes a safe
`fn`** — announcing an address pins it and nothing more, so the whole unsafe surface collapses
into the dereference. That is a strictly smaller contract than the alternative bought.

What is given up, recorded so it is a trade and not an oversight: nothing stops a caller
retargeting a guard while holding a pointer derived from its previous announcement. Miri in a
client catches it; the type system will not.

### `try_protect` is the only required method; `protect` is a default (decided)

The trait carries **no retry loop**. One required method:

```rust
fn try_protect<T>(&mut self, p: *mut T, src: &AtomicPtr<T>)
    -> Result<*mut T, *mut T>;   // Ok(p) | Err(what src holds now)
```

One attempt: announce `p`, fence, re-read `src`. The caller passes the pointer **it already
loaded**, so there is no redundant load and nothing to re-derive, and it absorbs the retry into
the loop it was going to write anyway.

A looping `protect` that hid the retry was rejected. The reason is not that nesting is slow:

- **Both loops retry on the same condition** — `src` changed — and only the caller knows what
  the pointer was *for*. The inner loop can only re-stabilise and hand back a value the
  caller's CAS may immediately find stale, so its work is duplicated, **and duplicated in the
  most expensive currency HP has**: every inner retry is a `SeqCst` fence establishing a fact
  with a lifetime of zero instructions.
- Cost composes by the wrong operator: nested is inner × outer, flat is inner + outer.
- An unbounded spin inside a library call whose retry condition the caller can already observe
  is the mistake already removed from `pop`.

The convenience survives as a **provided method** with a default body over `try_protect`, so
implementors write one method and nothing else, and a caller with no candidate yet (the start
of an operation — "give me the current head, protected") still has it.

### `Ok`/`Err` mean protectability, not freshness (decided)

`Ok(p)`: `p` is protected and safe to dereference until this guard is retargeted or dropped.
`Err(q)`: `p` could **not** be protected; `q` is what `src` held when that was determined,
offered to save the caller a load. **`q` carries no promise of being current.**

So epoch returns `Ok(p)` unconditionally and does **no** load — a pin already covers `p`
whether or not `src` moved. HP's re-read stays, but it belongs to HP's own safety argument
rather than being a service to the caller.

The alternative — freshness in the contract, forcing epoch to re-read — was rejected on two
grounds:

1. **Freshness is not durable.** `Ok(p)` could only ever mean "`src` held `p` at some instant
   after the announce"; by the time the caller acts it may be stale again. Any client that must
   act on freshness has to re-check at the point of action, and for a mutating client that
   re-check **is the CAS**. A guarantee nobody can rely on is not worth a load on epoch's
   hottest path.
2. **The client can do its own load, and Harris must tolerate staleness regardless.** Harris
   reads `pred.next` itself and already restarts on an inconsistency, because a stale read is
   possible between any load and any use no matter what `try_protect` promises.

Note on the apparent precedent: folly's `try_protect(T*& ptr, const Atom<T*>& src)` and
haphazard's `Err`-carries-the-new-value look like votes for freshness, but **both libraries are
HP-only, and for HP the two contracts are indistinguishable** — HP must re-read for safety
either way. They are not evidence.

Falls out of this, and is assertable: **`retries == 0` under `Epoch`, `retries > 0` under
`Hazard`.**

### Scheme-agnostic means one source text, not one execution path (decided)

C8's acceptance was "the Treiber stack runs against `Leak`, `Hazard` and `Epoch` unchanged".
*Unchanged* means **one source text — no `cfg`, no scheme-specific branch in the client**. It
never meant identical runtime control flow, and reading it that way would have cost epoch a
load per protect to keep a test's shape tidy.

So every client test splits in two:

- **the generic body** carries the scheme-agnostic invariants: no lost nodes, no duplicates,
  LIFO, Miri clean;
- **three thin wrappers** carry outcomes that are specific per implementation — `Leak`:
  reclaim count `== 0` and a monotonically growing retired chain; `Hazard`: `retries > 0` under
  contention, at least one loom interleaving reaching the retry path, garbage bounded by the
  threshold, both negative controls; `Epoch`: `retries == 0`, garbage bounded by three
  generations, pin/unpin nesting.

Asserting "epoch's `Err` path is dead" is a real claim about epoch that a uniform-control-flow
test would have hidden.

### `retire` is required per scheme; the shared part is the trigger, not the storage (decided)

The two schemes do not agree on what a retired-object list *is*:

| | Hazard | Epoch |
|---|---|---|
| storage | **intrusive** — `RetireLink` on the object, zero allocation | **bags** — arrays of `Deferred`, `MAX_OBJECTS = 64`, nothing on the object |
| where objects wait | the domain's sharded list, pushed in runs of ~20 | **thread-local**, no shared state 63 times in 64 |
| granularity of the free decision | per **object** — free some, retain others | per **bag** — all 64 or none |
| ordering | irrelevant | **FIFO by seal epoch**, which is what makes peeking only the head sufficient |

A single representation has to pick a side on row 1, and both choices reverse a decision this
plan already made for a stated reason: intrusive storage costs epoch its bags, and with them
the contiguous 64-at-a-time free and `Pointable for [MaybeUninit<T>]`, which is C7's reason to
exist; bag storage costs hazard its intrusive zero-allocation list, which is step 1's reason #4
for making the retire list scheme-owned at all. Row 3 adds that a batch predicate must be a
*filter* for hazard and *accept-or-reject* for epoch — fit it to hazard and epoch's O(1)
head-peek degrades to a walk.

So **`retire` is a required trait method.** That does not mean C2 and C4 are written twice —
they are a library of components the implementors assemble differently:

| Component | Edit | Used by |
|---|---|---|
| `Retired` — type-erased `(ptr, reclaim_fn)` | C2 | both |
| `RetireLink` + intrusive sharded list + batch push | C2 | `Leak`, `Hazard` |
| `Bag` / `Deferred` + sealed-bag queue | C6 | `Epoch` |
| **count + time triggers · offload executor · recursion flattener** | C4 | **both** |

The genuinely shared surface is the last row, and it is shared for a reason unrelated to
storage: it decides **who pays for a round and when**, which is the same question for both
schemes. That is also why C4's acceptance test — offload on and off produce identical
reclamation with different latency owners — stays scheme-independent.

This is *not* the `can_free(ptr, stamp)` shape step 1 rejected. That put the list in the
**client**, which killed bounded garbage and made matching O(R × H). Here the list stays
scheme-owned and only the plumbing is shared.

Falsifier, if it ever needs revisiting: name one representation that keeps intrusive
zero-allocation for hazard **and** contiguous 64-object bags for epoch. folly and
crossbeam-epoch, written by people who knew both schemes cold, converged on different
representations.

### Blog beat

A vs B is the clearest "the two schemes disagree about what a guard *is*" moment in the whole
build - hazard is per-address, epoch is per-thread - and the argument lands without any
measurement. Keep it for the reclamation series.

### Three more, decided (two against the obvious answer)

**`Domain` is a cheaply-clonable handle, not a global static and not a lifetime.**
Internally an `Arc`, like `crossbeam-epoch`'s `Collector`; `Domain::global()` for the common
case. **No type-level families** — that is where `haphazard` became unsound (its open issue
#54: `unique_domain!` can mint two domains sharing one family). Instead the queue *stores*
the domain it was built with, so an object can never be retired in one domain while a guard
from another protects it — prevented by construction rather than by a type parameter. For
loom, tests build an explicit `Domain` inside `loom::model` and never touch `global()`, which
sidesteps the global-state problem that made two tests race each other in the prototype.

**`retire` does NOT take a drop function — the `Retire` trait carries an overridable
`reclaim`.** This reverses the earlier lean, and the reason is the named consumers:

```rust
pub unsafe trait Retire: Send {
    fn retire_link(&self) -> &RetireLink;
    /// Dispose of this object once the grace period has passed.
    /// Default: run the destructor and free the `Box` allocation.
    unsafe fn reclaim(ptr: *mut Self);
}
```

`T: Send` plus a monomorphised `drop_in_place` would be enough **only if every retired object
came from a `Box`.** `bufpool` page reclaim and P4's price levels are both named consumers of
this crate and both allocate from a pool, so `Box::from_raw` would be wrong for them. folly
solves this with a deleter type parameter (`hazptr_deleter<T, D>`); putting it on the object's
trait instead keeps `retire`'s signature to one parameter and still monomorphises into the
stored function pointer, so it costs nothing.

**No default type parameter on `SegQueue<T, R>` — not yet, and never `Leak`.** The earlier
lean was `R = Leak` so call sites would not churn. That is a bad production default: someone
writes `SegQueue<usize>` and silently gets an unbounded leak. Worse, it would hide the leak
exactly where the honest-test discipline wants it visible — `SegQueue<usize, Leak>` **names
the leak in the type**, which is what makes step 2's acceptance test mean something.

So: explicit through steps 2–4 (few call sites: tests and the bench), then add a default equal
to whatever step 4 measures as the best general-purpose scheme — probably epoch. Adding a
default later is backward-compatible, so this costs nothing. `Leak` stays public but
documented as benchmark-only.

### Known trap, to state in the docs

A refcount-in-the-object **can** implement this trait and still be unsound: `protect` would be
`fetch_add` on the object, which is exactly the bug already proven. **The safety docs constrain
the client, not the implementor.** Each scheme needs its own soundness argument.

---

## Step 2 — Integrate with SegQueue

### Why a `Leak` implementor comes first

It is the only implementor that can exist before the registry, and it gives the refactor an
acceptance test it otherwise would not have: `SegQueue<T, Leak>` must reproduce today's
behaviour exactly.

### The restructure, and why it belongs here

(A) is currently violated. `protect` is called on `&cur.next`, a write-once field, so a retired
segment stays obtainable forever. Move both advances onto the moving cursors:

```
pop advance:   CAS head A→B, then protect(&self.head)    // not &A.next
push advance:  CAS tail A→B, then protect(&self.tail)    // not &A.next
```

The successor is still *read* out of `A.next` while `A` is guarded, and used as a CAS
argument — it is just never announced through. Root set becomes `{head, tail}`, and (A) holds.
Falls out of it: guards per thread drops 2 → 1, because the overlap existed only to cover a
vacuous validate.

### The Treiber stack, and why it is in this step

A second *client* validates the trait's client-facing surface; a second *implementation*
validates the implementor-facing one. The stack is ~80 lines, a scheme is 200–400, so the cheap
falsifier runs first. Finding out the trait is wrong after writing two schemes against it is the
expensive order.

It is also the contrasting case: a popped node genuinely *is* unlinked, so it satisfies (A) the
ordinary way, while SegQueue satisfies it through the root-set argument. Having both is what
makes the contract legible.

### Acceptance

- 9 std tests and 5 loom models green against `SegQueue<T, Leak>`
- Miri clean with `-Zmiri-ignore-leaks`
- a test asserting the chain **does** grow — `Leak` must be observably leaking, or the step
  proved nothing
- the Treiber stack passes its own loom models on the same trait

---

## Step 3 — Implement the schemes

`notes/smr_inventory.md` marks every mechanism in folly's `hazptr` and in `crossbeam-epoch` as
core / important / optional / skip. Take scope from the `core` rows; the rest is a menu.

### Scope: epoch ships its native API too (decided)

The `Reclaim` trait is the lowest common denominator — a plain `AtomicPtr`, because hazard
pointers cannot express a guard-lifetimed `Shared<'g, T>`. So epoch carries **two** APIs, the
same shape as `crossbeam-epoch`:

| API | Used by |
|---|---|
| the `Reclaim` trait impl | SegQueue, Treiber stack — anything that wants to be scheme-agnostic |
| the native typed API: `Atomic<T>` / `Owned<T>` / `Shared<'g, T>` / `Pointable`, pointer tagging, the full `compare_exchange` family | clients that need a mark bit on a pointer, i.e. logical deletion |

Breakdown of the 38 h: reclamation core — three-generation cycle, bags, advance policy —
20 h; `Atomic`/`Owned`/`Shared` + tagging + the CAS family, 12 h; `Pointable` (unsized and
`[MaybeUninit<T>]` payloads, which is what the bags need), 6 h.

### The typed API's acceptance test: a Harris linked set (6 h, scheduled)

SegQueue goes through the trait and never touches the typed API, so without a consumer the
pointer types would ship untested until the skiplist. A **Harris (2001) lock-free linked
set** is the cheapest one that exercises all of it, and it is the direct precursor to the
skiplist — a skiplist is Harris's list at several levels.

What it tests that nothing else in this plan does:

| Mechanism | Why only this client reaches it |
|---|---|
| pointer tagging | logical deletion sets a **mark bit in the node's `next` pointer**; this is tagging's actual use, not a demo |
| the tagged `compare_exchange` family | physical unlinking is a CAS on a pointer whose low bit is part of the value |
| **two guards held at once** | traversal is hand-over-hand on `(pred, curr)` — the acceptance test for the growable-guard decision above |

It also completes the contract's teaching surface, because the three clients end up with
three *different* arguments for requirement (A):

| Client | How (A) is satisfied |
|---|---|
| Treiber stack | the node is unlinked by the pop that removes it |
| Harris set | **deferred** unlink — a node is logically deleted (marked) while still reachable, and may only be retired after it is *physically* unlinked |
| SegQueue | never unlinked at all; (A) comes from the root set being `{head, tail}` |

That middle row is the one worth having. "Mark now, retire later" is the case a contract
stated only as "unlink before retire" gets wrong.

Things to derive rather than copy:

- **Hazard pointers** — the announce/validate pair is a store followed by a load of a *different*
  variable on the same thread. `Release`/`Acquire` cannot order that. Loom is trustworthy here:
  the handoff is store/load, not CAS, so the blind spot in `reference_loom_cas_blindspot` does
  not apply.
- **Epoch** — why **three** generations and not two. Build it with two and let loom produce the
  counterexample.

Negative controls are mandatory for both (`reference_concurrent_test_checklist`). For a
scan-based scheme they bracket it from both sides: always-protected must stall reclamation and
fail a chain-length test; never-protected must produce a Miri UAF.

---

## Step 4 — Bench the reclamation axis

The reason the trait exists. The previous measurement could not attribute the 6× gap to
crossbeam, because crossbeam differs in **two** ways at once — layout *and* reclamation. Behind
one trait, with identical queue code, reclamation becomes the only variable.

Arms: `Leak` · `Hazard` · `Epoch` · `Mutex<VecDeque>` · real `crossbeam_queue::SegQueue`.
Shape: N producers → 1 consumer, N ∈ {1, 2, 4, 8} — the shape both real callers have.

One more arm, to settle the abstraction question with a number instead of an argument:
**`Epoch` through the `Reclaim` trait vs `Epoch` through its native API**, identical queue,
identical scheme, the only difference being whether the calls cross the trait. Prediction to
seal: indistinguishable, inside noise, because every call monomorphises and inlines. If that
holds, the trait is free and the question is closed for the rest of the plan. If it does not,
the gap names a place where the trait's shape costs a load or loses an optimisation - which is
worth more than the argument was.

**Two measurements, not one, because throughput cannot see the thing that matters.** A
millisecond scan spike charged to one unlucky `retire` averages into "HP is a bit slower", so
the mechanisms in C2/C4/C5 that exist to control it would be invisible here — neither
justifiable nor, if omitted, detectable:

1. **Retire-call latency distribution** — p99 and max, not just the mean. This is the arm that
   prices the offload executor and the shards. Without it the plan has no measurement that
   could ever have justified porting them.
2. **Guard-acquisition cost, and acquisitions per operation.** This is the thread cache's
   falsifier. Shape B should make acquisition ~1 per operation; if that holds and the cost is
   invisible, `hazptr_tc` becomes a decided skip with evidence rather than a judgement call.

Seal the prediction first. The open question: the last measurement went `7.7 → 77.8 → 105.9 →
104.9` ns against crossbeam's flat `13.8 → 16.7`. If `Epoch` lands near crossbeam, reclamation
was the whole story; if it stays 6× off, layout is implicated and step 5 becomes the interesting
one rather than a formality.

Note before measuring: HP's per-protect `SeqCst` fence is the known cost, and folly's fix —
asymmetric barriers via `membarrier` — is **not available on macOS/aarch64**. Put that in the
prediction note so a bad HP number is explainable rather than mysterious.

---

## Step 5 — The crossbeam approach

Two sub-steps, each moving one thing.

**5a — layout A, a global index (7 h).** Replace the per-segment `claimed`/`consumed` pair with
one global position (index + block pointer), the crossbeam family's shape. Still on the
`Reclaim` trait — this step moves layout, not reclamation — with the scheme pinned to step 4's
winner, so the comparison is **pure layout**: one permanently-hot contended line that never
moves, against a cursor pair that migrates to each new segment and arrives cold.

Concretely: `Segment` loses both cursors and keeps only `slots` + `next`; `SegQueue` gains
`head`/`tail` as `{ index: AtomicUsize, block: AtomicPtr<Block> }`; `push` does one `fetch_add`
on the global index and derives `(block, slot)` from it. **This is where Track B's avoided case
shows up** — a producer can be handed a ticket for a block that is not linked yet and has to
wait for whoever is installing it. Expect that spin to be visible in the 8-producer arm; it is
the known cost of the layout, and the reason B was built first.

**5b — crossbeam-exact (11 h).** Block cursor, per-slot `WRITE` / `READ` / `DESTROY` bits, the
tunings. Then diff against the real source.

### Why 5b is a third reclamation technique and not a `Reclaim` implementation

Real `crossbeam_queue::SegQueue` does **not** use `crossbeam-epoch`. It self-reclaims: each slot
carries state bits, and the last thread to finish with a block frees it. The participant set per
slot is statically known and finite, which is what makes a cooperative hand-off possible at all.

**That cannot be a `Reclaim` implementor.** The announcement is per-slot state *inside the
structure being reclaimed* — the same shape as the ordered walk, and the same reason a
"is this pointer protected?" query cannot live on the trait (epoch has no per-pointer
information and could not answer it).

So 5b sits outside step 1's abstraction on purpose, and the question it answers is worth being
able to answer out loud: *crossbeam ships an epoch crate — why doesn't its own queue use it?*
Because this structure self-reclaims more cheaply than pin/unpin, and no interface unifying
hazard pointers with epochs can express it.

---

## Step 6 — Which wins, and why

The full matrix, and the written argument. Comparisons that are each one variable:

| Comparison | Isolates |
|---|---|
| `Leak` vs `Hazard` vs `Epoch`, layout B | reclamation scheme |
| layout B vs layout A, reclamation fixed | layout |
| layout A + best scheme vs crossbeam-exact | crossbeam's own DESTROY-bit scheme |
| crossbeam-exact vs real `crossbeam` | whatever is left: tunings and the source diff |

The deliverable is not the fastest number. It is being able to say which of the four axes the
difference came from, with a measurement per axis — which is the thing the earlier six rounds
could not do.

---

## Reclamation techniques this plan covers

Five families, which is the "lose no technique" goal made concrete:

| Technique | Where | State |
|---|---|---|
| Leak / no reclamation | step 2 | baseline |
| Reference count inside the object | done | **proven unsound** |
| RAII guard + three free conditions + ordered trailing cursor | done | **all three survive** |
| Hazard pointers — announce *addresses* | step 3 | |
| **Link counting** — make a child safe because its *parent* was protected | step 3, C10 | the mechanism that lifts HP's "3 of 18 structures" limit |
| Epoch — announce *time* | step 3 | |
| Per-slot cooperative hand-off (`DESTROY` bit) | step 5b | not a trait impl, on purpose |

Deliberately out of scope, recorded in `notes/smr_inventory.md` so it is a decision and not an
oversight: QSBR, RCU, hazard eras / IBR, Hyaline, Crystalline, and the optimistic-access family.
VBR is the interesting one — the only family that drops requirement (A) entirely, paying for it
with a type-preserving allocator and a version word per mutable field.

---

## Not in this plan

- **EventCount**, and the `CachePadded` / `Backoff` corrections — `concurrent`, not SegQueue.
  Independent work, scheduled in `PRODUCT_TREE.md` §9.
- The remaining folly gaps — `notes/folly_gap_analysis.md`, each picked up with its own artifact.
- The blog. An output of finished work, not part of the build.
