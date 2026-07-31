# Rework SPEC — bringing every `plan/WNNN.md` to the W004 standard

You are a **worker** reworking ONE week file of a 144-week Rust/reth/HFT learning plan so it matches the
depth of the gold-standard exemplar `plan/W004.md`. You write the file in place. A **coordinator** will then
review your work against `REVIEW_RUBRIC.md` and either approve it or hand you a list of blocking issues to fix.
The loop repeats until the coordinator approves. Aim to pass on the first round.

This SPEC is self-contained — you do NOT need to read all 3278 lines of W004. The pattern is distilled below,
with two short verbatim excerpts. You MAY open `plan/W004.md` (concurrency template), `plan/W005.md`
(non-concurrency template), or `plan/W006.md` for additional worked examples when a specific case is unclear.

---

## 0. The one thing that matters: the TRAP-TEST ARC

The learner (a 12-yr Java/Kotlin engineer transitioning to reth/HFT) asked for exactly this:

> "walk me through errors that I may have during the implementation so that I understand every piece of unsafe
> tuning code there without failed, and build me the muscle for future concurrency tuning."

So every `**Build**` / `**Extend**` / `**Refactor**` exercise must be a **three-phase arc** where Phase-2
tests are *engineered to trap a naive first attempt*, and Phase-3 walks from the naive failure → the fix → the
final correct implementation. A test the naive impl passes is too loose — tighten it until it bites. This arc
IS the deliverable. If a Build exercise lacks engineered trap tests + ordered failure cards, the rework failed.

---

## 1. Frontmatter (every week)

Keep the existing frontmatter block; ensure it carries ALL of:
```
> **Phase**: N
> **Month**: MN
> **Track**: Reth (primary) | HFT | Tempo | Multi | (career/ops label)
> **Hours target**: 30h/wk (M1-M18) | 40h/wk (M19-M34) | 30h/wk (M35-M36)
> **Mirror target**: <exact upstream module(s) this week mirrors — e.g. alloy-consensus::Receipt>
> **Crate(s) touched**: <crate vX → vY; [NEW] crates/foo/ when introduced>
> **Bar**: (b) | (c)   ← REQUIRED. bar (c) = HFT critical path + Layer-1 substrate; bar (b) = Reth/db/Tempo.
> **Feeds into**: <downstream weeks/crates that consume this week's output — W58 X, W77 Y>
> **References** (consult when stuck — not scheduled as cover-to-cover tasks): <specs/sections only>
> **v2 modifications**: <delta or "none">
```
Rules:
- **Bar** is mandatory. Determine it from README §"Bar policy" (lines ~72-83): bar (c) crates are
  `concurrent, time, bufpool, wal, recovery, txn, epoch-gc, bloom, runtime-thread-per-core, mmap-queue,
  matching-engine, ledger-deterministic, messaging-aeron, marketdata-kernelbypass, consensus-raft,
  consensus-bft, consensus-vsr`. Everything else (storage-trie, exec-vm, consensus-engine, lsm-core, mini-db,
  vector-db, all eth-* crates, Tempo) is bar (b). Career/ops/conference weeks: omit or mark `(n/a)`.
- **References**: fold any "read book/tutorial cover-to-cover" task into this line as a *lookup-when-stuck*
  pointer. NEVER schedule cover-to-cover reading as a task. Keep only spec sections a Build directly depends on.
- Mirror target + Feeds into come from README §"Inheritance Map" (~699-828) and §"APPENDIX E" (~1214-1311).

---

## 2. Day structure (every week)

Preserve the existing Monday→Saturday day headers and the Sunday ritual block. Preserve all existing checkbox
completion marks: `[X]` stays `[X]`, `[ ]` stays `[ ]`. You are EXPANDING content, never resetting progress.
Each day ends with a commit/log item. The Sunday block stays as the pointer to the Weekly Ritual Template.

**Day-load balance:** no single day may exceed ~5-6h of estimated work. Each item carries a `(~N min)` estimate.
If a day overflows, move items to the week's lighter days (or flag explicitly in a note — do NOT cut coverage to
fit the hour cap; the learner invests more time, coverage is non-negotiable).

---

## 3. EVERY work-item (Build-tagged or not) must carry

1. **File path(s)** it creates/edits, named explicitly and first. An item the learner cannot start without
   asking "which file?" is defective.
2. An **Output** line: the artifact produced + the exact command or observation that proves it done
   (e.g. `Output: notes/26_wal_design.md exists with R1–R7, D1–D6; cargo test -p wal wal_ green`).
3. A reason to exist: an upstream mirror AND a downstream consumer. An item with no file, no upstream module,
   and no downstream consumer is **filler — cut it or fold it into a real component.** This applies to
   observability/glue/load-test/"learn library X" items too: integrate the library into a reusable component,
   never bolt it onto throwaway test code.

Purely informational micro-tasks (Commit, Tag, PR-address) keep their one-liner + estimate; they don't need the
full treatment but should still be concrete.

---

## 4. The Build exercise template (the core of the rework)

Every `**Build**`/`**Extend**`/`**Refactor**` exercise expands to this shape. (Concurrency/atomics/unsafe/
lock-free/variance/macro exercises get the DEEPEST version; ordinary Reth data-type builds get a lighter but
still complete version — see §6 for depth tiers.)

**Two companion files govern this template:** `PAPER_DRILLS.md` (the by-hand reasoning-artifact catalog — every
Build's Phase 1 must carry a 🧮 Paper drill selecting from it, for ALL its concepts + domain skills), and
`../concept_cadence.md` (the spaced-repetition ledger — when authoring a W1–W70 week that sits in a concept's
drought window, fold in that concept's scheduled touchpoint, sized small, obeying the Inheritance Principle).

```
- [ ] **Build**: `crates/<crate>/src/<file>.rs` — <one-line what>. (~total N min across 3 phases)
  - 🔧 **Why this matters**: <upstream mirror> + the 2-4 specific traps + named downstream consumers (W.. , W..).
    State plainly that first-attempt impls fail ≥1 test below and the debug session is the lesson.

  #### Phase 1 — Problem, prediction, design (~N min, no code yet)
  - 🎯 **Pre-prediction prompt** — write sealed answers in `notes/NN_<name>_prediction.md`:
    - P1..P5: the primitive they'd reach for (exact import line), a predicted latency/size number, the
      predicted concurrent-first-call outcome, a derive-correctness prediction, AND P5 = "list every Phase-2
      test you predict your first attempt will fail." Compared to reality after tests run.
  - *Mental model*: what the component is + why it exists (1-3 sentences).
  - *Contract*: the signature-level interface — types, methods, trait bounds. NEVER the solution body.
  - *Back-of-envelope*: throughput × latency budget → implications, using Jeff-Dean numbers (L1 0.5ns, branch
    mispredict 5ns, L2 7ns, mutex 25ns, main mem 100ns, syscall 300ns, malloc ~50ns, float div 10-30 cyc).
    For bar (c): name the cost in ns/cycles/allocs and the cliff ratio (e.g. "cold 1µs vs cached 3ns = 300×").
  - **Requirements** R1..Rn — write in `notes/NN_<name>_design.md`: the precise contract clauses.
  - **Design walk-through** D1..Dn — answer in the design doc; no peeking at the upstream source until Phase 3:
    - Enumerate every primitive you could reach for, with a property table (Sync? latency? allocs? blocking?).
    - **Dx. Pre-mortem (load-bearing):** write the EXACT 4-5 line bad interleaving / failure trace. If you
      can't write the bad interleaving, you can't prove your design prevents it — that's the point.
    - **Dy. Anti-shortcut audit:** name the "obvious" std/crossbeam/alloc reach for each motivation and one
      line on why it's wrong HERE.
  - 🧮 **Paper drill** — write by hand in `notes/NN_<name>_drill.md` (the worked reasoning, distinct from the
    sealed prediction + the R/D design doc). Select 1–3 artifacts from `PAPER_DRILLS.md` matching EVERY core
    concept AND domain skill this Build teaches — not only memory ordering. Each names a CONCRETE instance:
    `- <D-XX> on <concrete instance, e.g. a 3-node tree / repr(C) struct / 5-record log>: <one-line task>.`
    Domain-knowledge builds get **D-WALK** (hand-trace one concrete input) and **D-LAYOUT** where a wire/disk
    format is involved. A Build that teaches a catalogued concept but omits its drill is a rework reject (G9).
  - **Output**: `notes/NN_<name>_design.md` with R1..Rn, D1..Dn, the bad-interleaving sketch; plus
    `notes/NN_<name>_drill.md` with the worked paper drills.

  #### Phase 2 — Write the failing tests (~N min)
  Each test is engineered so at least one common naive impl FAILS it. Annotate the skill tier it traps
  (NEWBIE/SENIOR/EXPERT). Include real `rust` code blocks (assert-level precision). The week MUST contain at
  least these three trap shapes across its Build exercises:
    - **Cross-primitive trap**: composes this Build with another from the same/earlier week — passes each in
      isolation, fires on the composition (e.g. CachePadded correct alone, false-shares inside a repr(Rust) struct).
    - **Anti-shortcut trap**: the obvious reach passes correctness but fails the BAR — latency budget, asm shape,
      syscall count, allocation count, false sharing. (Required for bar (c).)
    - **Negative-control "test-the-test"**: proves a bench/loom/trybuild actually catches what it claims — loom
      run takes ≥100ms, bench result lands in a physically-plausible band (e.g. 3e7–5e8 ops/s/thread), trybuild
      emits the EXACT expected error string.
  For concurrency: include a `#[cfg(loom)]` model test. For bar (c): include a criterion bench test with a
  layout/observation assertion so it fails fast if the bench measures nothing.

  #### Phase 3 — Make the tests pass, one failure at a time (~N min)
  Ordered failure cards, **newbie failures first, expert last** — the ladder the learner actually walks. Each card:
    - **(TIER) When `<test_name>` reports `<exact symptom>`:**
      - Symptom: the concrete failure (assertion text / loom output / miri report / perf regression / UAF).
      - Why: the root cause named precisely (not "be careful with atomics").
      - Fix: the concrete remedy, ≤1 sentence (may include a short corrected code fragment).
      - Cost avoided: <ns / cycles / allocs / cache-lines> — REQUIRED for bar (c) ("(bar (c) violation)" tag
        when the failure disqualifies the crate from its bar, not merely "slow").
      - Muscle: the one-sentence invariant the bug teaches + **Reapplies at: W.. <feature>, W.. <feature>** —
        concrete future weeks where the same lesson recurs.

  #### 5-year failure mode
  Write in `notes/NN_<name>_followup.md` after green: the future condition that breaks this design (hardware
  change, scale change, fork), the measurable TRIGGER, and the 1-line migration.
- [ ] Commit + log (~10 min)
```

### Verbatim excerpt — a Phase-2 trap test (from W004 CachePadded)
```rust
#[test]
fn cache_padded_pins_offset_in_struct() {
    #[repr(C)]
    struct Pair { a: CachePadded<AtomicU64>, b: CachePadded<AtomicU64> }
    let cache_line = if cfg!(target_arch = "aarch64") { 128 } else { 64 };
    assert!(std::mem::offset_of!(Pair, b) >= cache_line,
        "Pair.b at offset {} but cache line is {} — adjacent atomics will share a line",
        std::mem::offset_of!(Pair, b), cache_line);
}
```

### Verbatim excerpt — a Phase-3 failure card (from W004 CachePadded)
```
- **(SENIOR) When `cache_padded_pins_offset_in_struct` reports `Pair.b at offset 8` instead of ≥64:**
  - Symptom: assertion fails — align(128) correct, but offset_of!(Pair, b) is 8.
  - Why: the enclosing struct Pair is repr(Rust); Rust packs b at offset 8 ignoring the 120-byte tail pad.
  - Fix: add #[repr(C)] to every struct where padding is load-bearing.
  - Cost avoided: silent re-introduction of false sharing (30-70% regression); green cargo test, RED bench.
  - Muscle: when padding is load-bearing, lock the layout — repr(C) is non-negotiable. Reapplies at: W11 MPMC
    ring struct, W58 matching-engine order book, W65 Disruptor sequencer/cursor, W77 aeron term metadata.
```

---

## 5. Inheritance discipline (non-negotiable, from the curriculum principle)

Every exercise must be a **production-grade reusable component** that (a) mirrors a named upstream
alloy/reth/revm/Chronicle/Seastar/Aeron/TigerBeetle/Qdrant/Tempo module, (b) is built in a real workspace crate
at a named path, and (c) is consumed by a named later week. No throwaways, no "intentionally fail to compile"
programs, no "tiny X to discard," no exercises that exist only to teach syntax. If a Rust concept has no natural
upstream component, find a different concept-vehicle that does — the concept serves the artifact, not vice versa.

Do NOT re-build something an earlier week already fully built. If you find an `**Extend**` that re-builds a
prior week's component, scope it down to a genuine delta or cut it. Cross-link "Reapplies at" by week number so
the plan reads as a pedagogical arc.

---

## 6. Depth tiers (scale the treatment to the exercise)

- **Concurrency / atomics / memory-ordering / loom / unsafe / lock-free / variance / macro** exercises →
  DEEPEST. Full Phase-1/2/3, 5-6 named failure cards minimum for EBR/lock-free-skiplist/hazard-pointer-class
  primitives; 3-4 for load-bearing primitives (bounded MPMC, SegQueue); 2-3 for lighter primitives
  (CachePadded, Backoff, AtomicCell, Parker). Loom model + slow-negative-control mandatory. Two-track perf
  pitfalls (correctness AND measured-cost) mandatory for bar (c).
- **Ordinary Reth/alloy data-type builds** (Receipt, Log, RLP types, EIP structs, trie nodes) → full
  Phase-1/2/3 with engineered trap tests + ordered failure cards, but correctness-focused; perf pitfalls
  optional (bar (b)). See W008 Receipt/Log for the worked pattern at this tier.
- **Crate-scaffold items** (new Cargo.toml workspace member) → name the path, the mirror, the deps, the
  downstream consumers, an Output line. No 3-phase arc needed (nothing to test yet) but state what it will hold.
- **Light weeks** (decision gates, conference, ops, interview prep, career outreach, relocation) → these
  rarely contain Build exercises. Still apply §3 to EVERY item: name the file/artifact + Output line, cut
  filler, fold references into frontmatter. Learning/recall items get a `🎯 Hardened acceptance:` line (a
  concrete pass criterion) and a `*Mental model*` of why the item exists now (see W120 Saturday Tempo-recall as
  the worked pattern). Do not manufacture fake Build exercises for these weeks — depth ≠ padding.

---

## 7. bar (c) two-track perf requirement (HFT critical path + Layer-1 substrate)

For bar (c) crates, correctness pitfalls alone are insufficient — they don't catch per-poll `Box::pin`, float
in the hot path, cache-line straddling, a syscall in the inner loop, or false sharing. Every bar (c) Build must
include:
- a `*Back-of-envelope*` line up front (throughput × latency budget → implications, Jeff-Dean numbers), AND
- at least one `**Expect to hit (perf) #N — <failure> (bar (c) violation)**` card with the full
  Symptom/Fix/Muscle/Reapplies-at structure PLUS a **Cost avoided: <ns/cycles/allocs/cache lines>** line.
Apply the hft-review lens: mechanical sympathy (cache lines, MESI), single-writer principle, zero-copy,
static/zero-allocation hot paths, back-pressure as correctness, and (where applicable) deterministic-simulation
/ VOPR testing. bar (b) crates MAY include perf pitfalls but are not required to.

---

## 8. Output contract for you, the worker

- Edit `plan/WNNN.md` IN PLACE (use Write to replace the whole file, or Edit for surgical changes). Preserve
  frontmatter, day headers, Sunday ritual, and all `[X]`/`[ ]` checkbox states.
- Keep the week's existing crate/theme intent (from its current stub + README Appendix E). You are deepening,
  not redesigning the curriculum.
- Match the prose voice and formatting of W004 (the 🔧 / 🎯 / 🪤 emoji markers, `*italic*` field labels,
  `**bold**` card headers, fenced ```rust blocks, `(~N min)` estimates).
- When you finish, your final text response = a short manifest: the week number, bar, the Build exercises you
  expanded, the trap shapes you included (cross-primitive / anti-shortcut / negative-control), and any day-load
  or filler-cut decisions you made. This manifest is what the coordinator reads first.
