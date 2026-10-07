# `reclaim` — the journey

Append-only. One entry per decision, written **when it is made**, not
reconstructed afterwards. The blog answers *how*, not *what*: the value is the
route, including the turns that were wrong. A finished crate cannot be read
backwards into this — the near-misses leave no trace in the code.

Entry shape, every time:

```
## N. <the question>
**First answer** — what we were about to do.
**What killed it** — the argument, counterexample, or tool. Name who or what caught it.
**Decided** — the choice.
**Cost** — what was given up, or what it bought in hours.
```

Companion: `notes/07_unsafe_traits.md` holds the same knowledge distilled by
topic. This file holds the order it arrived in.

---

# Session 2026-10-05 — the C1 contract

Thirteen entries. The crate had one `mod sync;` line in it at the start and the
same at the end: all of this is design.

## 1. Does `protect` mint a guard, or retarget one you hold?

**First answer** — framed it as a counting problem: minting costs one
slot acquisition per pointer load, retargeting costs one per operation. `pop`'s
retry loop goes N → 1, a Harris traversal N → 2.

**What killed it** — the counting was true and was not the reason. Minting
**cannot express "stay protected across a retry" at all**: you either drop the
old guard first, leaving an unprotected window (under epoch, an `unpin`/`pin`
gap that anything carried across the loop boundary falls into), or hold both,
which is two live slots for one logical cursor — exactly the 2 → 1 reduction Q3
exists to buy.

**Decided** — retarget. `domain.guard()` once, then overwrite the announcement
in place.

**Cost** — one extra state to document: a guard owning a slot while announcing
nothing. Later turned out to be CORE in P1121, which distinguishes *no record*
from *an unassociated record* — three states, not two.

## 2. Is one trait over hazard pointers and epoch worth the abstraction?

**First answer** — worry about per-operation overhead.

**What killed it** — wrong axis. Monomorphisation makes dispatch free, and
epoch's `protect` compiles to the same load either way. The real tax is
**expressiveness, paid by the client**: a scheme-agnostic structure is written
to hazard pointers' constraints, which are strictly tighter, and the trait can
never carry epoch's `Shared<'g, T>` because hazard pointers have no notion of a
guard-lifetimed pointer.

**Decided** — keep it, and keep it thin — justified as a **measurement
harness**, not as production polymorphism. Nobody will swap schemes at runtime;
the six earlier bench rounds could not attribute a 6× gap because layout and
reclamation varied together, and one queue body behind one trait is what makes
reclamation the only variable.

**Cost** — the honest expectation that the trait serves SegQueue and the Treiber
stack and nothing else ever; Harris and the skiplist go to epoch's native API.
Precedent checked both ways: nobody who has picked a winner ships this
abstraction (crossbeam-epoch, haphazard, folly, seize are all single-scheme);
the libraries that do are the comparison-oriented ones, libcds and xenium. Added
a bench arm — epoch-through-trait vs epoch-native — to price it with a number
instead of an argument.

## 3. Should `protect` hide the retry loop?

**First answer** — three shapes laid out, with a lean toward `protect` looping
plus a `try_protect` escape hatch.

**What killed it** — the objection that a hidden loop is a non-deterministic
spin inside a library call, which is a **recurrence of a bug already fixed**:
`pop`'s check-then-claim removed exactly this. Sharpened past the original
objection: the two loops retry on *the same condition* (`src` changed) and only
the caller knows what the pointer was for, so the inner loop can only
re-stabilise a fact with a lifetime of zero instructions — **and it pays for it
in `SeqCst` fences**, the most expensive thing hazard pointers do. Cost composes
as inner × outer rather than inner + outer.

**Decided** — `try_protect` is the only required method; a looping `protect` is
a **provided default** written over it. One required method, no loop anywhere in
the trait.

**Cost** — nothing, and it resolved an earlier objection about widening a thin
trait.

## 4. Who marked the offload executor "skip"?

**First answer** — asserted that `notes/smr_inventory.md` had marked it skip.

**What killed it** — the question "who marked it as skip?". The inventory marks
it **IMPORTANT**, with the forcing pressure recorded verbatim as the
tail-latency spike that had just been re-derived from first principles minutes
earlier. The plan's not-ported list had demoted it, and claimed the inventory
said so.

**Decided** — audit the whole list. Six of its seven items were marked
IMPORTANT, not optional or skip. One mechanism it omitted — the hashed guarded
set — was **CORE and not listed at all**, and the plan's own argument #2 for a
scheme-owned retire list *is* O(R + H), which a linear scan does not deliver.
Two items were disclaimed in the opposite direction: `sync::list` and
`sync::queue` are built by this plan under other names.

**Cost** — **+35.5 h**, and the realisation that step 4 could not have seen any
of it: throughput averages a millisecond scan spike into "HP is a bit slower",
so the mechanisms that control it were neither justifiable nor, if omitted,
detectable.

## 5. Does `try_protect` promise freshness, or only protectability?

**First answer** — argued for freshness, on Harris, citing folly's
`try_protect(T*& ptr, …)` and haphazard's `Err`-carries-the-new-value as
precedent.

**What killed it** — "but can Harris implement the load themselves?" Yes. And
following that through: **freshness is not durable.** `Ok(p)` could only ever
mean "`src` held `p` at some instant after the announce"; by the time the caller
acts it may be stale again, and a mutating client's re-check *is* its CAS. Then
the precedent collapsed too — **both cited libraries are hazard-pointer-only,
and for hazard pointers the two contracts are indistinguishable**, since it must
re-read for safety either way. The citation could not discriminate and should
never have been offered.

**Decided** — protectability. Epoch returns `Ok(p)` unconditionally and does no
load; hazard pointers keep the re-read as part of their own safety argument, not
as a service to the caller.

**Cost** — none; it removed a load from epoch's hottest path that had nearly
been added to keep a test's shape tidy.

## 6. What does "runs against all three schemes unchanged" mean?

**First answer** — proposed making epoch validate so that client control flow
would be identical under every scheme, on the grounds that a retry path only
hazard pointers execute is a place a bug can hide.

**What killed it** — "then the acceptance test needs to be updated; the outcomes
need to be specific per implementation." Correct, and it exposes the conflation:
*unchanged* means **one source text — no `cfg`, no scheme-specific branch** — and
never meant identical runtime paths. Reading it the other way would have cost
epoch a load per protect to serve a test's shape.

**Decided** — a generic body carrying the scheme-agnostic invariants, plus three
thin wrappers carrying per-scheme outcomes: `retries == 0` asserted under epoch,
`retries > 0` under hazard pointers.

**Cost** — none, and it is the stronger test: "epoch's `Err` path is dead" is a
real claim that uniform control flow would have hidden.

## 7. We have batching — does that hurt tail latency?

**First answer** — separated the axes and showed batching amortises the push
while the scan spike is the executor's job, both already in scope.

**What killed it** — nothing, but the question found something neither audit
had: **an object in a partial batch is invisible to every reclamation round.**
Nineteen of twenty, or sixty-three of sixty-four, are garbage nobody will
collect. Thread exit (C3) and structure teardown (C11) cover two ways they get
stranded. The third — a thread that simply goes quiet — is covered by
`kSyncTimePeriod`, **IMPORTANT at inventory 459, and absent from both the plan
and the not-ported list.** A count threshold bounds garbage by *count*; a slow
retirer sits below it forever and, in the row's own words, "in a queue the
segments are never freed."

**Decided** — both triggers, count and time. Also recorded the sharp detail that
came with it: the retired count must be **signed**, because a round zeroes it
then subtracts what it reclaimed while others add, so it legitimately goes
negative and an unsigned counter wraps into a huge value and triggers runaway
reclamation.

**Cost** — +2 h, and the second silent absence in one session.

## 8. Why did two mechanisms go missing when an audit had just been run?

**First answer** — treated the first audit as thorough.

**What killed it** — the pattern. The guarded set was found by checking the
plan's own complexity claim; the time trigger by asking what batching leaves
stuck. **Neither was findable by the audit that was run, because it started from
the list of *claimed* omissions — which by construction cannot contain the
silent ones.** Two misses from a partial sample meant the real count was not
zero.

**Decided** — sweep from the inventory side and give **all 192 rows** a
disposition: `notes/smr_coverage.md`. Generated programmatically with the row
count asserted, so absence is impossible rather than unlikely.

**Cost** — **+12.5 h** across 25 further gaps, two of them soundness bugs
(announcing a tagged pointer word; no double-retire detection), one
plan/inventory contradiction (C7 requires a row the inventory marks SKIP, and
its stated justification — "what the bags need" — is wrong, since `Bag` is a
fixed array; the real consumer is the skiplist's tower). Three crossbeam rows
reclassified from omissions to a deliberate divergence, since C3 chose immortal
records.

## 9. What does `protect` return?

**First answer** — parked it deliberately: write C1 with the signature that
looks right and let C9's Harris set refute it under `trybuild`.

**What killed it** — the sweep, two sections later. `swap(a, b)` is **CORE**
(inventory 1054) and is how P1121's own traversal example advances two pointers.
A `&'a T` borrowed from `&'a mut self` cannot coexist with a swap, because the
swap needs `&mut` while the reference is live. It did not need to wait for C9.

**Decided** — raw pointer. Which also makes `try_protect` a **safe `fn`** —
announcing an address pins it and nothing more, so the whole unsafe surface
collapses into the dereference.

**Cost** — lifetime-scoped protection is the one thing the inventory credits
haphazard with over folly and P1121 (row 1283), and it is given up. Nothing now
stops a caller retargeting while holding a pointer derived from the previous
announcement; Miri in a client catches it, the type system will not.

## 10. Is `retire` required per scheme, or provided over a shared list?

**First answer** — asked the question as API placement, `Reclaimer` vs
`Domain<R>`, which is nearly cosmetic once `Domain<R> = Arc<R>`.

**What killed it** — the real question is underneath: the two schemes **do not
agree on what a retired list is.** Hazard pointers store intrusively with a
link on the object; epoch stores in bags of 64 with nothing on the object.
Hazard pointers decide per object and retain some; epoch decides per bag,
all-or-nothing, FIFO by seal epoch. A single representation has to pick a side,
and both choices reverse a decision already made for a stated reason — intrusive
costs epoch its bags and with them `Pointable for [MaybeUninit<T>]`; bags cost
hazard pointers the zero-allocation intrusive list that was reason #4 for
scheme-owned lists in the first place.

**Decided** — required per scheme. The shared surface is the **trigger
machinery** — thresholds, executor, flattener — because that decides *who pays
and when*, which is the same question for both. Not the storage.

**Cost** — none; and the falsifier is recorded in case it needs revisiting: name
one representation that keeps intrusive zero-allocation for hazard pointers and
contiguous 64-object bags for epoch.

## 11. Declared roots, or documented roots?

**First answer** — presented it as a genuine toss-up, conservative versus novel,
and leaned on the absence of precedent.

**What killed it** — "from the engineering side, I do not like to solve a
problem and guard it by document." Then the strongest counter-argument failed on
its own: a **test-only harness**, where the client registers its roots just for
the test and the production signature stays untouched, cannot work — because
**the bug *is* the divergence between the believed root set and the actual
protect sources.** The original queue believed `{head, tail}` while protecting
through `cur.next`; a check fed the believed set walks head and tail, finds the
retired segment unreachable, and passes. Any check that takes the root set on
trust is blind to exactly this bug class. The inverse — *observing* protect
sources at runtime — fails differently: an observed address can point inside an
object later freed, so the walk itself becomes a use-after-free.

**Decided** — declared. `try_protect` takes `Root<T>`; the domain holds the
roots' addresses and a debug-only `retire` walks them via C10's `for_each_link`
and asserts the retired object is unreachable. Against the original SegQueue the
assert fires. Nothing is locked out: `unsafe fn Root::assume_root` is the
documented regime, available per call site with an `unsafe` and a comment
instead of crate-wide by default.

**Cost** — +4 h, a production signature that cannot be retrofitted, and three
limits written down so a green assert never oversells itself: it catches
*permanent* reachability and not resurrection; `for_each_link` stays trusted
input; it is debug-only and claims nothing about production.

## 12. Should RCU be in scope?

**First answer** — it was already a bare name in the out-of-scope families list,
with no reason recorded.

**What killed it** — nothing; it was declined on merits, and the point is that
"already excluded" was not an answer. Three reasons: it occupies **the same cell
of the ERA lattice as epoch** (integration + applicability, not robustness), and
the five techniques were chosen to span the lattice; its headline property — a
~zero-cost read side — is **unmeasurable on this machine**, since userspace has
no preemption-disabled section and `membarrier` is unavailable on macOS/aarch64,
so the read side would come out epoch-shaped with worse garbage; and its failure
mode is unbounded garbage with blocking progress, which is the property C5 exists
to not have.

**Decided** — out, with the reasoning and two revisit triggers (a read-mostly
pointer with a named consumer, or a move to Linux). What RCU contributes anyway,
free: `synchronize_rcu()` **is** C4's `cleanup()`, and `call_rcu()`'s async half
is the offload executor.

**Cost** — none. And the interview answer is better than having built one.

## 13. `Retirable`'s five questions

Worked as a tutored sequence rather than a decision. Two beats worth keeping.

**The wrong turn** — asked why the trait must be `unsafe` to implement, the
answer given was about `*mut Self` offering no guarantee against concurrent
writers, with a check-before-and-after to detect them. Two things wrong, and the
second is the concept: **at `reclaim` time the grace period has passed, so
nobody is racing you by construction** — that is what the scheme bought — and
the observe/act/re-check instinct is `try_protect`'s protocol, which belongs on
the reader side. Carrying it into `reclaim` would have produced a check in C5
that cannot fail and means nothing. The missing concept underneath: **`unsafe fn`
constrains the caller; `unsafe trait` constrains the implementor**, and the
litmus test is whether a 100 %-safe impl could break safety in code that never
writes `unsafe`.

**The discovery** — asked for the exhibit showing a link whose address moves, the
answer reached for `UnsafeCell`, since interior mutability is the only way to
mutate through `&self`. Right mechanism, and it splits in two: mutating what
*contains* the link moves its address, while mutating the link's *contents*
corrupts the chain. **One mechanism, two violations, two clauses** — and the
second one, non-interference, had not been on the list. `retire_link` turns out
to promise four things, not one: ownership, determinism, stability,
non-interference, with stability and non-interference sharing one precisely
bounded window, from `retire(p)` until `reclaim(p)` returns.

---

# Session 2026-10-05 (cont.) — C1.1, `Retirable`

## 14. Should `reclaim` have a default body?

**First answer** — yes, `drop(Box::from_raw(ptr))`. It is the common case, and
making every implementor restate it is noise.

**What killed it** — the same argument that rejected `SegQueue<T, R = Leak>`.
A default is silently wrong for exactly the consumers the overridable `reclaim`
was designed for:

```
someone implements Retirable for a pool-allocated page
forgets to override reclaim
the default runs Box::from_raw on memory the global allocator never handed out
→ UB, on a reclaimer thread, minutes later
```

`bufpool` pages and P4 price levels are the two named consumers, and a `Box`
default points a footgun at precisely their case. **A bad default is worse than
no default, because it is silently wrong rather than loudly absent.**

**Decided** — no default. Every implementor states its own deallocator.

**Cost** — three lines per implementor, and it sidesteps the question of whether
`Sized` belongs on the method or the trait: `Box::from_raw` in a default body
does not compile, because `Self` in a trait is implicitly `?Sized`.

## 15. A clean build is not evidence that a trait is sound

**First answer** — the C1.1 gate was "`cargo build -p reclaim` clean", and the
first version built clean with `trait Retirable: Send`.

**What killed it** — nothing in the compiler. The trait was missing `unsafe`,
and **every exhibit written minutes earlier** — a shared `static` link eating
the chain, a branching getter leaking the tail, an `UnsafeCell` swap dangling a
stored pointer — was reachable by an implementor writing zero `unsafe`.

`unsafe fn` the compiler enforces. `unsafe trait` is a judgement, and the litmus
test is the only thing that catches it: *could a 100 %-safe impl break memory
safety in code that never writes `unsafe`?*

**Decided** — `pub unsafe trait Retirable: Send`, with the implementor's
obligations as the trait-level `# Safety` block and the caller's as `reclaim`'s.
Keeping those two blocks apart is the same distinction as the trait keyword.

**Cost** — none, but the gate was wrong. For anything with an `unsafe trait` in
it, "compiles and tests pass" is not a gate; the litmus test has to be applied
by hand.

## 16. `retire_link` promises four things, not one

Recorded because the fourth was not on the list when the review started.

**First answer** — two properties: the link must belong to this object, and it
must be the same link every call.

**What killed it** — asking for an exhibit where the link's *address* moves. The
answer reached for `UnsafeCell`, interior mutability being the only way to
mutate through `&self`. Right mechanism, and it **splits in two**: mutating what
*contains* the link moves its address, while mutating the link's *contents*
corrupts the chain. One mechanism, two violations, two clauses.

**Decided** — four properties: ownership, determinism, **stability**,
**non-interference** — with the last two sharing one precisely bounded window,
from `retire(p)` until `reclaim(p)` returns. Naming the window is what makes
them statable rather than vague.

**Cost** — none. The generalisable move is that an exhibit, not the clause,
is the unit of work: writing the trace is what revealed there were two.

## 17. A breakdown that lives only in conversation drifts

**First answer** — C1 was split into seven pieces in conversation, with
**C1.2 = `Root<T>` + `declare_root`**, and the split was never written down.

**What killed it** — `declare_root` takes `&self` on `Domain`, and `Domain` is
C1.3. So C1.2 could not contain it, and the code written for C1.2 had only
`assume_root` in it. From the outside that reads as the `declare_root` concept
having quietly vanished — it had not (it is in the plan in three places), but
nothing checkable said so.

**Decided** — the seven pieces live in the plan with their dependencies, and
their hours are asserted against C1's 7 h row. `declare_root` is C1.3.

**Cost** — none in hours. The lesson is the same one as entry 8: **a plan that
exists only in conversation cannot be audited.** The sub-step split was exactly
the kind of thing that felt too small to write down.

## 18. Decompose by idea, not by smallest compilable unit

**First answer** — C1 was split so each piece was the smallest thing that
compiles on its own. `Root<T>` and its accessors in C1.2; `declare_root` in
C1.3, because it takes `&self` on `Domain` and `Domain` did not exist yet.

**What killed it** — working on roots meant half of roots was somewhere else.
The split optimised for *compiles in isolation* when the property that matters
for learning is **finishes one idea in one sitting**.

And the dependency that forced it was not real. `declare_root` does not need a
whole `Domain<R>` — it needs a **list of root addresses**, and that list is
scheme-independent. A plain non-generic `RootRegistry` removes the dependency
entirely, after which `Domain<R>` is an `Arc` wrapper with a forwarding method
and drops from 2 h to 0.5 h.

**Decided** — eight pieces, each finishing a concept: the object side, roots,
the scheme contract, the reader surface, the handle, then something that
executes. The client-facing API does not change — `domain.declare_root(&self.head)`
is still what callers write.

**Cost** — none in hours. Two generalisable points. First, a dependency that
forces an awkward split is worth re-examining before accepting the split: here
it dissolved under one question, *what does this function actually need?*
Second, there is a difference between a decomposition that is correct and one
that is **teachable**, and the first does not imply the second.

## 19. The build ladder assumed a model that was not there

**First answer** — go straight into C1 and let the design decisions arrive as
they came up. Nine of them did get decided and recorded, and the code for C1.1
and C1.2a is sound.

**What killed it** — the sessions kept stalling on the *model*, not on Rust.
"What is a root" and "what is a Domain" are modeling questions, and they arrived
*after* code depending on the answers had been written. Three separate stalls in
one session, each one a sign that the build ladder was being executed against a
model that existed only in the plan document, not in the builder's head.

**Decided** — `plan/reclaim_modeling.md`: seven sessions, ≈ 8 h, before any more
of the crate. The race · the two halves · roots · what a guard is · how grace is
detected · who owns the garbage · then the traits as *consequences* rather than
decisions. Each session ends in an argument that can be defended and an exhibit
that can be written, not in a file that compiles.

**Cost** — ≈ 8 h, and it is not in the 171 h, which is implementation. Cheap
against three stalls in one session.

**Also decided, after being caught:** tests are the learner's work, not mine.
The existing memory classes unit tests as scaffolding Claude writes directly —
but the tests in `retire.rs` and `root.rs` were **exhibits turned into tests**,
which is the specific skill being taught. Writing them removed the exercise.
Both test modules are now a `TODO(you)` listing exactly which properties need
covering, including the two that cannot be tested single-threaded.
