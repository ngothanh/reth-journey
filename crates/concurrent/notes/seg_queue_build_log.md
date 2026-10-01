# `SegQueue` build log — the failure sequence

> **TEMPORARY FILE.** Raw material for the blog series only. Kept up to date while
> the build is in progress; **delete once the blog is finalised.** Everything here
> that belongs in permanent documentation has already been written into the module
> docs, `seg_queue_bench_results.md`, `backoff_bench_results.md`, or the commit
> messages — this file exists to preserve the *order in which things went wrong*,
> which is the part that normally evaporates.

Machine: Apple Silicon (aarch64).

**Authorship, because this is blog source material and it matters.** The `SegQueue`
implementation is the user's: `push`, `pop`, the boundary protocol, the slot state flag,
`SegGuard`, the refcount and the free conditions were all written by him. Claude wrote
the criterion harness, the loom test bodies, and two refactors done on explicit request
(the local-cursor rewrite of `push`, and the trailing `reclaim` cursor). Where this log
says "the user caught X", that is literal; where it records a wrong hypothesis about
false sharing or a mis-aimed benchmark, that was Claude's.

---

## 0. Brainstorm — framings worth keeping

**Problem statement.** A fan-in queue whose capacity cannot be known in advance,
because it is a function of *consumer latency*, not of workload. The motivating
consumer: WAL group-commit, where N request threads append and exactly one flush
thread calls `fsync` (~500 µs). A bounded ring forces you to pick `N` at
construction, and then answer: what happens to producer N+1 when `fsync` stalls
for 10 ms? Too small and every request thread blocks behind the disk; sized for
peak burst and you pre-pay RAM for a rare event — and there is always a larger
burst.

**"Unbounded" does not remove backpressure, it changes the failure mode from
*block* to *OOM*.** What the structure actually does is separate two things a
bounded ring conflates: storage capacity (the data structure's job) and admission
policy (the caller's job, e.g. a `pending_count` soft cap).

**Single responsibility, settled early and load-bearing for the rest of the
build.** The queue owns **rate** — temporal decoupling of producer and consumer.
It does *not* own:
- **order** beyond enqueue order. There are two orderings and they diverge:
  ticket order (`next_lsn.fetch_add`) and physical push order. A producer can take
  LSN 42, be preempted, and push after the producer holding 43 — so the queue's
  FIFO is push-order, not LSN-order. Application order belongs to the LSN.
- **admission** — that is the caller's soft cap.
- **durability** — WAL + fsync + a `durable_lsn` watermark.

A user question that sharpened this: *"does the consumer even need FIFO, or just a
bag?"* Answer: order almost always matters (payment: debit-then-credit ≠
credit-then-debit), and ordering that matters for correctness can never be
established by racing producers anyway — it must be fixed *before* the concurrency
point.

**Why segments rather than one node per element (Michael–Scott).** MS-queue is
correct and solves the same problem. Segments buy four amortisations, measured or
reasoned:
1. alloc count: 1 malloc per 32 pushes instead of per push, and malloc touches a
   *shared* allocator across cores
2. cache locality: contiguous slots vs a pointer chase per element (~100 ns miss),
   and pointer chasing is address-dependent so the prefetcher is blind
3. CAS *quality*: `fetch_add` (one instruction, no retry) vs a CAS retry loop that
   degrades with contention
4. reclamation granularity: 32 elements freed as one unit
Cost: wasted space when near-empty, plus a genuinely hard boundary race.

**Segment ≠ ring.** User's own insight, and it was right: slots are written once
and read once, never reused, so no wrap-generation sequence counter is needed — a
one-shot flag suffices. A ring needs `seq` *because* it reuses slots.

**The reclamation circularity**, walked through before any code (this became the
spine of the R1→R3 ladder):

```
C2: load head  → raw pointer p to seg k
C1: drains last slot, refcount 1→0, frees seg k
C2: p->refcount.fetch_add(1)   ← bumping a counter inside freed memory
```

The counter that is supposed to tell you "this memory is live, go ahead and touch
it" **lives inside the memory you are not yet sure is live**. Provably not fixable
from the inside, which is why every real solution (hazard pointers, EBR) puts the
"I am reading this" signal in per-thread state *outside* the object.

**Correction made during brainstorm:** real crossbeam `SegQueue` does **not** use
`crossbeam-epoch`. It self-reclaims blocks via per-slot WRITE/READ/DESTROY bits.
`crossbeam-epoch` is for `SkipList`. The teaching question that falls out: crossbeam
*has* epoch — why not use it for the queue? Because the queue's structure lets it
self-reclaim more cheaply than epoch's pin/unpin.

---

## 1. Struct modelling — the questions asked

Good questions, in the order they came, each of which closed a real gap:

- **"Why does a slot need `state`?"** Because `push` is two steps: `fetch_add` to
  reserve index `i`, then store the value. A consumer computing the same `i` can
  arrive *between* them and read `MaybeUninit::uninit()`. `state` is the only thing
  that distinguishes "reserved" from "written".
- **"Why a pointer and not a reference to `next`?"** A `&Segment` carries a
  lifetime the compiler must prove; the successor is heap-allocated at runtime,
  linked concurrently, and eventually freed by a scheme the compiler cannot see.
  Also `next` is null until a boundary is crossed, and `&T` is never null.
- **"Why `AtomicPtr` and not plain `*mut`?"** `next` is written and read
  concurrently (data race), the boundary needs `compare_exchange` so exactly one
  producer links the successor, and loom cannot model a plain `*mut`.
- **"Why `*mut` and not `*const`?"** `AtomicPtr<T>` is defined over `*mut T`; the
  pointee gets mutated and eventually freed through `Box::from_raw`.
- **"Should it be `claimed`/`consumed`, not `claim`/`consume`?"** Yes — the field
  holds a *count*, so the noun form is right. (Also: do not name them `head`/`tail`,
  those are already the segment pointers one level up.)

One correct subtle choice made unprompted: `unsafe impl<T: Send> Sync`, not
`T: Sync`. The queue never hands out `&T` across threads — it moves `T` in and out —
so `T: Send` is the right bound. Matches crossbeam.

---

## 2. `push` — four iterations

**v1.** Used `consumed` (the *consumer's* cursor) for the producer; created a fresh
`Slot::new()` and assigned it over the existing slot before writing; never
published `state`; never returned, so the success path looped forever and the
borrow checker rejected the second move of `value`.

The move error was the useful one: *a successful write must be the last thing the
function does, because `value` is consumed.* The type system was describing the
control flow.

**v2.** Fixed the write form and added `state` + `return`, but still initialised
the loop from `consumed`, and still overwrote the slot with `Slot::new()` —
a non-atomic write to memory a consumer may be reading, which also reset the
`state` atomic mid-flight.

**v3.** Attempted the boundary. Encoded several broken ideas at once:
- `break` on finding `next` already linked → the function returned having pushed
  nothing, silently dropping `value`. Should have been `continue`.
- `self.tail.store(next)` then `(*cur_seg).next = AtomicPtr::new(next)` — blind
  non-atomic writes, so N overflowing producers each allocated a segment and raced
  to clobber both fields.
- `cur_seg = (*next).next.load(...)` — read the *successor's* successor (null) and
  then dereferenced it.
- `idx = 0` — self-assigned slot 0 of the new segment without claiming it, so
  every overflowing producer would take slot 0.
- `> SEG_LEN` instead of `>= SEG_LEN`.

**v4 (correct).** Boundary protocol: read `next`; if null, allocate and
`compare_exchange(null → mine)`; winner uses its own, loser frees its spare and
uses the winner's; nudge `tail` forward with a CAS; `continue` and re-claim.

A user question here was sharper than my own description: *"advancing `tail` doesn't
solve anything for this producer — it has to go to `next` and claim again."*
Correct, and I had conflated two concerns: the producer's own progress (must claim
on `next`) and the courtesy tail advance (reduces other threads' spinning). That
distinction produced the local-cursor refactor.

**v5 (refactor, written by me at user's request).** Local cursor: load `tail` once,
walk forward with your own pointer, `fetch_add` instead of a CAS loop, boundary
extracted into `advance_tail`. The user then asked for all orderings reverted to
`Relaxed` — *"I have to learn memory ordering myself"* — which turned out to be the
right call, because the two Miri failures below became his to derive.

User question worth preserving because the answer is a genuine semantic gap, not a
slip: **"with `fetch_add`, who gets index 0?"** The caller does — `fetch_add`
returns the *previous* value and then adds. That is what "fetch" means.

And the right follow-up: **"isn't blind `fetch_add` also wrong in push, since the
counter grows past `SEG_LEN`?"** No, and the asymmetry is the interesting part: a
producer's wasted ticket costs nothing because the item is not bound to the ticket —
it travels with the producer to the next segment. A consumer's wasted ticket
*orphans a specific slot*. Same primitive, opposite consequence.

---

## 3. `pop` — the data-loss bug

**v1** mirrored push: blind `consumed.fetch_add`, then if `state != WRITTEN` return
`None`. This loses data, and the trace is the single best teaching artifact of the
whole build:

```
push(A)   → claimed=1, slot0 WRITTEN
pop()     → fetch_add → 0, state[0]=WRITTEN → returns A.  consumed=1
pop()     → fetch_add → 1, state[1]=EMPTY   → returns None. consumed=2   ← index 1 BURNED
push(B)   → claimed.fetch_add → 1 → writes slot1, WRITTEN
pop()     → fetch_add → 2, state[2]=EMPTY   → None
```

**B is in slot 1, `consumed` is already past 1. B is lost forever.**

Two things were conflated: a consumer that finds `state == EMPTY` might be looking
at a slot *no producer has claimed* (genuinely empty) or at one *a producer owns and
is mid-write* (not empty — an item is coming). Returning `None` in the second case is
wrong, and committing `consumed` in the first case is fatal.

**v2** added a `claimed` check but kept the blind `fetch_add` *above* it, so the
index was already burned by the time the check ran. Also `continue`d on
`state != WRITTEN`, which re-ran `fetch_add` and burned a *fresh* index per spin.

**v3** off-by-one argument. I said empty is `c >= claimed`; the user argued for
`c > claimed` — *"`c == claimed` means a producer is still writing"*. The resolution
is count-vs-index: `claimed = p` means tickets `0..p-1` have been issued, so the
highest claimed index is `p-1`. The producer that may be mid-write sits at `p-1`,
which `c < p` already covers. `c == p` is the first *unissued* index, i.e. empty.

**v4 (correct).** Conditional claim: load `c`, bail if `c >= claimed` without
touching `consumed`, then `compare_exchange(c → c+1)` to commit, then wait for
`WRITTEN`. The asymmetry stated plainly: **push always succeeds so blind
`fetch_add` is fine; pop can be empty so it needs a conditional CAS.**

**v4 bug, caught by inspection, untested at the time.** The head-advance branch
moved `cur_seg = next` but did not reload `consuming` from the new segment, so
after exhausting a segment `pop` walked to the end of the chain and returned `None`,
skipping every item in every later segment.

---

## 4. Tests that passed while testing nothing

**The vacuous `assert_ne!`.** `test_auto_create_new_segment` asserted
`assert_ne!(queue.head.as_ptr(), queue.tail.as_ptr())` and passed. `AtomicPtr::as_ptr()`
returns `*mut *mut T` — the address of the *atomic itself*, not the pointer it
holds. Two different fields always differ, so the assertion was a tautology. It
only surfaced when the user changed it to `assert_eq!` for the "one segment"
precondition and saw the failure: `0x1703ae5c8` vs `0x1703ae5d0` — **exactly 8 bytes
apart**, which is also how the `head`/`tail` same-cache-line observation was found.

**The weak concurrent test.** First version: 2 producers spawned *before* the
consumers (so they finished during consumer-spawn overhead), 99 items total, no
no-loss/no-duplicate assertion at all, and an ordering assertion
(`assert!(v[i+1] > v[i])`) that demanded *global* increasing order across both
producers. It passed only because the producers happened to serialise; under real
interleaving it would have false-failed on correct behaviour. Rewritten with a
barrier, 80 000 items, values encoded as `producer * N + seq`, a `vec![false; TOTAL]`
mark-once check, and *per-producer* ordering checked within each consumer's own log.

---

## 5. Miri — two UB reports, and the lesson about hardware tests

### 5a. The headline result

| Tool | Outcome |
|---|---|
| Stress test ×100, **release**, 80 000 items, 8 threads | 🟢 0/100 fail |
| Stress test ×100, **debug** | 🟢 0/100 fail |
| **Miri, first run** | 🔴 **UB: data race** |

**200/200 green on real hardware while the code contained real UB.** The window
never opened: `value` and `state` sit on the same cache line and Apple Silicon is
practically strong for this pattern. This is the empirical case for why loom and
Miri exist, and it is worth stating in the blog exactly this way, with the numbers.

### 5b. UB #1 — the value handoff (all orderings `Relaxed`)

```
error: Undefined Behavior: Data race detected between (1) retag write on thread
`unnamed-9` and (2) retag read of type `std::mem::MaybeUninit<usize>` on thread
`unnamed-3` at alloc58566+0x10
   --> crates/concurrent/src/seg_queue.rs:135
135 |   return Some((*(*cur_seg).slots[idx].value.get()).assume_init_read());
    |   (2) just happened here
help: and (1) occurred earlier here
   --> crates/concurrent/src/seg_queue.rs:66
 66 |   (*(*cur_seg).slots[idx].value.get()).write(value);
```

Miri says *where*, not *what to do*. The user derived the fix himself:
`state.store(WRITTEN, Release)` in push, `state.load(Acquire)` in pop.

### 5c. UB #2 — publishing a pointer before its pointee

Same shape, different signal variable: the value write/read pair was fixed, but
the *segment* was still published with `Relaxed`.

```
error: Undefined Behavior: Data race detected between (1) retag write on thread
`unnamed-8` and (2) atomic read-modify-write on thread `unnamed-7` at alloc82546+0x210
   --> crates/concurrent/src/seg_queue.rs:63
 63 |   let idx = unsafe { (*cur_seg).claimed.fetch_add(1, Ordering::Release) };
    |   (2) just happened here
help: and (1) occurred earlier here
   --> crates/concurrent/src/seg_queue.rs:80
 80 |   let raw = Box::into_raw(Box::new(Segment::new()));
```

(An earlier variant of the same race reported (2) as `consumed.load` in `pop`.)

The non-obvious part, and a genuine user objection — *"`consumed` is an
`AtomicUsize`, how can reading it race?"* — **atomicity belongs to the access, not
to the type.** The segment's fields are *initialised* by plain non-atomic writes
inside `Box::new`, so init-write vs atomic-load is still a data race without a
happens-before edge.

Framing that unlocked it: **every pointer is also a signal.** Publishing a pointer
publishes everything the pointee contains. `state` guards `value`; `next`, `tail`
and `head` each guard an entire `Segment`.

### 5d. The CAS blind spot — the hardest single step

After Release on the `next` CAS *success* ordering, Miri still failed. The hole was
the **failure** ordering:

```rust
next.compare_exchange(null_mut(), raw, Release, Relaxed)
//                                              ^^^^^^^ a LOAD, and its result is used
Err(winner) => { drop(Box::from_raw(raw)); next = winner; }   // then dereferenced
```

A failing CAS performs **only a load**, and its ordering is the failure ordering.
The loser receives the winner's pointer through a `Relaxed` load, so it sees the
address without the contents. Rules that fell out:

- **A floor (Release) exists only where there is a store; a ceiling (Acquire) only
  where there is a load.** CAS success = load + store, so it can need both; CAS
  failure = load only, so `Release`/`AcqRel` as a failure ordering is meaningless.
- My first statement of the rule was too broad — *"if you use the `Err` value you
  need Acquire"*. Corrected: **only if that value is a handle to memory whose
  contents you will read.** `consumed`'s `Err` value is used too, but it is a
  *number*, not a door into a room, so `Relaxed` is right. `next` returns a key to a
  room that must already be furnished.

### 5e. Reasoning beats tools, demonstrated

`tail.load` was weakened from `Acquire` to `Relaxed` and Miri stayed green across
seeds. It was still a latent race: it only bites when a thread's *first* sight of a
segment comes via `tail` rather than via its own `Acquire` on `next`, which is rare.
Lesson: **Miri green after a weakening is not a licence.** Authority order is
(1) happens-before derivation, (2) loom (near-verifier for its bounded model),
(3) Miri (falsifier only), (4) asm/bench (cost only).

### 5f. "Miri reports UB" that wasn't

Later, Miri exited 1 with 20 `error: memory leaked` lines and no UB. Miri prints
both kinds as `error:`, so they are easy to conflate. The leak is R0 working as
designed; **the leak count (20 across 9 tests) is the acceptance test for B1** — drop
`-Zmiri-ignore-leaks` and it must reach 0. I also misread this once myself: an early
run showed `EXIT=1` with `test result: ok` and I reported it as clean.

Also: `#[cfg(miri)]` shrinking of the workload constants took Miri from **379 s to
2.7 s**, which is what made a 20-seed sweep possible at all.

---

## 6. loom

**Setup friction, all real:** missing `AtomicPtr` in the loom import branch;
`SEG_LEN` not shrunk under `cfg(loom)` (state-space explosion); `loom::cell::UnsafeCell`
has no `.get()`, only `with`/`with_mut` closures, which forced the `Slot::write_value`
/ `read_existing_value` helpers; `#[cfg(loom)]` instead of `#[cfg(all(test, loom))]`,
which the compiler itself diagnosed by reporting the test module's imports as unused.
Correct and non-obvious choice made by the user unprompted: map the spin hint to
`loom::thread::yield_now()`, without which loom deadlocks at the wait loop.

Also: `RUSTFLAGS="--cfg loom"` is needed because cargo has no `--cfg` flag (I wrote
the wrong command first), and a separate `CARGO_TARGET_DIR` avoids rebuilding the
workspace on every toggle.

**The vacuous loom test.** First version asserted `assert_eq!(queue.pop(), Some(1))`
— which false-fails, since a consumer may legitimately run first. The "fix" was to
move `join()` *before* `pop()`. That made it pass, and **destroyed the test**:
`join()` is itself a happens-before edge, so the handoff ordering was no longer
exercised at all. The correct shape allows both outcomes and asserts the invariant:
`got = pop()` → `join()` → `rest = pop()` → exactly one of them is `Some(1)`.

Diagnostic value of matching on the pair: `(Some, Some)` = duplicate,
`(None, None)` = lost, `(Some(garbage), None)` = ordering bug.

**Five one-axis tests**, then the negative control that proves they have teeth.
Weakening `state.store` to `Relaxed`:

| test | result |
|---|---|
| L1 `one_producer_one_consumer` | 🔴 FAILED |
| L5 `empty_pop_does_not_burn_the_slot` | 🔴 FAILED |
| L2 `two_producers_claim_distinct_slots` | 🟢 ok |
| L3 `two_consumers_take_one_value_once` | 🟢 ok |
| L4 `two_producers_race_to_install_next_segment` | 🟢 ok |

Exactly the right shape: only the two tests whose axis *is* the `state` handoff
failed. One-axis-per-test means a failure names the broken edge.

L4 carries a comment recording that loom 0.7 does not reliably model handoffs
through `compare_exchange`, so green there is weak evidence.

---

## 7. Benchmarks — where my own hypothesis was wrong

Full numbers live in `seg_queue_bench_results.md`. What belongs here is the arc.

**My hypothesis was aimed at the wrong pair.** I pointed at `head`/`tail` because
they are 8 bytes apart and logically disjoint. They are also written **once per 32
ops**, which makes them cold. The hot pair is `Segment::{claimed, consumed}` — same
adjacency, written on *every* push and pop.

**The number that refuted me was already in the user's own notes.**
`cache_padded_bench_results.md` had measured false sharing at 12.75 ns vs 2.09 ns
uncontended — a ~10.7 ns penalty. `10.7 / 32 = 0.33 ns/op` kills the `head`/`tail`
hypothesis in one division. The whole detour was avoidable by opening a file.

**Padding is not free:** spsc −34%, uncontended +74%. Separating the counters costs
a second cache line per push+pop and a larger `Segment` to initialise.

**The caller dominated everything.** Giving the bench drain loops a backoff took
spsc from 28.3 → 12.5 ns (−56%), reproducibly, while backoff *inside* `pop` was
noise. A consumer hot-spinning on `None` floods the shared lines and starves the
producer it is waiting for.

**Why the mutex had looked so strong:** its consumer must take the lock to discover
emptiness, which self-throttles it and leaves the producer room. **The lock's
serialisation was acting as accidental backpressure.** A lock-free queue has no such
throttle and must re-create it explicitly — the cost side of `pop() -> Option<T>`
refusing to own the wait policy.

**The sealed prediction, and its falsification.** First prediction exercise under
the new discipline. User predicted 70 / 35 / 45 / 45 for
uncontended / spsc / 2p2c / 4p4c after the check-then-claim refactor, reasoning
*"`pop` is deterministic now, so the thread count stops mattering"*.
Measured 17.3 / 13.8 / **44.5** / 71.9 — one hit inside 1%, and the thesis refuted
(2p2c = 44.5 vs 4p4c = 71.9).

Why the reasoning failed, which is the whole point of sealing it: removing the wait
from inside `pop` bounds `pop`'s instruction count but **relocates** the waiting.
A consumer that owned its slot used to spin on *one line it already held*; now it
bails out and re-enters, touching `head` + `consumed` + `claimed` + `state` each
time. Coherence traffic is the dominant term, so a local wait converted into a
global retry **multiplies** it.

**The follow-up hypothesis, also falsified.** User argued the 4p4c deficit was the
caller's fault and a smarter consumer (or a notified future) would fix it. Making
the drain policy an explicit swept dimension answered it in one run: SegQueue loses
to the mutex under **all three** policies (75–85 vs ~54), while the mutex is flat at
54.1–54.6. At 8 contending threads serialisation is simply the better strategy —
the lock converts contention into queueing, the lock-free structure converts it
into coherence traffic. Policy swings SegQueue 1.9× and the mutex not at all, which
is the cleanest confirmation of the accidental-backpressure finding.

Counter-intuitive sub-result: **hot spinning is the worst spsc policy** (23.4 vs
12.5 for yield) with only 2 threads and no CPU oversubscription — the spinner
starves the producer through *coherence traffic*, not CPU.

---

## 8. A side finding: `Backoff` is tuned for x86

`SPIN_LIMIT = 6` is inherited from `crossbeam-utils`, tuned against x86 `pause`
(~1 ns). `core::hint::spin_loop()` lowers to `isb SY` on aarch64, measured at
**12.2 ns**. So the same constant buys a 12× larger budget: 784 ns at the capped
burst, ~1.55 µs for the full ladder before the first yield (vs ~64 ns / ~127 ns on
x86). The module doc claimed "~64 ns on modern x86/aarch64" and was wrong here by
12×.

Wasted CPU is defensible (`yield_now` measures 4.6 µs, so ~1.55 µs of spinning
first is a sane ~1:3 ratio). The non-obvious cost is **check granularity**: the
caller re-tests its condition every 784 ns instead of every ~64 ns. Harmless for
waiters resolving in tens of ns; not harmless in the 100 ns–1 µs band.

The user found this by reading his own measurement notes back against a doc claim —
the "notes directory answers instead of memory" mechanism working as intended.

---

## 9. Meta — process observations worth a paragraph in the blog

- **Where the work divided well:** the user wrote every line touching atomics,
  `unsafe`, ordering and lock-free protocol; I wrote scaffolding and bench harness.
  Where it divided *badly*: I also formed the benchmark hypothesis and interpreted
  the numbers, which is where the learning was. I got the false-sharing hypothesis
  wrong and the measurement corrected *me* — that correction should have been his.
  Fix adopted mid-build: the user seals a numeric prediction in `notes/` before each
  measurement; I keep the harness but not the hypothesis.
- **Three buckets are the whole cost model**, not a memorised table: same-core
  L1 ~1–2 ns, cross-core line transfer ~10 ns, scheduler involvement ~5 µs. They are
  5× and 500× apart, so only the bucket matters, never the digit. Everything else is
  counting frequency — which is exactly the step that would have killed the
  `head`/`tail` hypothesis.
- **The queue's contract pushes a performance-critical decision onto the caller**,
  and the caller will get it wrong. That is the honest cost of `pop() -> Option<T>`,
  and it is the thread that ties the brainstorm's single-responsibility argument to
  the final benchmark table.

---

## 10. R1 — naive refcount. Where it broke, in order

### 10a. Manual acquire/release does not survive six exit paths

First attempt put bare `fetch_add`/`fetch_sub` at the call sites. **8 sites, 5 wrong:**

- **The lost `return`.** Rewriting `return Some(...)` into `let res = ...; fetch_sub(1); res`
  left the match as a *statement*, so the trailing `;` discarded the value and the
  loop went round again. Effect, traced on `test_push_pop`: CAS(0→1) succeeds, the
  value is **read out of the slot** (moved!), **discarded**, loop continues;
  `consuming` is stale so the next CAS fails, `consuming` becomes 1, then
  `claimed(1) <= consuming(1)` → `return None`. **The item is destroyed and `pop`
  reports empty.** With `T = usize` it evaporates silently; with `String`/`Box` the
  payload would be dropped while the queue claims to be empty.
- **Why that showed up as a hang, not a failure, in the MPMC test.** Its termination
  is `popped >= TOTAL`, and `popped` only advances when `pop` returns `Some` — which
  now never happened. Consumers spun forever. One bug, two completely different
  symptoms depending on the test.
- **The wrong-segment release.** In `pop`'s head-advance branch:
  `cur_seg = next; ...; (*cur_seg).release_ref();` — the acquire at the loop top was
  on the **old** segment, so the release landed on the **new** one, which this thread
  never acquired. Old segment leaks +1; new segment does `fetch_sub` on `0` and
  **wraps to `usize::MAX`**, after which `previous == 1` is unreachable forever.
- **Four leaking exit paths in `pop`** (`next.is_null()`, `claimed <= consuming`,
  `state != WRITTEN`, CAS `Err` → `continue`) plus a **double bump** on `push`'s
  overflow path.

Crucially: **all nine tests passed the whole time.** Nothing reads the refcount, so
none of these bugs were observable. The mechanism was being built with no feedback
loop at all — which is its own lesson about what "the tests are green" buys you.

### 10b. The guard fixes the bookkeeping and nothing else

RAII (`acquire_ref` in the constructor, `release_ref` in `Drop`) reduces 8 sites to
one acquisition and zero manual releases; `return`, `continue` and unwinding are all
handled by the compiler; reassigning the guard gives **acquire-new-then-release-old**
for free, because Rust evaluates the RHS before dropping the old value. The
wrong-segment bug becomes unrepresentable rather than merely fixed. Two further
gifts: passing `&SegGuard` to `advance_tail` *proves* the caller pins that segment, so
the callee's own acquire/release pair disappears entirely; and `fn get(&self) -> &Segment<T>`
makes `let p = guard.get(); drop(guard); use(p);` stop compiling.

**But it cannot fix the race**, because constructing the guard *is* a dereference of
the pointer whose validity is in question.

Why `Arc` is not a counter-example — the distinction worth a paragraph in the post:

| | `Arc::clone` | this guard |
|---|---|---|
| pointer comes from | an `Arc` you **already own** | a shared slot (`head`/`tail`/`next`) you just **loaded** |
| proof the count is ≥ 1 | the `&self` you cloned from | **none** |

`Arc` never builds itself from a raw pointer found in shared memory. That is precisely
what this does, and no amount of `Drop` makes it sound.

### 10c. `&&` short-circuits — the bug that hides the bug

```rust
if self.get().consumed.load(Relaxed) == SEG_LEN && self.get().release_ref() == 1 {
```

Rust's `&&` is lazy, so when the segment is not fully consumed `release_ref()` **never
runs**. An actively-used segment is by definition not fully consumed, so this skips the
release for most guards and the count on a live segment only ever rises — it can never
reach 1, nothing is ever freed, and the leak count never moves. A bug whose symptom is
*the absence of the symptom you were looking for*.

Correct shape: read `consumed` **first** (after releasing, if you were not the last
holder another thread may have freed it and the read is a UAF), release
**unconditionally**, decide last.

### 10d. ⭐ The stale decision — the trace to show deep in the blog

The refcount answers *"how many threads hold S right now?"*. Freeing needs
*"will anyone ever hold S again?"*. Those are different questions, and
`release_ref() == 1` answers only the first while the code acts as though it answered
the second.

S is fully consumed, `head` still points at it. Two threads call `pop`:

| time | thread A | thread B | S's count |
|---|---|---|---|
| t1 | loads `head` → S | | 0 |
| t2 | `acquire` S | | **1** |
| t3 | `consuming >= SEG_LEN`, `next` null → `return None` | | 1 |
| t4 | guard drops: reads `consumed == SEG_LEN` ✓ | | 1 |
| t5 | `release_ref()` → 1 ✓ **decides to free** | | **0** |
| t6 | | loads `head` → S, `acquire` S | **1** |
| t7 | `fence`, `Box::from_raw` → **frees S** | | — |
| t8 | | reads `S.consumed` | **use-after-free** |

A's decision was *correct at t5*. By t7, when it acts, the world has changed.
**The decision and the action are not atomic, and the fact being decided on can change
in between.** That sentence is the heart of the post.

### 10e. The double free — where "0 → 1 → 0" comes from

Same setup, B gets further before A's free lands:

| time | thread | action | count |
|---|---|---|---|
| t1 | A | acquire S | 0 → **1** |
| t2 | A | drop: `release_ref()` → 1 → **decides to free** | 1 → **0** |
| t3 | B | acquire S | 0 → **1** |
| t4 | B | `return None`, drop, `consumed == SEG_LEN` ✓ | 1 |
| t5 | B | `release_ref()` → 1 → **also decides to free** | 1 → **0** |
| t6 | A | `Box::from_raw(S)` — first free | — |
| t7 | B | `Box::from_raw(S)` — **second free** | — |

Both observed `previous == 1`. **Neither was wrong about the present; both were wrong
about the future.**

### 10f. What would fix it, and why it is unavailable here

Zero is only trustworthy if the count is **monotonically non-increasing** from the
moment you start trusting it: then it can only fall, and once at zero it stays there.
What lets it *rise* is **reachability** — a thread obtaining a fresh pointer. So the
condition is "make it unreachable **first**, then trust zero", not an extra check
bolted onto the free.

Three conditions, each necessary, and the pair of orthogonal ones is worth stating
because they look redundant and are not:

| condition | what it establishes | what breaks without it |
|---|---|---|
| `ref_count == 0` | nobody is **mid-operation** | use-after-free |
| `consumed == SEG_LEN` | every item has been **taken out** | silent data loss — and the `T`s leak, since `MaybeUninit` never drops |
| **unreachable, one-shot** | nobody can **obtain a new pointer** | the two traces above |

A nice illustration that the first two complement rather than duplicate each other:
`consumed` counts *tickets issued*, not *reads completed* — a consumer that CAS'd
`consumed` to `SEG_LEN` may still be inside `read_existing_value`. That gap is covered
by the *other* condition, because that consumer still holds a guard.

The one-shot already exists in the structure: exactly one thread wins
`head.compare_exchange(S, S.next)`, and only that winner may release the queue's own
reference (so a segment is born with count `1`, not `0`, which also removes the
"a fresh segment is at zero" ambiguity).

**And then the wall.** The `head` CAS stops *new arrivals* from reaching S — they load
`head` and get `S.next` or later. It does not stop this:

```
Thread T calls pop(), loads head → P, acquires a guard on P.
T is descheduled for a long time.
Meanwhile: others drain P, then drain S, head moves past both, S is FREED.
T wakes. Its guard on P is still perfectly valid; P is alive.
T reads P.next  →  S                  ← a pointer to freed memory
T calls acquire(S)                     ← dereferences it to bump a counter
```

T did nothing wrong. The principle, which is the line the whole step exists to earn:

> **A guard on `P` protects `P`. It says nothing about what `P` points to.
> Reachability is transitive; a per-node refcount is not.**

Which is why no additional condition can rescue this design, and why
"make it unreachable first" only works where unlinking is atomic — a Treiber stack pop
removes the node with the same CAS that reads it. A segmented queue has no such moment.
Hazard pointers and epoch escape by making the *acquire itself* safe: the thread
announces "I am traversing" in per-thread state **before** touching anything, so the
freer can hold off without the thread having to dereference first.

### 10g. ⭐ Miri names it — the payoff, verbatim

Built the strongest honest version of R1 before measuring: a RAII guard, plus a
third cursor `reclaim` trailing `head`, freeing strictly oldest-first under a
one-holder flag, with all three conditions checked. Reclamation **works**:

| signal | result |
|---|---|
| `memory leaked` without `-Zmiri-ignore-leaks` | **20 → 0** |
| Undefined Behavior | **1 × data race / use-after-free** |

The report:

```
error: Undefined Behavior: Data race detected between
  (1) atomic load                   on thread `unnamed-5`
  (2) retag write of Segment<usize> on thread `unnamed-2`
  at alloc63178+0x200

(1) crates/concurrent/src/seg_queue.rs:296
      let next = guard.get().next.load(Ordering::Acquire);     [pop]
(2) Box::from_raw
      → seg_queue.rs:285   try_reclaim
      → seg_queue.rs:311   pop
```

The interleaving:

```
T5 (pop):  loads self.head → S          head == S here; S.ref_count == 0
T2 (pop):  advances head  S → S.next
T2:        drops its guard on S         S.ref_count still 0
T2:        try_reclaim():  reclaim == S
                          S != head          ✓  (head just moved)
                          S drained          ✓
                          S.ref_count == 0   ✓  ← all three conditions hold
T5:        SegGuard::acquire(S)         0 → 1   ← too late; T2 has already decided
T5:        line 296: reads S.next
T2:        Box::from_raw(S)             frees S
                                        ⇒ data race / use-after-free
```

**Every one of the three conditions was true at the instant T2 checked it.** The
defect is not a missing fourth condition. T5 held a pointer to S but had not yet
registered interest, and the gap between *obtaining a pointer* and *registering*
cannot be covered by anything stored inside S — because registering means touching
S.

This is §10d's stale-decision trace confirmed by a tool on real code rather than by
argument, which is the single most useful thing to show in the post: the reasoning
predicted the failure, the strongest implementation of the design still exhibited it,
and Miri named it on the first run while all nine unit tests stayed green.

### 10h. Things built along the way that survive into R2/R3

- **The guard interface.** `push`/`pop`/`advance_tail` no longer care what mechanism
  is underneath, which is also the condition for the benchmarks to be comparable
  across steps.
- **`advance_tail(&SegGuard)`** — taking a guard by reference *proves* the caller
  pins that segment, so the callee needs no acquire of its own. "Who holds what"
  became a type-level fact.
- **`fn get(&self) -> &Segment<T>`** — the returned reference borrows the guard, so
  `let p = guard.get(); drop(guard); use(p);` does not compile.
- **The ordered trailing-reclaim cursor** is a genuinely sound answer to a different
  question (making a count of zero *final*), and the reason it does not rescue R1 is
  worth stating precisely: monotonicity was never the problem.
- **Cost noted, not measured yet:** the `reclaiming` flag reintroduces a
  serialisation point — the same trade the `Mutex<VecDeque>` baseline makes — and
  one thread sitting on an old segment blocks reclamation of everything after it,
  which is the same pathology epoch has with a thread parked inside a pin.

### 10i. Design axis the user spotted: inline vs deferred reclamation

Observation, unprompted: *"pop will take more time because it does two things at
once — not single responsibility. It's the same as GC: either you collect during
process, or you mark it collectable and do it as a side job."*

Correct on both counts, and the taxonomy is the standard one:

| approach | who pays | cost shape |
|---|---|---|
| **inline / synchronous** — collect during the operation *(R1 as built)* | the mutator, at an unpredictable moment | **latency spikes** |
| **deferred** — mark retired, drain later | amortised, or a separate thread | steady, at the cost of delayed frees |

And the punchline: **epoch-based reclamation *is* the deferred variant.**
`defer_destroy` pushes onto a per-thread garbage bag which drains in batches once the
epoch advances. "Mark it collectable then do it as a side job" is not an alternative
to R3 — it is R3, arrived at from single-responsibility reasoning rather than from
reading the paper.

The irony worth putting in the post: check-then-claim removed the unbounded spin from
`pop` *specifically* to bound its latency — and then R1 put a reclamation walk in the
same function, reintroducing a spike. Bounded now, but variable: one pop in every
`SEG_LEN` pays a contended flag `swap` plus however many segments happen to be
freeable.

Four options for this structure, with the one that matches the design philosophy
already committed to:

1. bounded work per call (free at most 1–2) — caps the spike, still the mutator's problem
2. a dedicated reclaimer thread — clean, but a low-level primitive that spawns threads
   is a serious API imposition (ownership, 10 000 queues, shutdown)
3. **caller-driven**: make `try_reclaim()` public and let the caller choose when —
   the same move as `pop() -> Option` leaving the wait policy outside
4. per-thread retire lists drained at safe points — i.e. epoch

Two costs recorded rather than measured:

- the `reclaiming` flag is a **serialisation point**, and the measured finding at
  4P/4C was that *serialisation is why `Mutex<VecDeque>` wins there*. R1 therefore
  pushes the lock-free queue toward the mutex's cost profile.
- one thread sitting on an old segment blocks reclamation of **everything after it**.
  Same pathology epoch has with a thread parked inside a pin.

**And a measurement-design lesson:** the spike is *invisible to the current bench*.
Criterion reports a median per-item cost, and a spike on 1 pop in 32 barely moves a
median — it lands almost entirely in p99. Seeing it needs percentile instrumentation
(HdrHistogram / `latency-lab`, a later node). Choosing the wrong statistic hides the
very effect you set out to find.

---

## 11. How hard is this, honestly — context for the post's framing

The user asked, at the point of maximum frustration: *who invented this, did they
struggle like this, how long did it take them?* The answer is load-bearing for the
blog, because the honest version is encouraging in a way reassurance is not.

| technique | who | when | context |
|---|---|---|---|
| Michael–Scott queue (the baseline segments are compared against) | Maged Michael & Michael Scott | 1996, PODC | — |
| **Hazard pointers** (R2) | Maged Michael, IBM Research | 2002–2004 | **the same Michael, 6–8 years after the queue** |
| **Epoch-based reclamation** (R3) | Keir Fraser | 2004 | **a Cambridge PhD thesis**, *Practical Lock-Freedom* |
| the memory model all of this is reasoned in | Boehm, Adve et al. | PLDI 2008 → C++11 | Adve's memory-model work starts with her **1993 PhD** |
| crossbeam / `crossbeam-epoch` | Aaron Turon (~2015); largely rewritten by Stjepan Glavina (~2017–18) | | multiple FTE-years |

Three facts that reframe the difficulty:

1. **Hazard pointers and EBR exist *because* refcounting was tried first and failed.**
   The whole "safe memory reclamation" literature is the field working around exactly
   the wall hit here. Re-deriving that negative result took two days; the field took
   years to converge on it.
2. **Published lock-free algorithms have repeatedly been proven incorrect after
   publication**, by experts, in peer-reviewed venues.
3. **loom exists because experts could not get this right by reading their own code.**
   So do Relacy and CDSChecker. The tools are the admission.

And the formal vocabulary — happens-before, release/acquire as a *specification*
rather than per-architecture folklore — was not usable until **2011**. Everyone before
that wrote this code against informal per-CPU rules and got it wrong routinely.

**The "chaotic memory ordering" feeling, audited.** At the moment the user reported
the orderings felt chaotic, all 24 sites in the file were checked: **23 followed the
rule, 1 was wrong — and that one was *over-strict*, i.e. harmless** (`acquire_ref` as
`fetch_add(Release)` where a bump publishes nothing). The subtle ones were right:
`release_ref(Release)` paired with `ref_count.load(Acquire)`, the CAS failure ordering
on `next`, `Relaxed` on both ticket counters, a correct `swap(Acquire)` /
`store(Release)` lock.

So the chaos was not in the artifact — it was eight orderings across three functions
held in working memory at once. That is a capacity limit, not a comprehension failure,
and "understand harder" is the wrong remedy. The remedy that worked was writing the
**handoff table** down: one row per signal variable, who publishes, who observes, what
it guards. It collapsed seven open questions into two categories in about a minute, and
it now lives permanently in the module doc rather than in a conversation.

That is probably the most transferable thing in the whole series: **the difficulty is
real and historically validated, and the fix is an artifact, not more effort.**

## 12. ⭐ The benchmark was measuring the wrong thing — twice, and the user caught both

Worth a chapter of its own, because it is the most transferable failure in the build and
neither catch was mine.

**Catch 1 — the wrong shape.** Rounds 1–5 all measured NPNC: 1P1C, 2P2C, 4P4C. But both
real consumers of this queue are **N producers → exactly one consumer** (WAL
group-commit's fan-in to the sole `fsync` caller; the runtime's cross-shard inbox). With
one consumer, `consumed`, the `head` CAS and half the refcount traffic are *uncontended*.
Five rounds of careful measurement had been quantifying pressure that neither use case
produces.

**Catch 2 — the wrong opponents.** The question the user actually asked was *"why
implement this hard thing instead of using a mutex?"*, and I had been answering it against
`Mutex<VecDeque>` alone. But for unbounded MPSC the real alternatives are
`std::sync::mpsc` (the standard library's purpose-built answer to exactly that shape) and
`crossbeam_queue::SegQueue` (the reference implementation of the algorithm being
reimplemented). His framing was sharper than mine: *"why do you bench it with the ones it
will never be applied to, and against opponents not optimised for that use case?"*

Fixing both reversed the conclusion in **both** directions at once:

- **the structure is vindicated** — crossbeam is flat from 1 to 8 producers (13.8 → 16.7 ns)
  and beats a mutex 2.3× and `std::sync::mpsc` 10× at 8 producers. The answer to "why not
  a mutex" is a measured 2.3×.
- **this implementation is indicted** — 7.7 → 77.8 → 105.9 → 104.9, a **10× cliff from one
  producer to two**, and slower than a plain mutex at 4 and 8 producers.

And one consolation that localises the fault precisely: **at 1P1C `seg_queue` beats crossbeam**
(7.66 vs 13.78). The push/pop/boundary machinery is fine; the reclamation scheme bolted
on top is what costs.

**The transferable lesson:** a benchmark encodes a belief about how the thing will be
used, and about what it is competing with. Both beliefs are assumptions, both were wrong
here, and neither was visible from inside the numbers — five rounds of increasingly
careful measurement of the wrong configuration produced increasingly confident wrong
conclusions. The fix was not better statistics.

**An attribution caveat to carry into B3.** Crossbeam differs from `seg_queue` in *two*
ways at once: layout A (global index) versus layout B (per-segment counters), **and**
DESTROY-bit reclamation versus a refcount. So the 6× cannot be attributed to the refcount
from this data alone. That makes B3's bench the decisive measurement:

- if **B3 (layout B + epoch) lands near crossbeam** → the refcount was the problem and
  layout B is fine
- if **B3 is still ~6× off** → layout B is the problem, and Track A stops being "understand
  everything" and becomes necessary

## Running TODO for the blog

- [x] B1 (naive refcount) — reasoning §10, Miri UAF verbatim §10g, bench §12 + bench-notes rounds 5-6
- [ ] B2 (hazard pointers) — capture the per-read fence cost
- [ ] B3 (epoch) — capture the quiescence design and the loom adversarial cases
- [ ] A3 / CB — capture the layout comparison and the diff against crossbeam source
- [ ] **Delete this file once the blog is finalised**
