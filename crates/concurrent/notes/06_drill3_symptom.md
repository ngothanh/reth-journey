# Drill #3 symptom card — Dekker two-flag mutual exclusion

## The three sentences

**1. Symptom.** With `Release` on each flag store and `Acquire` on each flag peek, two
threads each entering the critical section 1,000,000 times produced a counter of
**1,471,350 instead of 2,000,000 — 528,650 increments lost (26.4%)**, reproducibly.
Loom named it directly rather than statistically: `Causality violation: Concurrent write
accesses to UnsafeCell`, `write one: thread #0 @ dekker.rs:206`,
`write two: thread #1 @ dekker.rs:201`. Both threads observed the other's flag as
**still-false** and entered together.

**2. Root cause.** The missing edge is not between the two threads — it is *within each
thread*, between **its own `Release` store and its own `Acquire` load**. `Release` stops
earlier operations sinking below the store; `Acquire` stops later operations rising above
the load. Both barriers point **away from the gap between them**, so nothing orders the
store against the load. That pair is **StoreLoad**, the one combination Release/Acquire
never covers. Concretely on aarch64: `Ordering::Acquire` lowered to **`LDAPR`** (RCpc),
which ARM explicitly permits to reorder ahead of an earlier `STLR`; `Ordering::SeqCst`
lowers to **`LDAR`** (RCsc), which it does not. One letter of one mnemonic — see
[`06_drill3_dekker.asm.txt`](06_drill3_dekker.asm.txt).

**3. Rule.** **Release-Acquire pairs do not prevent StoreLoad reordering.** If your
invariant needs "my write is globally visible before I read yours," Release/Acquire is
not enough and no amount of adding more of them will help — you need SeqCst, either a
`fence(SeqCst)` between the two operations or `SeqCst` on both.

## Where the fence goes, and why

Don't guess the placement — derive it. SeqCst fences form a single total order. Assume
both threads entered:

```
T0:  store flag0=true  →  F0  →  load flag1 = false
T1:  store flag1=true  →  F1  →  load flag0 = false
```

WLOG `F0 < F1`. Then `store flag0=true` is sequenced before `F0`, `F0 < F1`, and
`load flag0` is sequenced after `F1` — so the store precedes the load and T1 **must** read
`true`. Contradiction. ∎

The proof tells you the three placement rules:

| The proof needs | So the fence must be |
|---|---|
| store sequenced-before fence | **after** the store |
| load sequenced-after fence | **before** the load |
| two fences to order against each other | on **every** thread running the pattern |

A fence on one side only is not half a fix; it is no fix.

## Method for next time

1. Write both threads' operation sequences.
2. Name the ordering the invariant needs.
3. Classify the pair: LoadLoad / LoadStore / StoreStore / **StoreLoad**.
4. First three → Release/Acquire suffice. StoreLoad → SeqCst, no exceptions.
5. Place strictly between the pair, on every participating thread.

## Two secondary rules this drill also taught

**A green loom run is a proof about loom's model, not about the hardware.**
`DekkerSeqCst` — a *correct* implementation — fails loom with the same causality
violation. Loom gives `fence(SeqCst)` a global clock (`rt/thread.rs::seq_cst_fence`,
whose only caller in the whole crate is `rt/atomic.rs::fence_seqcst`), but SeqCst *loads
and stores* never join it. Dekker needs the total order across two *different* locations,
which loom does not build for plain SeqCst ops. Knowing which of a model checker's
answers to trust is part of using one.

**An SC barrier is free until it is contended.** Uncontended, all three variants cost
1.78–1.80 ns — adding a `dmb ish` costs ~4 picoseconds. Contended, the fenced variant
costs ~103 ns. The barrier's price is the cross-core traffic it serialises, not the
instruction. Corollary: **the fix for an expensive barrier is almost never a cheaper
barrier — it is removing the sharing.** Full numbers in
[`06_drill3_numbers.md`](06_drill3_numbers.md).

## Reapplies at

- **W74 — VSR quorum-ack counting.** Every Prepare/PrepareOk carries a "have N replicas
  acked?" check that is structurally Dekker-shaped: set my state, peek at everyone else's.
  The lesson to carry is not "add a fence" — it is the corollary above. Per-replica ack
  slots indexed by replica-id have no shared flag, so there is no traffic to serialise and
  **no SC needed at all**.
- **W83 — ledger 2PC vote.** Same shape: each participant publishes its vote, then reads
  the others' to decide commit. Same trap, same escape route.
- **Anywhere two threads each write their own location then read the other's.** That
  crossed store-then-load is the fingerprint. If you see it, StoreLoad is live.
