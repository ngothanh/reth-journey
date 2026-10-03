# `reclaim` + SegQueue — implementation plan

> **Component**: SegQueue (unbounded lock-free MPMC) + `reclaim` (safe memory reclamation)
> **Crate(s) touched**: **[NEW]** `crates/reclaim/`; `concurrent` → `SegQueue` becomes generic over the scheme
> **Bar**: (c) — both are Layer-1 substrate, so the two-track rule applies: a sealed numeric prediction *before* every measurement, then the measured number, then reconcile the gap
> **Mirror target**: folly `hazptr` (Domain / holder / retire), `crossbeam-epoch` (Collector / LocalHandle / Guard / three-epoch cycle), `haphazard`, and `crossbeam_queue::SegQueue` (block cursor + per-slot DESTROY bit)
> **Feeds into**: `concurrent` skiplist · `bufpool` page reclaim · P4 price levels · P5 ledger + cross-shard queues
> **Current position**: refcount built, proven unsound, and benched. Reclamation is the open problem.
> **Remaining**: **≈ 91 h** across 6 steps
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
| **1** | **Define the problem** | the `Reclaim` trait and its safety documentation. No scheme, no registry. | A contract with no implementor and no client is unfalsifiable | 4 h |
| **2** | **Integrate with SegQueue** | `Leak` (the trivial implementor) · `SegQueue<T, R: Reclaim>` · the protect-source restructure · a Treiber stack as a second client | Integrable and not SegQueue-shaped — but nothing yet reclaims anything | 13 h |
| **3** | **Implement the schemes** | the shared registry + `Domain` (10 h) · hazard pointers (14 h) · epoch (20 h) | Two schemes exist behind one interface and have never been compared | 44 h |
| **4** | **Bench the reclamation axis** | `Leak` / `Hazard` / `Epoch` / `Mutex<VecDeque>` / real `crossbeam`, identical queue code | Reclamation is now isolated. Any remaining gap to crossbeam is **layout** or **crossbeam's own scheme** — and neither has been built | 8 h |
| **5** | **The crossbeam approach** | layout A, a global index, reclamation held fixed (7 h) · crossbeam-exact: block cursor + per-slot `WRITE`/`READ`/`DESTROY` bits (11 h) | Everything is built; nothing has been compared head to head | 18 h |
| **6** | **Final bench — which wins, and why** | the full matrix, one variable per comparison, and the written argument for the winner | — endpoint | 4 h |

---

## Step 1 — Define the problem

The deliverable is the safety documentation, because that is the load-bearing part and the
part most libraries get wrong.

### What the docs must state

An SMR scheme's safety argument has two halves, and a scheme supplies only one:

- **(A) Unobtainability** — after `retire(p)`, no thread that does not already hold `p` may
  obtain it. **The data structure supplies this.**
- **(B) Grace** — threads that already hold `p` eventually let go. **The scheme supplies
  this.**

"Unlink before retire" is the usual phrasing and it is a special case. The precise statement
is about protect sources:

> **The set of atomics ever passed to `protect` is the root set. `retire(p)` is sound once
> `p` is unreachable from that root set.** A heap edge nobody protects through is not a root.

This is why hazard pointers apply to only 3 of 18 data structures in Singh's survey.

### Open questions to settle here

Each one changes the signature, so decide before writing:

1. **Shield granularity** — one pointer per shield, so two simultaneous protections means two
   shields? Or N slots per shield? Hazard pointers have a real per-record limit; epoch has
   none. Either choice imposes HP's constraint on epoch or hides it.
2. **Does `retire` carry a drop function**, or is `T: Send` enough to monomorphise
   `drop_in_place`?
3. **Who owns the retire list.** The live trade: SegQueue's chain from `reclaim` to `head`
   *is* a retire list — ordered, in the structure, free. A scheme-owned list throws that away
   and allocates. Epoch needs its own garbage bags regardless.
4. **`Domain`, or one global registry.** Global is simpler, and is what made two tests race
   each other in the discarded prototype.

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

The successor is still *read* out of `A.next` while `A` is shielded, and used as a CAS
argument — it is just never announced through. Root set becomes `{head, tail}`, and (A) holds.
Falls out of it: shields per thread drops 2 → 1, because the overlap existed only to cover a
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
one global position (index + block pointer), the crossbeam family's shape. Reclamation held
fixed at the best scheme from step 4, so the comparison is **pure layout**: one permanently-hot
contended line that never moves, against a cursor pair that migrates to each new segment and
arrives cold.

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
