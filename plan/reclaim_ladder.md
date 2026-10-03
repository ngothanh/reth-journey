# `reclaim` + SegQueue — the build steps

Reworked 2026-10-03. Supersedes the B0→B3 ordering, which built reclamation schemes
*inside* SegQueue. They are general concepts and now live in their own Layer-1 crate
(`PRODUCT_TREE.md` §3/§4/§6/§8/§9), so the order inverts: **contract first, then one
trivial implementation to integrate against, then a second client to falsify the contract,
then the real implementations.**

Each step ends at a wall that is the reason the next step exists, and each has an
acceptance test that can fail. Every bench is preceded by a numeric prediction sealed in
`notes/` first.

---

## Where we are

Spent ≈ **26 h** (from `notes/seg_queue_build_log.md` and the old estimate lines):

| Done | What it bought | Spent |
|---|---|---:|
| **B0** queue on leak | push/pop, ticket cursors, boundary CAS, 9 std tests, 5 loom models, Miri clean with `-Zmiri-ignore-leaks`, bench vs `Mutex<VecDeque>` | ~8 h |
| **B0.5** check-then-claim `pop` | removed the unbounded internal spin; pop's worst case stops depending on another thread's scheduler | ~1.5 h |
| **B1** refcount, and its impossibility | the announcement-is-the-dangerous-access proof: a counter inside the object cannot be made sound. Also RAII `SegGuard`, the three free conditions, and the ordered trailing `reclaim` cursor — **all three survive into the final design** | ~3 h |
| **Blog** 6 parts + 33 figures | on branch `blog/segqueue` | ~11 h |
| **Research** | `notes/smr_inventory.md` (192 mechanisms), `notes/folly_gap_analysis.md` (~24 gaps) | ~2 h |

Currently in `crates/concurrent/src/seg_queue.rs`: 9 passing tests, 5 loom models, the refcount
and the ordered walk. **R1 deletes the refcount** — its value is already banked in blog Part 5,
and the code was only ever kept as a documented failure.

---

## Remaining: 87 h

| | Steps | Hours |
|---|---|---:|
| **Phase 1 — reclamation axis** | R0 → R6 | **69 h** |
| **Phase 2 — layout axis (the crossbeam techniques)** | L1 → L2 | **18 h** |

## Phase 1 — the steps

| Step | What you build | The wall that forces the next step | Est |
|---|---|---|---:|
| **R0** | The `Reclaim` trait and **its safety documentation**. No scheme, no registry. The docs are the deliverable. | A contract with no implementor and no client is unfalsifiable — nothing yet says it is *writable* | 4 h |
| **R1** | `Leak` — the trivial implementor. `shield()` is a no-op, `protect` is a plain load, `retire` drops the pointer on the floor. Then `SegQueue<T, R: Reclaim>` and the protect-source restructure. | Proves the trait is *integrable* and the queue still works — but `Leak` exercises none of the contract, and nothing confirms the trait fits anything but SegQueue | 8 h |
| **R2** | A **Treiber stack** on the same trait. Second *client*, not second implementation. | Proves the trait is not SegQueue-shaped. Still only one implementor, so nothing confirms the trait is implementable by a scheme that actually reclaims | 5 h |
| **R3** | The **registry**: immortal per-thread records + `Domain`. Shared by both real schemes; no reclamation policy yet. | Per-thread state that is never freed exists, but nothing announces into it | 10 h |
| **R4** | **Hazard pointers** behind the trait. Announce addresses; validate; scan. | Works. Costs a store and a fence on *every* protect — per-pointer, on the hot path | 14 h |
| **R5** | **Epoch (EBR)** behind the trait. Announce epochs; three-generation garbage; advance policy. | Works, and pays per *critical section* instead of per pointer — but garbage is now unbounded if a thread parks pinned | 20 h |
| **R6** | The **bench matrix**: `Leak` / `Hazard` / `Epoch` / `Mutex<VecDeque>` / real `crossbeam`, same queue code, one variable at a time. | Reclamation is now isolated from layout. Whatever gap remains against crossbeam is attributable to **layout**, which is the layout axis below | 8 h |

**Phase 1 subtotal ≈ 69 h.** Phase 2 (the layout axis and crossbeam's own scheme) is below.

---

## R0 — the contract

The deliverable is the safety documentation, because that is the part that is load-bearing
and the part that is wrong in most libraries.

### What the docs must state

An SMR scheme's safety argument has two halves, and a scheme supplies only one:

- **(A) Unobtainability** — after `retire(p)`, no thread that does not already hold `p`
  may obtain it. **The data structure supplies this.**
- **(B) Grace** — threads that already hold `p` eventually let go. **The scheme supplies
  this.**

"Unlink before retire" is the usual phrasing and it is a special case. The precise
statement is about protect sources:

> **The set of atomics ever passed to `protect` is the root set. `retire(p)` is sound once
> `p` is unreachable from that root set.** A heap edge nobody protects through is not a
> root.

Source: `notes/smr_inventory.md`, which is also why hazard pointers apply to only 3 of 18
data structures in Singh's survey.

### Open questions to settle at R0

Decide these before writing the trait, because each one changes the signature:

1. **Shield granularity.** One pointer per shield, so two simultaneous protections means
   two shields? Or N slots per shield? HP has a real per-record limit; EBR has none.
   Whichever is chosen imposes HP's constraint on EBR or hides it.
2. **Does `retire` take a drop function**, or is `T: Send` enough to monomorphise
   `drop_in_place`?
3. **Who owns the retire list.** This is the live trade: SegQueue's chain from `reclaim`
   to `head` *is* a retire list, in order, for free. A scheme-owned retire list throws
   that away and allocates. EBR needs its own garbage bags regardless.
4. **`Domain` or one global registry.** Global is simpler and is what made two tests race
   each other in the discarded prototype; `Domain` is the standard answer.

### Known trap

A refcount-in-the-object *can* implement this trait and still be unsound: `protect` would
be `fetch_add` on the object, which is the announcement-is-the-dangerous-access bug from
blog Part 5. **The trait's safety docs constrain the client, not the implementor.** The
implementor needs its own soundness argument, which is what R4 and R5 are.

---

## R1 — `Leak`, and the restructure

### Why `Leak` first

It is the only implementor that can exist before the registry, and it gives the refactor
an acceptance test it otherwise would not have: `SegQueue<T, Leak>` must reproduce today's
behaviour exactly.

### The restructure, and why it belongs here

(A) is currently violated. `protect` is called on `&cur.next`, a write-once field, so a
retired segment stays permanently obtainable. Move both advances onto the moving cursors:

```
pop advance:   CAS head A→B, then protect(&self.head)    // not &A.next
push advance:  CAS tail A→B, then protect(&self.tail)    // not &A.next
```

The successor is still *read* out of `A.next` while `A` is shielded, and used as a CAS
argument — it is just never announced through. Root set becomes `{head, tail}`, and (A)
holds.

Falls out of it: shields-per-thread drops 2 → 1, because the overlap existed only to cover
a vacuous validate.

### Acceptance

- 9 std tests and 5 loom models green against `SegQueue<T, Leak>`
- Miri clean with `-Zmiri-ignore-leaks`
- A test asserting the chain **does** grow — `Leak` must be observably leaking, or the
  step proved nothing

---

## R2 — Treiber stack, before any real scheme

Ordering argument: a second *client* validates the trait's client-facing surface; a second
*implementation* validates the implementor-facing surface. The stack is ~80 lines, a
reclaimer is 200–400, so the cheap falsifier goes first. Discovering the trait is wrong
after writing two schemes against it is the expensive order.

The stack is also the contrasting case: a popped node genuinely *is* unlinked, so it
satisfies (A) the ordinary way. SegQueue satisfies it through the root-set argument. Having
both is what makes the contract legible.

---

## R3–R5 — the schemes

`notes/smr_inventory.md` marks every mechanism in folly's `hazptr` and in
`crossbeam-epoch` as core / important / optional / skip. Pick scope from the `core` rows;
treat the rest as a menu, not a checklist.

Specific things to derive rather than copy:

- **R4** — the announce/validate pair is a store followed by a load of a *different*
  variable on the same thread. `Release`/`Acquire` cannot order that. Loom should be
  trusted here: the handoff is store/load, not CAS, so the blind spot in
  `reference_loom_cas_blindspot` does not apply.
- **R5** — why **three** epoch generations and not two. Build it with two and let loom
  produce the counterexample.

Negative controls are mandatory at both (`reference_concurrent_test_checklist`). For a
scan-based scheme they bracket it from both sides: always-protected must stall reclamation
and fail a chain-length test; never-protected must produce a Miri UAF.

---

## R6 — the bench

The reason the trait exists. Blog Part 6 could not attribute the 6× gap to crossbeam
because crossbeam differs in **two** ways at once — layout *and* reclamation. Behind one
trait, with identical queue code, reclamation becomes the only variable.

Arms: `Leak` · `Hazard` · `Epoch` · `Mutex<VecDeque>` · real `crossbeam_queue::SegQueue`.
Shape: N producers → 1 consumer, N ∈ {1,2,4,8} — the shape both real callers have.

Seal the prediction first. The open question it answers: the previous measurement went
`7.7 → 77.8 → 105.9 → 104.9` ns against crossbeam's flat `13.8 → 16.7`. If `Epoch` lands
near crossbeam, reclamation was the whole story. If it stays 6× off, layout is implicated
and A3 becomes the interesting step rather than a formality.

Note before measuring: HP's per-protect `SeqCst` fence is the known cost, and folly's fix
(asymmetric barriers via `membarrier`) is **not available on macOS/aarch64**. Put that in
the prediction note so a bad HP number is explainable rather than mysterious.

---

---

## Phase 2 — the layout axis, and crossbeam's own technique

Phase 1 makes reclamation swappable, so a measurement can finally change **one** variable.
Phase 2 does the same for layout. This is also where crossbeam's actual reclamation scheme
gets built — which is *not* one of Phase 1's, and that is the point.

| Step | What you build | The wall that forces the next step | Est |
|---|---|---|---:|
| **L1** | **Layout A — a global index.** Replace the per-segment `claimed`/`consumed` pair with one global position (index + block pointer), the crossbeam family's shape. Reclamation held fixed at `Epoch`. | Bench `B-epoch` vs `A-epoch` is now **pure layout**: one permanently-hot contended line that never moves, against a cursor pair that migrates to each new segment and is cold on arrival. Whichever wins, the remaining gap to real crossbeam is neither layout nor reclamation — it is crossbeam's own scheme | 7 h |
| **L2** | **crossbeam-exact.** Block cursor, per-slot `WRITE`/`READ`/`DESTROY` bits, and the tunings. Then diff against the real source. | — endpoint | 11 h |

### Why L2 is a third reclamation technique and not a `Reclaim` implementation

Real `crossbeam_queue::SegQueue` does **not** use `crossbeam-epoch`. It self-reclaims: each
slot carries state bits, and the last thread to finish with a block frees it. The
participant set per slot is statically known and finite, which is what makes a cooperative
hand-off possible at all.

**That cannot be a `Reclaim` implementor.** The announcement is per-slot state *inside the
structure being reclaimed* — the same shape as the ordered walk from B1, and the same reason
`is_protected` cannot be in the trait. So L2 is deliberately outside Phase 1's abstraction,
and the teaching question it answers is: *crossbeam ships an epoch crate, so why doesn't its
queue use it?* Answer: this structure self-reclaims more cheaply than pin/unpin, and the
trait that unifies hazard pointers with epochs cannot express it.

---

## Reclamation techniques the finished plan covers

Five families, which is the "lose no technique" claim made concrete:

| Technique | Where | State |
|---|---|---|
| Leak / no reclamation | R1 `Leak` | baseline |
| Reference count inside the object | B1 | **done — proven unsound** |
| RAII guard + three free conditions + ordered trailing cursor | B1 | **done — all three survive** |
| Hazard pointers (announce addresses) | R4 | |
| Epoch (announce time) | R5 | |
| Per-slot cooperative hand-off (`DESTROY` bit) | L2 | not a trait impl, on purpose |

Deliberately out of scope, recorded in `notes/smr_inventory.md` so the decision is explicit
rather than an oversight: QSBR, RCU, hazard eras / IBR, Hyaline, Crystalline, VBR and the
optimistic-access family. VBR is the interesting one — it is the only family that drops
requirement (A) entirely, at the cost of a type-preserving allocator and a version word per
mutable field.

---

## Not in this plan

- **EventCount**, `CachePadded` and `Backoff` corrections — `concurrent`, not SegQueue.
  Independent; `PRODUCT_TREE.md` §9.
- The remaining folly gaps — `notes/folly_gap_analysis.md`, each picked up with its own
  artifact.
