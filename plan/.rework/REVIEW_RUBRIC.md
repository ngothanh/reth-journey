# Coordinator REVIEW RUBRIC — gating a reworked `plan/WNNN.md`

You are the **coordinator**. A worker has reworked `plan/WNNN.md` to the W004 standard (see `SPEC.md`). Read the
reworked file and judge it against this rubric, applying the **/hft-review** (Martin Thompson — mechanical
sympathy, single-writer, zero-alloc, deterministic latency) and **/jeff-dean** (back-of-envelope first, the
numbers, design-for-failure) review lenses. You must be a hard grader: the learner's whole career plan depends
on these files actually building muscle. Return a structured verdict. Only `approved: true` when EVERY blocking
gate passes. When you reject, give precise, actionable fixes the worker can apply directly (section + problem +
exact fix), not vague notes.

## Calibration
- The gold standard is `plan/W004.md`. Open it for comparison when unsure what "good" looks like.
- Depth must match the exercise tier (SPEC §6) and the bar (SPEC §7). A light career/ops week does NOT need
  Build arcs — judge it on SPEC §3 (file + Output line per item, filler cut, references folded) and reject only
  for filler, missing files/Output lines, or scheduled cover-to-cover reading. Do NOT demand fake Build
  exercises for a conference/interview/relocation week. Conversely, a concurrency week with weak trap tests is a
  hard reject regardless of how polished the prose is.

## BLOCKING gates (any failure ⇒ approved: false)

G1. **Frontmatter complete** — Phase/Month/Track/Hours/Mirror target/Crate(s)/**Bar (b|c)**/Feeds into/
   References/v2. Bar label present and correct per README §"Bar policy". References contain no cover-to-cover
   reading task.

G2. **Every work-item names its file(s) + carries an Output line** (SPEC §3). Spot-check 100% of non-trivial
   items. Trivial commit/tag/PR one-liners are exempt. An item with no file, no upstream, no downstream
   consumer is filler ⇒ reject and name it.

G3. **The trap-test arc exists for every Build/Extend/Refactor** (SPEC §0, §4): Phase 1 (prediction + mental
   model + contract + back-of-envelope + R-list + D-list incl. a load-bearing pre-mortem interleaving +
   anti-shortcut audit), Phase 2 (failing tests with real assertions, tier-annotated), Phase 3 (ordered failure
   cards newbie→expert, each with Symptom/Why/Fix/Muscle/Reapplies-at), and a 5-year failure mode. A Build that
   is just a requirements list with no engineered trap tests + ordered failure cards is a hard reject.

G4. **Tests actually trap the naive solution.** This is the heart. For each Phase-2 test, ask: "would a
   plausible first-attempt implementation PASS this by accident?" If yes, the test is too loose ⇒ reject with
   the specific tightening. The week must contain all three trap shapes across its Builds: cross-primitive,
   anti-shortcut, and negative-control test-the-test. Verify the negative-control genuinely proves the
   bench/loom/trybuild catches what it claims (loom ≥100ms, bench in a physically-plausible band, trybuild's
   exact error string).

G5. **Failure cards walk naive→fix→final and teach muscle.** Each card names a concrete symptom (not "it might
   break"), the precise root cause, a concrete fix, and a **Reapplies at: W..** cross-link. Ordering is
   newbie→expert. Reject cards that gesture ("be careful with ordering") instead of naming the bug + symptom.

G6. **bar (c) two-track perf** (SPEC §7) — for bar (c) weeks ONLY: every bar (c) Build has a Back-of-envelope
   line with real numbers AND ≥1 perf failure card with a **Cost avoided: <ns/cycles/allocs/cache lines>** line
   and the `(bar (c) violation)` tag where apt. Apply hft-review: is the hot path zero-allocation? single
   writer? cache-line-aware? Are the latency numbers physically plausible against the jeff-dean table (L1
   0.5ns, L2 7ns, mem 100ns, mutex 25ns, syscall 300ns, malloc ~50ns)? Reject implausible numbers.

G7. **Inheritance discipline** (SPEC §5) — every exercise mirrors a named upstream module, lives at a real
   crate path, and is consumed by a named later week. No throwaways. No re-building a component an earlier week
   already shipped (flag duplication with the specific earlier week). Reject standalone "learn library X" items.

G8. **Day-load balance** — no day exceeds ~5-6h of `(~N min)` estimates. Checkbox `[X]`/`[ ]` states preserved
   from the original. Day headers + Sunday ritual block intact.

G9. **Paper drill present per concept** (SPEC §4, `PAPER_DRILLS.md`) — every Build/Extend/Refactor's Phase 1
   carries a 🧮 Paper drill block selecting catalog artifact(s) on a CONCRETE instance, covering EVERY concept
   AND domain skill it teaches (not only memory ordering). A Build that drills a catalogued concept but omits
   its by-hand artifact is a reject; name the missing drill. Domain-only weeks must still carry D-WALK (+
   D-LAYOUT for wire/disk formats).

G10. **Concept cadence honored** (`../concept_cadence.md`) — if this week sits in a drought window for a
   concept in the ledger, the scheduled small touchpoint is folded in (real component, not throwaway). Reject
   if a drought-window week ignores its due touchpoint.

## QUALITY signals (don't block alone, but note them; many minor misses ⇒ reject)
- Numbers are specific and correct (cliff ratios, sizes, alignments per target arch).
- Pre-mortem interleavings are concrete enough to be falsifiable.
- Voice/format matches W004 (emoji markers, italic field labels, fenced rust blocks).
- "Reapplies at" links point at real plan weeks doing the named thing.
- Mental models are crisp; contracts are signatures-only (no solution code leaked).

## Output (return via the structured schema)
- `approved`: true only if ALL blocking gates pass.
- `score`: 0-100 overall quality (approve threshold ≈ ≥85).
- `blocking_issues`: [] when approved; otherwise each = {section, problem, fix} — precise and directly
  actionable. Order by severity. Do not include nits here.
- `quality_notes`: brief non-blocking observations + what was done well.
