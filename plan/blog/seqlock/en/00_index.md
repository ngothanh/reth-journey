# Designing a SeqLock

Somewhere inside a system that reads one shared value far more often than it writes
it — a blockchain node asking "what's the chain head?" tens of thousands of times a
second between blocks, an exchange reading the mark price on every single order —
there's a primitive that lets all those readers proceed without ever blocking and
without ever writing a byte to shared memory. It's called a SeqLock, and if you've
called `clock_gettime` on Linux without it hitting the kernel, you've used one.
This series designs one from scratch.

It's a design investigation, not a coding tutorial. We start from the problem a
SeqLock exists to solve, watch each lock you'd normally reach for fail exactly one
of the constraints, and then make the bet that defines the whole primitive: instead
of preventing the reader from observing a half-written value, we let it happen and
make the reader detect it. Every decision after that is forced — by a use case, or
by the failure of the simpler alternative, or, in one memorable case, by an ARM
processor reordering your instructions and corrupting a value your test suite swears
is fine.

No lock-free background is assumed. The machinery — `Relaxed`/`Acquire`/`Release`,
fences, `Pod`, Miri, loom — is introduced at the moment the design collides with it.

## The parts

**[Part 1 — The problem, and why the obvious locks don't fit.](01_the_problem.md)**
One writer, many readers, and a value of several fields that doesn't fit in a single
machine word — so there's always an instant where memory is half-old, half-new, and a
reader landing there gets a value that never existed. The three constraints that make
this hard, and then the tour of failures: a `RwLock` lets readers in together but makes
every one of them *write* a shared counter, serialising cores that don't conflict;
`ArcSwap` and RCU dodge the tearing but drag the reader back into announcing itself for
reclamation. Every correct option breaks the same rule, and it points at the only way
out — the reader must be invisible.

**[Part 2 — The bet: let it tear, and catch it.](02_the_bet.md)**
If the writer can't be stopped and the reader can't announce itself, one move is left:
let the read tear, and give the reader a way to notice afterward and retry. That reduces
everything to a single question — how does a reader know it read during a write? — and
we derive the answer the hard way, watching a boolean flag fail because it carries no
history, until the only thing that works falls out: a counter that's even when the value
is stable and odd while it's being written, sampled before and after.

**[Part 3 — Getting the memory ordering right.](03_memory_ordering.md)**
The protocol is correct on paper and still tears on a real Apple M2, four runs out of
five green — the fingerprint of a memory-ordering bug. The counter builds a window;
nothing yet forces the payload to stay inside it. We fix it with fences, and to place
them we need the idea people always get backwards: `Release` and `Acquire` are one-way
gates, each guarding only one side of the operation it's attached to. Two of the four
window edges can use an ordering on the atomic itself; the other two need a standalone
fence — and the fences, it turns out, are what let two threads shake hands into a
happens-before relationship. This is the heart of the series.

**[Part 4 — Reading without UB, and trusting it.](04_trusting_it.md)**
We deliberately let the reader read bytes the writer is changing. In C that's a folk
practice with `volatile`; in Rust's memory model it's a data race — undefined behaviour
— and Miri says so out loud. The fix makes every payload access atomic, word by word,
turning "reading garbage" from UB into a legal read the counter throws away — which
forces the payload to be `Pod`, a bound that turns out to be a license the implementer
*signs* rather than one the compiler checks, and which doesn't even cover alignment.
Then the sequence counter earns a second job as the writers' lock, and the trust
question: the test that must *fail*, Miri for the race, loom for the interleavings, and
a benchmark that shows — in nanoseconds — a read path that stays flat while a `RwLock`
degrades to 450× slower.

## How to read it

In order — each part opens where the previous one stopped and closes on the question
the next answers. Ten to fifteen minutes each. Part 1 sets the problem and rules out the
alternatives; Part 2 makes the core bet; Part 3 is the memory-ordering heart; Part 4 is
the language, the multi-writer case, and the proof. Stopping after Part 2 already gives
you the whole idea; Parts 3–4 are where it meets the hardware and the language, and
where most real SeqLock bugs live.

## Scope

This series designs a generic, reusable `SeqLock<T>` — the kind you'd put in a
concurrency crate, not a one-off hardcoded to a single struct. Everything is built and
measured on `aarch64` (Apple M2), because the weak memory model is where the interesting
failures show; an x86 run would hide half of Part 3. The numbers in Part 4 are real,
from the crate's `criterion` benches.

## Glossary

- **SeqLock** — a lock where readers never block and never write shared memory; they
  read optimistically and retry if a write overlapped. Single value, many readers,
  rare writer.
- **torn read** — observing a value that's part-old, part-new because a write was in
  progress; a value that never actually existed as a whole.
- **payload** — the protected value itself (as opposed to the sequence counter that
  guards it).
- **sequence counter / seq** — the integer the writer bumps around each write; even =
  stable, odd = a write is in progress. The reader reads it before and after.
- **`Relaxed` / `Acquire` / `Release`** — memory orderings on an atomic operation.
  `Relaxed` = atomic but no ordering guarantees; `Acquire`/`Release` add one-directional
  ordering and pair across threads to establish happens-before.
- **fence** — a standalone ordering barrier (`atomic::fence`), not attached to any one
  atomic; two-sided for the operations it governs, where an ordering-on-an-op is one-sided.
- **`Pod`** ("plain old data") — a marker trait promising a type is just bytes: no
  padding, every bit pattern valid, defined layout. Lets you reinterpret it as raw
  words safely.
- **MESI / cache coherence** — the protocol that keeps per-core caches consistent; a
  cache line written by one core must be invalidated in others, which is why a shared
  written counter serialises cores that don't logically conflict.
- **Miri** — an interpreter that runs Rust against the memory model and catches
  undefined behaviour (data races, invalid pointers) that a normal test executes but
  cannot detect.
- **loom** — a model checker that re-runs a small concurrent test under every possible
  thread interleaving; the verifier for lock-free code.

*Deutsch: [`../de/00_index.md`](../de/00_index.md)*
