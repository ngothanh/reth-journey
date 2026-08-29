---
name: blog
description: >-
  Produce a technical blog series (the reth-journey house style) from a finished build:
  a multi-part, problem-first design investigation in English + German, with EVERY code
  snippet and diagram rendered as a PNG (for beehiiv), an optional German A2+ Wortschatz
  (PDF + markdown), and understated low-latency-influencer launch posts (LinkedIn/X).
  Use when the user wants to write up / blog a primitive or product they just built
  (e.g. "blog the MPMC ring", "write the SeqLock series", "make the launch post"),
  or continue/extend an existing series under plan/blog/<topic>/.
---

# Blog series (house style)

The workflow that produced `plan/blog/async-semaphore/` and `plan/blog/seqlock/`. It turns a
finished build into a shareable series. **Blog posts are an output of finished work** — the
implementation conversation + measurement notes are the raw material; don't schedule work around them.

Reusable scripts live in `scripts/` next to this file. Series content lives in
`plan/blog/<topic>/`. Voice + style rules are also in memory: `feedback_social_post_style`,
`feedback_vietnamese_tech_terms`, `project_seqlock_blog`, `project_async_semaphore`.

Pick the stages the request needs — you rarely do all five in one go.

---

## Layout (per series)

```
plan/blog/<topic>/
  README.md              # bilingual entry: switcher + parts table
  en/00_index.md         # EN index (part summaries, how-to-read, scope, glossary)
  en/0N_*.md             # EN parts
  de/00_index.md de/0N_*.md   # German edition (same structure)
  wortschatz_a2.{md,pdf} # optional German vocab reference
  img/
    src/en/ src/de/      # per-language figure HTML (diagrams, charts, tables)
    src/cards/           # code + terminal cards (language-neutral; comments in English)
    en/ de/ cards/       # rendered PNGs
    cards.py cards_data.py assemble_wortschatz.py render.mjs render_pdf.mjs package.json
```

Set up rendering once per series: `cd plan/blog/<topic>/img && npm install` (pulls puppeteer;
downloads Chromium once). Copy the scripts from this skill's `scripts/` into that `img/`.

---

## Stage 1 — Author the series (EN)

Format: a **design investigation, not a tutorial**. Problem-first. Each part opens where the
previous stopped and **closes on the question the next answers**. "No decision falls from the
sky" — every choice is forced by a use case or by the simpler alternative failing.
~10–15 min/part. Consolidate to fewer, meatier posts (~10–12 min each) over many thin ones.

Each part file: `# Part N — Title`, prose with `![alt](../img/...)` figures, and a footer:
`*Next: [Part N+1 — …](0M_….md) · [Index](00_index.md)*` + `*Deutsch: [`../de/0N_….md`](…)*`.
The index (`00_index.md`) carries part summaries, a "How to read it", a Scope note, and a Glossary.
Study a prior series (`plan/blog/async-semaphore/en/04_fairness.md`) for the exact voice: ASCII
timelines, underlined decisions, comparison tables, a hook first line, one claim per figure.

## Stage 2 — Figures (the load-bearing part — everything is a PNG)

beehiiv needs images, so **every code snippet AND diagram is a rendered PNG**. Pipeline:
author self-contained HTML → screenshot via headless Chrome (`render.mjs`) at 2× DPR.

- **Code cards** (One Dark, Silicon/Carbon look): edit `cards_data.py` (see `cards_data.example.py`)
  → `python3 cards.py` writes `src/cards/*.html` → render each. Gate comments containing ①② render
  **amber (writer)**, ③④ **teal (reader)** — tie code to the ordering diagrams. Terminal cards
  (Miri/test output) use the same generator with a terminal variant.
- **Diagrams**: hand-author inline SVG in `src/<lang>/*.html` on the dark theme (tokens below), or
  reuse an approved artifact's `<figure>` SVGs and re-label per language. One figure, one claim;
  label the arrows; writer=amber, reader=teal, tear/hot=red.
- **Tables** as images too (for beehiiv consistency): a dark table-card (`src/<lang>/tbl_*.html`).
- **Charts**: compute coordinates in Python, emit an SVG line/bar chart on the dark theme (real
  measured numbers — the "money shot", e.g. the 450× read-scaling chart).

Render: `node render.mjs <src.html> <out.png>` (screenshots the `.fig` element). Preview a full
page with `shot_preview.mjs`. **Verify every `![](…)` path resolves** before publishing.

Optional companion **Artifact**: publish the ordering diagrams as an interactive page
(`Artifact` tool) for discussion; keep its source in `img/` and re-render its SVGs to the blog PNGs.

## Stage 3 — German edition

Run the translate→review workflow (one agent per file, EN→DE, then a native-German fidelity pass).
Rules: natural German technical register (match `plan/blog/async-semaphore/de/`); **keep in English**
all code + Rust terms + `Relaxed`/`Acquire`/`Release`/`fence`/`Pod`/`Miri`/`loom`/`MESI`/`payload`/
`cache line`. Rewrite image paths `../img/en/` → `../img/de/` (diagrams are localized; `../img/cards/`
stay shared with English comments) and translate alt text. Flip the footer to an `*English:*` backlink.
Diagrams: regenerate `src/de/*.html` with German labels and re-render.

## Stage 4 — German Wortschatz (optional)

An A2+ vocabulary reference (see `plan/blog/seqlock/wortschatz_a2.pdf`). Three steps:
1. **Extract** (workflow, one agent per part, extract→verify): each entry = headword (nouns with
   article; verbs infinitive), category, grammar meta (plural / Stammformen / `(+ Dat.)` / loanword
   gender), English meaning, a **verbatim** German example, and its English parallel.
2. **Full-translate** (workflow): re-translate each example into a COMPLETE equivalent English
   sentence (not a fragment) with the keyword bolded in BOTH languages — so a learner reads the
   English and maps it onto the German. `merge_trans.py` merges it back.
3. **Assemble**: `python3 assemble_wortschatz.py <data.json> out.html out.md` → 8 sections
   (Kernwortschatz, nouns, verbs, adjectives, idioms, discourse markers, loanwords, grammar patterns)
   → `node render_pdf.mjs out.html out.pdf`. Vendor the `<data>.json` for reproducibility.

## Stage 5 — Launch posts

Understated **low-latency-influencer register** (Rigtorp/Godbolt/matklad) — see
`feedback_social_post_style`. No preamble/hype/emoji; lead with a claim or a number; name the hard
nouns unexplained; carry ONE checkable credibility morsel; zero code; the link is the CTA. X = 2–3
short lines. LinkedIn = understated voice + light scannability. Write a **separate native-German post**
for the DE edition, not an "EN+DE" footnote.

---

## Design tokens

**Code cards — One Dark:** bg `#282c34`, bar `#21252b`, text `#abb2bf`; keyword `#c678dd`, type
`#e5c07b`, fn/method `#61afef`, number `#d19a66`, string `#98c379`, comment `#7f848e` italic;
writer-gate comment `#e5a04e` italic, reader-gate comment `#56b6c2` italic. Font JetBrains Mono.
Card = rounded window + traffic-light dots + filename bar.

**Diagrams / charts / tables — dark slate:** ground `#0b0f14`→`#141c26` radial; surface `#161D25`;
ink `#E3E9EF`; muted `#8C9BAA`; line `#2B353F`; **writer `#E8A44E`**, **reader `#4FBCC7`**,
**tear/hot `#EC7A70`**. Fonts: Archivo (labels), JetBrains Mono (mono), Source Serif 4 (Wortschatz body).

**Wortschatz doc:** serif body, italic-teal article on headwords, keyword highlighted in both the
German example (teal) and its English line (bold). Legend + counts header. A4, printBackground.

## Reproducibility & commit

Commit the series md + rendered PNGs + the vendored data (`wortschatz_data.json`) + the scripts, so
a future rebuild needs no re-run. Do NOT commit `node_modules`. Branch off `main` for the blog
(`git checkout -b <topic>-blog`) and commit stages as you go — machine switches have lost uncommitted
work before, so commit often.
