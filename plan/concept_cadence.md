# Concept Cadence Ledger — spaced repetition for durable muscle

**Goal (learner-set):** every durable concept is drilled **≥5 times**, with **≥4 weeks** between
consecutive drills (no cramming) AND **no drought longer than ~12 weeks** (review before forgetting), across
**W1–W70** minimum. Reviewed/updated whenever a week is (re)authored.

**Method.** A concept is "drilled" in a week only if a task makes the learner *reason about or implement* it
(a paper drill from `.rework/PAPER_DRILLS.md` + a Phase-2 trap test counts; a passing mention does not).

---

## Current state (measured across W1–W72, 2026-06)

### Over-satisfied — leave alone (already 4–8× the target)
| Concept | Reps | Worst gap |
|---|---|---|
| Memory ordering / happens-before (C1) | ~38 | ≤3 wk |
| Atomics / CAS / coherence (C2) | ~23 | small |
| unsafe / UnsafeCell / raw ptr (C3) | ~24 | small |
| lifetimes / RAII guards (C5) | ~all | — |
| ownership / Drop / refcount (C6) | ~25 | small |
| Send/Sync (C7) | ~17 | 13 wk |
| lock-free structures (C8) | ~24 | small |
| loom (C9) | ~25 | small |
| cache-line / false-sharing (C12) | ~30 | small |
| microbench discipline (C13) | ~all | — |
| trait-design / type-state (C16) | ~all | — |

### Droughts — fix by injecting spaced touchpoints (these are the "I forgot W1–4" risks)
| Concept | Reps now | Original worst gap | Touchpoint reps (✓ landed through W48) |
|---|---|---|---|
| **async / Pin / Future (C18)** | 10+ | 40 wk (W3→W43) | W12✓, W20✓, W28✓, W36✓ + W43–44 natural (reactor/executor) — **drought closed** |
| **proc-macros (C15)** | 7 | huge | W15✓, W31✓, W39✓, W21 natural; W60 pending — **closed** |
| **declarative macros (C14)** | 8 | 27 wk | W17✓, W38✓, W21/W31 natural — **closed** |
| **futex / parking / blocking (C11)** | 15+ | 28 wk (W12→W40) | W14✓, W28✓, W36✓ + W40/W42 natural — **closed** |
| **miri / UB (active drill) (C10)** | 11 | 23 wk (W4→W27) | W12✓, W16✓ + W27/W33/W37/W43 natural — **closed** |
| **variance / PhantomData (C4)** | 11 | 21 wk (W12→W33) | W16✓, W33✓ (EBR `Shared<'g>`), W43✓ (`!Send` executor); W60 pending — **closed** |
| **manual-alloc / Layout (C17)** | 9 | 23 wk (W14→W37) | W22✓, W30✓ + W26/W28/W39 natural (mmap/slotted-page/footer) — **closed** |

> **All 7 W1–W70 droughts are now closed through W48** (Waves 1–4). The injected touchpoints + the natural reps
> the lock-free/storage/async weeks contribute mean every concept now has ≥5 reps with no >12-week gap across
> W1–W48. W20-overload was avoided by spreading touchpoints to fitting vehicles (futex→W14, variance/miri→W16,
> decl-macro→W17, async→W20, undo-pool→W30, MPT-derive→W31, async/futex→W36, decl-macro→W38, proc-macro→W39).
> Only **W60** (proc-macros + variance reps) remains, in the W49–W72 range. The no-code weeks (W25, W45, W46,
> W48) carry no drills by design.

---

## Touchpoint injection plan (drought fixes)

Each touchpoint must obey the **Inheritance Principle** — a real reusable component, never a throwaway. Proposed
vehicles (refine when authoring the week):

- **C18 async/Pin** — bridge W3→W43:
  - W12: `bufpool` async checkout — a `Future` that resolves when a buffer frees (Pin/Waker drill).
  - W20: trie-walker as an async `Stream` over nodes.
  - W28: B-tree async cursor (`poll_next` over leaf pages).
  - W36: snapshot async export (backpressured `Future` writer).
- **C15 proc-macros** — W15 (`#[derive(RlpEncodable)]` on EOF structs), W31 (`#[derive]` on MPT node types),
  W45 (derive on storage-trie records), W60 (derive on matching-engine messages).
- **C14 decl-macros** — W20 (`trie_node!` const builder), W45 (a `bench_case!` macro for the perf harness).
- **C11 futex/parking** — W20 (a `bufpool` blocking-acquire on exhaustion), W28 (B-tree page-latch wait),
  W36 (snapshot import barrier wait).
- **C10 miri** — make miri an *active* drill (not just CI): W12 (bufpool UnsafeCell slab under miri),
  W20 (trie unsafe node access under miri).
- **C4 variance/PhantomData** — W20 (a lifetime-branded `TrieCursor<'tx>`), W45 (branded storage handle),
  W60 (a `PhantomData`-tagged price-level marker).
- **C17 manual-alloc** — W22 (a small arena for stage buffers), W30 (recovery undo-record pool).

> These are **injections into existing weeks**, sized small (~30–60 min each), added as a secondary Build or an
> Extend on that week's main component. They do not displace the week's primary deliverable.

## Tier-2 advanced concepts (W73–W144) — dense, not drought-prone

The distributed/HFT muscles (their paper drills are D-QUORUM, D-VOPR, D-LIN, D-DET, D-CAP, D-FAIL, D-STALE) are
**concentrated** in the back half, so the risk is a *cold start*, not a drought. Each gets an early first rep
in W56–W72, then recurs densely through the consensus + capstone + BFT-apex arc:

| Tier-2 concept | Drill | First rep | Recurs (dense) |
|---|---|---|---|
| Consensus quorum/view-change | D-QUORUM | **W56 ✓ landed** (raft scaffold) | W59✓,62,65,68,72,91,118,120-128,143 |
| Deterministic simulation / VOPR | D-VOPR | **W60 ✓ landed** (det-sim harness) | W63,72,88,108,129-134,143 |
| Linearizability / single-log | D-LIN | **W59 ✓ landed** (raft replication; earlier than projected W72) | W72,91,103,108,117 |
| Cross-replica determinism (money) | D-DET | **W60 ✓ landed** (order-book match; earlier than projected W66) | W66,69,71,91,103,108,143 |
| Distributed capacity / tail | D-CAP | W58 (LMAX topology) — *not yet drilled as D-CAP; W58 used D-LAYOUT/D-COST* | W96,103,105,108 |
| Fault-tolerance / blast-radius | D-FAIL | W67 (raft snapshot) | W72,105,108,143 |
| Staleness contracts | D-STALE | W91 (ConsensusBackbone) | W103,108,117 |
| Interface swap-safety | D-SEAM | **W47 ✓ landed** (ConfigureEvm seam) | W91,103,117,135 |

So the Tier-2 cadence is healthy (≥5 reps, well-spread) **provided** the W56–W72 first-reps land — those weeks
are partly v3-superseded, so when they're JIT-regenerated, the SPEC's G9/G10 ensure the Tier-2 drill is folded
in. No early-injection touchpoints are needed for Tier-2 (unlike the W1–W70 droughts above).

## Maintenance
When (re)authoring any week W1–W70, check this ledger: if the week sits in a drought window for a concept above,
fold in its touchpoint. Update the reps/gap columns after each authoring pass. For W73+ (JIT-regenerated), the
`.rework` SPEC enforces the Tier-1 + Tier-2 drill checks (G9/G10) at generation time. This ledger now covers the
**full 144-week plan**: W1–W72 measured (above), W73–W144 governed by the SPEC at regeneration.
