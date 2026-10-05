# `reclaim` — the modeling ladder

> **What this is**: seven sessions that build the *model* of safe memory
> reclamation, before any more of the crate gets written. The build ladder
> (`plan/reclaim_ladder.md`, 171 h) assumes the model is already in your head.
> It was not, and the C1 sessions kept stalling on it — so the model gets its
> own ladder.
> **Not code.** Each session ends in an argument you can make and an exhibit
> you can write, not a file that compiles.
> **Est**: ≈ 8 h, ~1 h per session.
> **Teaching mode**: the one that worked on `Retire` — intent in two sentences,
> then numbered questions you answer, then I attack the answers. Where you are
> wrong, you find out from an **exhibit that does not hold up**, not from me
> saying so first.

---

## How a session runs

1. **Intent** — two sentences on what this session is for. Nothing more.
2. **The questions** — three to five, in order, each answerable from what you
   already hold. You answer in your own words; I push until the answer is
   precise rather than nearly right.
3. **The exhibit** — one numbered trace, threads labelled, last line naming a
   failure. This is the session's real output.
4. **The gate** — a thing you can state unaided. Not "do you understand", but a
   specific claim you can defend.

Nothing is pre-explained. If a session's answer is in
`notes/07_unsafe_traits.md` or `plan/reclaim_ladder.md`, **do not read it until
after**; comparing afterwards is the feedback.

---

## The seven sessions

| | Session | Gate — what you can state unaided afterwards | Est |
|---|---|---|---:|
| **M1** | Why you cannot just free it | the two-thread trace for a use-after-free on a queue segment, from memory | 1 h |
| **M2** | The two halves of the proof | which half a scheme supplies, which half the structure supplies, and **why no scheme can supply (A)** | 1 h |
| **M3** | Roots | the root set of three different structures, and which of them can satisfy (A) at all | 1.5 h |
| **M4** | What a guard is | why hazard pointers and epoch disagree about what a guard *is*, and what that forces on a shared trait | 1 h |
| **M5** | How you know the grace period passed | the two answers — announce *addresses*, announce *time* — and what each costs per read and per reclamation | 1.5 h |
| **M6** | Who owns the garbage, and when it dies | why bounded garbage needs more than a count threshold | 1 h |
| **M7** | The model becomes a contract | re-derive the three traits from M1–M6, then diff against what the plan already recorded | 1 h |

M3 gets the extra half hour because it is the one that already stalled twice,
and M5 because it is where the two schemes stop looking alike.

---

## M1 — Why you cannot just free it

**Intent.** Everything else exists because of one race. Before any machinery,
be able to write that race down.

**Questions.** What does a reader hold while it reads? · At which exact
instruction does freeing become unsafe? · What is the *earliest* moment a
freeing thread could learn that a reader is there?

**Exhibit.** Two threads on `SegQueue`: one popping, one reclaiming. End the
trace at a dereference of freed memory.

**Gate.** The trace, from memory, with the instructions in the right order.
If you have to look it up, M1 is not done.

**Trap to leave standing.** Most first attempts put the free *after* the read
and conclude it is fine. The question is not whether one ordering is safe; it
is whether *every* ordering is.

---

## M2 — The two halves of the proof

**Intent.** The contract has two obligations and they belong to different
parties. Derive the split rather than memorise it.

**Technique** — this is the method from `notes/07_unsafe_traits.md` §6, applied
to the model instead of a signature. Write the sentence:

> "Dereferencing `p` through a guard after `retire(p)` is sound because ___."

Try to finish it. You cannot. **It fails in two different places**, and those
two places are the two halves.

**Questions.** Where exactly does the sentence fail the first time? · And the
second? · One of the two can never be supplied by a reclamation scheme no
matter how clever — which one, and what is the reason it is structurally
impossible rather than merely hard?

**Gate.** Name the two halves, say who owes each, and defend the impossibility
claim. "The scheme has never seen your data structure" is the shape of the
answer, not the whole of it.

---

## M3 — Roots

**Intent.** (A) says nobody may *obtain* `p`. Obtain it from where? Answering
that precisely is the whole session.

**Questions.** A thread cannot invent a pointer — so where do pointers come
from? · If a root lives *inside* an object, what follows for that object? ·
`pop` still reads `cur.next` after Q3 but no longer protects through it — why
is reading not the same as protecting? · Harris protects through `pred.next`,
a different atomic per node: what is its root set, and what does that imply?

**Exhibit.** Take `SegQueue` **before** Q3, where `protect(&cur.next)` was
live. Write the trace that shows segment `A` can never be freed. Then take it
*after* Q3 and show why `A` can.

**Gate.** Given `SegQueue`, a Treiber stack and a Harris list, state the root
set of each and say which can satisfy (A) unaided. One of the three cannot —
know which, and know what mechanism exists to fix it.

**Why this session is longer.** It stalled twice already. The sticking point
was not reachability; it was that a root inside a retired object keeps that
object alive forever. Start there.

---

## M4 — What a guard is

**Intent.** Both schemes hand you something you hold while reading, and they
mean completely different things by it. The difference decides what a shared
trait can express.

**Questions.** What does a *reclaimer* need to know to decide it may free `p`?
· Answer that for a scheme that records addresses, then for one that records
time. · How many pointers does one guard cover, in each? · Epoch can hand out
a pointer whose type says "valid while this guard lives" — why can hazard
pointers not express that at all?

**Gate.** Build the three-row table yourself — hazard, epoch, leak — with
columns *a guard is*, *protects*, *costs to acquire*, *costs per protect*. Then
say what the last column means for a retry loop.

---

## M5 — How you know the grace period passed

**Intent.** (B) is "readers eventually let go." A scheme has to *detect* that.
There are exactly two ways, and they are the two schemes.

**Questions.** A reader stores the address it is reading into a slot every other
thread can see: how does the reclaimer use that, and what is the cost per
reclamation round in terms of R retired objects and H hazard slots? · A reader
instead publishes "I started reading at time T": how does the reclaimer use
*that*, and what does it no longer need to know? · The announce-then-check
sequence on the reader side is a store followed by a load of a **different**
variable — why can `Release`/`Acquire` not order that pair?

**Exhibit.** The reader-side race in the address-based scheme: announce
arriving one instruction too late. This is the exhibit that forces the full
fence.

**Gate.** Describe both detection mechanisms and give each one's per-read and
per-round cost. Then say which of the two bounds its garbage, and why.

**Left deliberately unanswered.** Why epoch needs *three* generations and not
two. Do not look it up — C6 builds it with two and lets loom produce the
counterexample. That is the single best failure in the whole build.

---

## M6 — Who owns the garbage, and when it dies

**Intent.** A scheme that only frees when someone asks is not bounded. The
machinery that bounds it is most of what C2 and C4 are.

**Questions.** If freeing happens only inside `pop`, what does a queue nobody
pops do? · Count the cost of a reclamation round: why must the threshold scale
with the number of hazard slots rather than being a constant? · Batching
retires into runs of twenty: what happens to the nineteen sitting in a partial
run when that thread goes quiet? · Who pays for a round, and why is "whoever
crossed the threshold" a bad answer for a latency-sensitive caller?

**Gate.** Explain why a count threshold alone does not bound garbage, and name
what else is needed. Then say which of the two schemes has an *unbounded*
garbage failure mode and what structural property causes it.

---

## M7 — The model becomes a contract

**Intent.** Now the signatures are consequences rather than decisions. Derive
them, then check them against the record.

**Task.** From M1–M6 alone, write the skeletons: what must `Reclaim` have, what
must a guard have, what must a retirable object have. Methods and arguments
only, no bodies, no docs.

**Then** diff against `plan/reclaim_ladder.md`'s step-1 decisions — nine of
them, each with its reasoning — and against `notes/reclaim_journey.md`, which
records which first answers were wrong and what killed them.

**Gate.** For each place your derivation differs from the record, decide which
is right. A disagreement you can defend is a better outcome than a match.

---

## After M7

Back to `plan/reclaim_ladder.md`. C1.1 (`Retire`, `RetireLink`) and C1.2a
(`Root<T>`) are already built and both carry a `TODO(you)` where their tests
belong — M3 and M7 are what make those tests writable, so they are the first
thing to do on return.
