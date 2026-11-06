# Building `reclaim`: safe memory reclamation in Rust

A lock-free structure removes a node with one compare-exchange. Freeing it is one call
to `free`. Between the two, on another core, a thread may still hold the node's address.
Languages with a garbage collector don't have this problem. Rust has no collector. So
something has to decide *when* removed memory may be freed, without making any reader
take a lock.

This job is called *safe memory reclamation*. This series builds a crate for it, called
`reclaim`, starting from an empty `lib.rs`. It's a design investigation, not a tutorial.
Every decision is forced by a crash, a verification tool or a measurement. The wrong
first attempts are kept, because they explain the design better than the final code
does. More parts will follow as the crate grows: a small record per thread that the
scheme can find, the rule that decides when freeing starts, hazard pointers, and epoch.

## The parts

The glossary at the end of this page explains the main terms.

**[Part 1 — The problem: freeing memory another thread may be
reading.](01_the_problem.md)** A six-line use-after-free in a Treiber stack, and why a
counter inside the node can never fix it: announcing "I'm reading it" already touches
the node. The two rules that must hold before an object can be freed safely, who is
responsible for each one, and the three roles that follow: object, structure, scheme.
What the crate offers that existing ones don't.

**[Part 2 — Roots: where every reader starts.](02_roots.md)** A reader can't invent an
address. Reading an address from a place is not the same as opening the object behind
it. A queue that treats every node's `next` as a root can never free anything. Why roots
are declared in the type system and not only documented: the bug *is* the gap between
the roots you think your code uses and the roots it really uses.

**[Part 3 — Two ways to say "I'm reading".](03_guards.md)** Hazard pointers publish an
address. Epoch publishes a time. So the same word, "guard", means two different things.
The race where a reader announces itself too late, and the re-read that prevents it.
Four decisions inside one trait method, and why the obvious answer was wrong each time.

**[Part 4 — What an unsafe trait promises.](04_the_contract.md)** What an object must
provide so the scheme can keep it on one pile with retired objects of other types. Why
the `reclaim` method has no default body. A test for when a trait must be `unsafe`. The
four promises inside one getter, `retire_link`. `Domain`, the `Leak` scheme (it never
frees anything), and a Treiber stack as the first structure built on the crate.

**[Part 5 — Where a root lives, and what only Miri saw.](05_where_a_root_lives.md)** A
root's place stored as a plain field, then in a `Box`, then behind a raw pointer. Both
early versions passed every test and every loom model. Miri rejected both. It rejected
the second under only one of Rust's two candidate aliasing models.

**[Part 6 — The pile of retired objects.](06_the_pile.md)** A pile that holds objects of
any type and never allocates. Each object carries a link of three machine words. One of
them is a function pointer that still knows the object's real type. Push works like a
stack push. Take empties the pile with one `swap`, so the taker owns every link before
it reads one. Free reads `next` before it frees the object. A check for retiring the
same object twice costs one load.

**[Part 7 — One `Release`, one `Acquire`, everything else `Relaxed`.](07_orderings.md)**
Data passes from one thread to another at only one point: a link pushed by one thread
and taken by another. That hand-off gets one `Release` and one `Acquire`. Why every
other access can be `Relaxed`. Release sequences, a small wrapper that lets loom see
every `UnsafeCell` access, and negative controls that prove the loom test can fail.

**[Part 8 — Measuring the pile: one head, eight heads, and chains.](08_measuring.md)**
Predictions written before running, compared with measurements on an M2. The part
measures one shared head (≈ 125 ns per push at 8 threads), eight shards (eight separate
heads), false sharing between them, a control where no thread shares anything, and
chains that push 20 objects with one exchange. Sharding plus chains matches the control.
A one-line formula explains why chains on one shared head missed the prediction.

## How to read it

In order. Each part starts where the previous one stopped and ends with the question the
next one answers. Parts 1–3 explain the model, with little code. Parts 4–5 build the
contract between object, structure and scheme, and the first structure that uses it.
Parts 6–8 build the crate's first concurrent component, the pile: first for one thread,
then for many, then measured with eight.

## Scope

This series covers the `reclaim` crate as built so far: the interface (`Retirable`,
`Reclaimer`, `Guard`, `Domain`, `Root`), the `Leak` scheme, and the pile of retired
objects, with sharding and chains. Schemes that really free memory (hazard pointers and
epoch) come in later parts. Built and measured on `aarch64` (Apple M2, 4 performance + 4
efficiency cores, 128-byte cache lines). Every correctness claim is one that Miri or
loom checked, with a negative control showing the check can fail. Every number comes
from the crate's `criterion` benches and is shown next to the prediction written before
the run.

## Glossary

- **safe memory reclamation** — deciding when memory removed from a concurrent structure
  may be freed, given that other threads may still hold its address.
- **retire** — the structure says "I'm done with this object". Nothing is freed.
- **reclaim** — the scheme frees the object, later, when no reader can still hold it.
- **grace period** — the wait between retire and reclaim.
- **Rule 1** — no *new* reader can get the object's address. The structure's job.
- **Rule 2** — every reader that already has the address has finished. The scheme's job.
- **object** — the thing that gets freed.
- **structure** — the thing that contains objects.
- **scheme** — the third party that decides when an object may be freed.
- **root** — a place a reader loads an address from and then opens (dereferences) the
  object at that address.
- **guard** — what a reader holds while its "I'm reading" announcement is published in a
  shared slot. Dropping the guard clears the slot.
- **hazard pointers** — a scheme where readers publish the *address* they're reading.
- **epoch** (epoch-based reclamation) — a scheme where readers publish *when* they
  started reading.
- **domain** — one instance of a scheme, plus its declared roots.
- **pile** — the scheme's list of retired objects, chained through links stored inside
  the objects.
- **Treiber stack** — the simplest lock-free stack: one atomic `head`, with push and pop
  each a compare-exchange on it.
- **`Release` / `Acquire`** — memory orderings. When an `Acquire` read sees a value
  written by a `Release` write, the reading thread also sees everything the writing
  thread did before that write. This link between the two threads is called a
  happens-before edge.
- **release sequence** — a chain of read-modify-write operations (such as `swap` or a
  compare-exchange) on one atomic. An `Acquire` read at the end of the chain
  synchronizes with every `Release` in it: it sees what each of them published.
- **false sharing** — independent data that sits on one cache line. A write to any part
  of it invalidates the other cores' copies of the whole line.
- **negative control** — breaking the code on purpose to check that a test or tool
  really fails (goes red).
- **loom** — a model checker. It runs a small concurrent test under every interleaving
  (every order of the threads' steps) that the memory model allows.
- **Miri** — an interpreter that runs Rust code, checks it against Rust's memory rules,
  and reports undefined behavior (such as aliasing violations) and memory leaks.
