# Archive — the week-by-week plan (reference only)

**This folder is no longer executed.** We do not work through it day by day.
The active plan is [`../PRODUCT_TREE.md`](../PRODUCT_TREE.md): jump straight into building a product,
take it from naive to production grade, and pull whatever is needed from here as it comes up.

## Why it is kept

It is the most complete inventory of advanced core techniques we have — every HFT and systems
technique worth learning, with its mirror, its bar, and the reasoning for why it matters. Losing it
would mean losing coverage. `PRODUCT_TREE.md` §6 maps each of those techniques to the product that now
owns it; **this folder is the detail behind every row of that table.**

## How to use it

Look things up here; do not schedule from here.

- **Building a component and want the depth treatment?** Find the week that covered it via
  [`INDEX.md`](INDEX.md) or the §6 table in `PRODUCT_TREE.md`, and read that week's file. The mental
  model, contract, named pitfalls, paper drill, and back-of-envelope cost model are all still valid.
- **Want to confirm nothing was dropped?** `PRODUCT_TREE.md` §6 is the checklist; this folder is the
  source it was built from.
- **Design rationale, bar policy, inheritance ratios, mirror targets:** [`../../README.md`](../../README.md)
  (still the v3 strategy document) and [`.rework/`](.rework/).

## Contents

| Path | What it is |
|---|---|
| `INDEX.md` | Week-by-week index, W1–W144, with per-phase framing |
| `W006.md` – `W144.md` | The unexecuted week files |
| `done/W001.md` – `done/W005.md` | The five weeks actually completed (Layer 0 + the start of `concurrent`) |
| `.rework/PAPER_DRILLS.md` | Paper-drill catalog — the by-hand derivations |
| `.rework/REVIEW_RUBRIC.md` | The G1–G11 rubric the plan was authored against |
| `.rework/SPEC.md`, `.rework/HYPERCORE_ADDITIONS.md` | Authoring spec and the v3.2 durable-infra additions |

## Still active, outside this folder

- [`../PRODUCT_TREE.md`](../PRODUCT_TREE.md) — the plan
- [`../concept_cadence.md`](../concept_cadence.md), [`../rebuild_ladder.md`](../rebuild_ladder.md) —
  spaced-repetition ledgers; orthogonal to the product axis, so they survive the switch
- [`../blog/`](../blog/) — shipped writing
