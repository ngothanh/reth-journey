export const meta = {
  name: 'plan-rework-w004-standard',
  description: 'Rework plan/WNNN.md files to the W004 trap-test standard via worker→coordinator fix-review loop',
  phases: [
    { title: 'Rework', detail: 'worker drafts/edits plan/WNNN.md to the W004 standard' },
    { title: 'Review', detail: 'coordinator gates against REVIEW_RUBRIC.md (hft-review + jeff-dean lens)' },
  ],
}

const ROOT = '/Users/thanhngo/tngo/projects/reth-journey'
const SPEC = `${ROOT}/plan/.rework/SPEC.md`
const RUBRIC = `${ROOT}/plan/.rework/REVIEW_RUBRIC.md`
let MAX_ROUNDS = 3
let SEEDS = {} // optional { "071": "coordinator feedback text to inject into round 1", ... }

const VERDICT = {
  type: 'object',
  additionalProperties: false,
  required: ['approved', 'score', 'blocking_issues', 'quality_notes'],
  properties: {
    approved: { type: 'boolean' },
    score: { type: 'number' },
    blocking_issues: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        required: ['section', 'problem', 'fix'],
        properties: {
          section: { type: 'string' },
          problem: { type: 'string' },
          fix: { type: 'string' },
        },
      },
    },
    quality_notes: { type: 'string' },
  },
}

function workerPrompt(w, feedback, round) {
  const file = `${ROOT}/plan/W${w}.md`
  let base = `You are a WORKER reworking ONE week file of a 144-week Rust/reth/HFT learning plan to the W004
gold-standard depth. This is round ${round} for week W${w}.

MUST READ FIRST (in this order):
1. ${SPEC}  — your complete operating manual (the trap-test arc, frontmatter, per-item rules, depth tiers, bar-c perf rules). Follow it exactly.
2. ${file}  — the week you are reworking. Preserve its frontmatter, day headers, Sunday ritual, and every [X]/[ ] checkbox state.
3. ${ROOT}/README.md sections: Bar policy (~lines 72-83), Inheritance Map (~699-828), APPENDIX E week→crate table (~1214-1311) — to set the Bar label, Mirror target, and Feeds-into correctly.
You MAY also open ${ROOT}/plan/W004.md (concurrency template), ${ROOT}/plan/W005.md (non-concurrency template), or ${ROOT}/plan/W008.md (ordinary Reth data-type Build at the right tier) for worked examples when a case is unclear.

YOUR JOB: rewrite ${file} IN PLACE so it meets the SPEC. The defining requirement: every Build/Extend/Refactor
exercise becomes a 3-phase arc whose Phase-2 tests are ENGINEERED TO TRAP a naive first attempt, and whose
Phase-3 failure cards walk naive→fix→final and teach transferable muscle with "Reapplies at: W.." cross-links.
A test a naive impl passes by accident is too loose — tighten it until it bites. Apply the correct depth tier
(SPEC §6) and, for bar (c) weeks, the two-track perf rules (SPEC §7) with real ns/cycle/alloc numbers.
Do NOT manufacture fake Build exercises for light career/ops/conference/interview weeks — for those, just bring
every item up to SPEC §3 (file path + Output line, cut filler, fold references) and add hardened-acceptance +
mental-model lines to learning/recall items.

Edit the file with Write (full replace) or Edit. Then your FINAL TEXT RESPONSE must be a short manifest:
week number, bar (b/c), the Build exercises you expanded, which of the three trap shapes (cross-primitive /
anti-shortcut / negative-control) you included, and any day-load-balance or filler-cut decisions. The manifest
is what the coordinator reads first — it is NOT shown to a human.`

  if (feedback) {
    base += `

THIS IS A REVISION. The coordinator REJECTED the previous draft (score ${feedback.score}/100). You MUST fix
every blocking issue below, then re-verify the whole file still meets the SPEC. Do not regress other sections.
BLOCKING ISSUES:
${feedback.blocking_issues.map((b, i) => `${i + 1}. [${b.section}] PROBLEM: ${b.problem}\n   FIX: ${b.fix}`).join('\n')}
Coordinator notes: ${feedback.quality_notes || '(none)'}`
  }
  return base
}

function reviewPrompt(w) {
  const file = `${ROOT}/plan/W${w}.md`
  return `You are the COORDINATOR grading a reworked week file. Apply the /hft-review (mechanical sympathy,
single-writer, zero-alloc, deterministic latency) and /jeff-dean (back-of-envelope first, the latency numbers,
design-for-failure) review lenses. Be a HARD grader — the learner's career plan depends on these files actually
building muscle.

⚠️ OUTPUT CONTRACT — READ FIRST: your ONLY deliverable is exactly ONE call to the StructuredOutput tool with
the verdict object. Do NOT write a prose review, summary, or any narrative text — that work is discarded. Read
the two files below, think briefly, then immediately call StructuredOutput once and stop. Always include a
score and quality_notes; blocking_issues is [] iff approved.

READ EXACTLY THESE TWO FILES (do not open W004.md or SPEC.md — the rubric is sufficient; reading more risks you
running out of turn before you emit the verdict):
1. ${RUBRIC}  — the rubric and blocking gates G1-G8. Judge against it exactly.
2. ${file}  — the reworked week you are grading.

The single most important gate (G4): for each Phase-2 test, ask "would a plausible naive first attempt PASS this
by accident?" If yes, the test fails to trap — reject with the exact tightening. Also verify: the week carries
all three trap shapes (cross-primitive, anti-shortcut, negative-control test-the-test) across its Builds;
failure cards name concrete symptoms + Reapplies-at links; bar-(c) weeks have two-track perf cards with
Cost-avoided numbers that are physically plausible; every work-item names a file + Output line with no filler.
For light career/ops/interview weeks, do NOT demand Build arcs — grade on rubric §"Calibration"/SPEC §3 only.

approved=true requires ALL blocking gates to pass (threshold score ~85). Every blocking_issue must be precise
and directly actionable: {section, problem, fix}. Now read the two files and call StructuredOutput exactly once.`
}

// Retry wrappers — an occasional throttled/StructuredOutput-missing agent should not fail the whole week.
async function reviewWithRetry(w, tag, attempt = 1) {
  try {
    return await agent(reviewPrompt(w), { label: `W${w} ${tag}${attempt > 1 ? ' retry' + attempt : ''}`, phase: 'Review', schema: VERDICT })
  } catch (e) {
    if (attempt < 4) { log(`W${w} ${tag} attempt ${attempt} failed (${String(e).slice(0, 60)}); retrying`); return reviewWithRetry(w, tag, attempt + 1) }
    throw e
  }
}
async function workWithRetry(w, feedback, round, attempt = 1) {
  try {
    return await agent(workerPrompt(w, feedback, round), { label: `W${w} draft r${round}${attempt > 1 ? ' retry' + attempt : ''}`, phase: 'Rework' })
  } catch (e) {
    if (attempt < 3) { log(`W${w} draft r${round} attempt ${attempt} failed; retrying`); return workWithRetry(w, feedback, round, attempt + 1) }
    throw e
  }
}

async function reworkOneWeek(w, reviewFirst) {
  let feedback = null
  let last = null
  // optional seed: inject a known coordinator rejection into round 1 so the worker fixes it up front.
  if (SEEDS[w]) feedback = { score: 83, blocking_issues: [{ section: 'PRIOR COORDINATOR REJECTION (seed)', problem: SEEDS[w], fix: 'Redesign per the problem; if the suggested fix is itself imperfect (e.g. a dt=0 boundary makes a twap-value assertion non-trapping), assert on a genuinely divergent property — sample membership/count/eviction — instead.' }], quality_notes: 'Seeded re-run: fix the issue below in round 1, do not regress other sections.' }
  // verify mode: review the existing file first; only enter the worker loop if it fails the bar.
  if (reviewFirst && !SEEDS[w]) {
    const v0 = await reviewWithRetry(w, 'verify')
    log(`W${w} verify: ${v0.approved ? 'APPROVED as-is' : 'needs work'} (score ${v0.score})`)
    if (v0.approved) return { week: w, approved: true, rounds: 0, score: v0.score }
    feedback = v0
  }
  for (let round = 1; round <= MAX_ROUNDS; round++) {
    await workWithRetry(w, feedback, round)
    const verdict = await reviewWithRetry(w, `review r${round}`)
    last = verdict
    log(`W${w} round ${round}: ${verdict.approved ? 'APPROVED' : 'rejected'} (score ${verdict.score})`)
    if (verdict.approved) return { week: w, approved: true, rounds: round, score: verdict.score }
    feedback = verdict
  }
  return { week: w, approved: false, rounds: MAX_ROUNDS, score: last?.score ?? 0, issues: last?.blocking_issues ?? [] }
}

// args: array of zero-padded week strings, OR {from, to, exclude:[], reviewFirst:bool}.
// args may arrive as an actual value OR as a JSON-encoded string — normalize first.
function pad(n) { return String(n).padStart(3, '0') }
let A = args
if (typeof A === 'string') { try { A = JSON.parse(A) } catch (e) { A = undefined } }
let weeks, reviewFirst = false
if (Array.isArray(A)) {
  weeks = A.map(x => pad(Number(x)))
} else if (A && typeof A === 'object' && Array.isArray(A.weeks)) {
  weeks = A.weeks.map(x => pad(Number(x)))
  reviewFirst = !!A.reviewFirst
  if (A.seeds) { SEEDS = {}; for (const k of Object.keys(A.seeds)) SEEDS[pad(Number(k))] = A.seeds[k] }
  if (A.maxRounds) MAX_ROUNDS = Number(A.maxRounds)
} else if (A && typeof A === 'object' && A.from != null) {
  const ex = new Set((A.exclude || []).map(Number))
  weeks = []
  for (let n = Number(A.from); n <= Number(A.to); n++) if (!ex.has(n)) weeks.push(pad(n))
  reviewFirst = !!A.reviewFirst
  if (A.maxRounds) MAX_ROUNDS = Number(A.maxRounds)
} else {
  throw new Error('Workflow args missing/invalid: pass {from,to,exclude?,reviewFirst?,maxRounds?}, {weeks:[],seeds?,maxRounds?}, or an array. Got: ' + JSON.stringify(args))
}
log(`${reviewFirst ? 'Verifying' : 'Reworking'} ${weeks.length} weeks to W004 standard: ${weeks.map(x => 'W' + x).join(', ')}`)
const results = await parallel(weeks.map(w => () => reworkOneWeek(w, reviewFirst)))
const clean = results.filter(Boolean)
const approved = clean.filter(r => r.approved)
const failed = clean.filter(r => !r.approved)
log(`DONE: ${approved.length}/${clean.length} approved. Failed: ${failed.map(r => 'W' + r.week).join(', ') || 'none'}`)
return { approved: approved.map(r => ({ week: r.week, rounds: r.rounds, score: r.score })),
         failed: failed.map(r => ({ week: r.week, score: r.score, issues: r.issues })) }
