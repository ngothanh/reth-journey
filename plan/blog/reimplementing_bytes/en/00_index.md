# Reimplementing `Bytes`: one type, three ways of owning memory

This is a series about a small but famously tricky piece of code: a *zero-copy byte
handle*. If you've ever used Rust's `bytes` crate, or Facebook's `IOBuf`, or Netty's
`ByteBuf`, this is the thing living inside them — except we're going to rebuild it
from scratch to understand why it's designed the way it is.

The series isn't a type-along coding tutorial. It's an *investigation*: we start
from an everyday need (a network program reads data in and passes it around), hit a
performance problem, try a few obvious fixes, watch them fail, and each time we hit
a wall a piece of the real design reveals itself. No piece falls from the sky; every
decision is *forced* by the one before it.

You don't need to know `Bytes`, `BytesMut`, or `freeze` beforehand — Part 1 builds
everything from zero.

The series comes in two tracks: **Parts 1–5 are *design*** (why `Bytes` has the shape
it does), **Parts 6–8 are *implementation*** (sitting down to write each vtable
function, `from_vec`, and `slice` correctly, along with the code details the design
track deliberately parked: the memory-ordering discipline of the refcount, the
bit-packing trick, and the promotion race).

## The design track — five parts

**[Part 1 — A byte's journey from the wire into your program.](01_the_problem.md)**
We set the scene: a network program takes in data, needs a writable buffer
(`BytesMut`), then needs to turn it into a shareable read-only handle (`Bytes`) via
an operation called `freeze`. We discover `freeze` can be very slow if it copies,
and impose a requirement: `freeze` must not copy. Then we try two obvious designs
(`Vec<u8>` and `Arc<[u8]>`) and watch where they break — surfacing the central
tension: *one type, three ways of cleaning up memory.*

**[Part 2 — One type, many behaviours.](02_vtable.md)**
In Rust, "how to clean up memory" normally lives in the *type*, and the compiler
handles it all. But we have only one type and need three behaviours. This part shows
how to lower the cleanup decision from the compiler down into *data inside the
struct* — a hand-written dispatch table (a vtable). Plus a question anyone designing
such a type must be able to answer: why does that table have exactly *two* slots.

**[Part 3 — Separating "which bytes" from "who owns them".](03_split_and_counting.md)**
The trick that makes this design both flexible and *fast*: lay out the struct's
fields so that reading bytes never has to look at the ownership information. This
part also introduces a simple way of thinking that backbones the hard parts to come —
every way of owning memory, reduced, is one question: *how many times does this
block get freed?*

**[Part 4 — The wall: when clone breaks everything.](04_promotion.md)**
This is the hardest part. Three of the four behaviours are easy, but cloning a
sole-owner handle causes a double-free. The only escape — called *promotion* —
forces something very unusual in Rust: a value has to *mutate back* into another,
already-existing value, mid-way through its life.

**[Part 5 — `AtomicPtr`: mutating back, safely.](05_atomics.md)**
The back-mutation from Part 4 poses three independent requirements, and all three
happen to be solved by a single choice of field type. This part walks through the
three concurrency concepts most people find most abstract — interior mutability,
CAS, and memory ordering — but this time each one attaches to a concrete problem
we're actually forced to solve, not abstract theory. It closes with five questions
you can carry into any systems problem later.

## The implementation track — three parts

**[Part 6 — From the model down to code: `static` and `shared`.](06_static_and_shared.md)**
We write the first four vtable functions. `static` is the warm-up (an empty `drop`
function is "free 0 times" written out). `shared` is hard in exactly one place, but
that place is an important ordering lesson the design track didn't touch: `share_drop`
must fight *free-while-read* — freeing the buffer while another thread is still reading
— with `Release` on the counter decrement and a `fence(Acquire)` before freeing. We
contrast it with Part 5's *publish* ordering to see two different hazards.

**[Part 7 — `from_vec` and the bit-packing trick.](07_from_vec_and_bit_tagging.md)**
How does one 8-byte slot (`data`) hold both a buffer pointer and a counter pointer, and
tell the two apart? The trick borrows the *lowest bit*, captured in one sentence: **VEC
odd, ARC even**. Because a `u8` buffer can be even or odd, we need *two* vtables
(`EVEN`/`ODD`) — and this part shows why one vtable is *not enough information*, rather
than laziness.

**[Part 8 — full promotable, and O(1) `slice`.](08_promotable_and_slice.md)**
Assembling it all: the four dispatch functions, the `promote_vec` function with its CAS
and its *losing* branch (where `actual` differs from `shared` — the classic bug), and
clone-then-narrow `slice`. The closing point: `slice` doesn't just *obey* the "VEC is
never cut" invariant — it *enforces* it by construction, and that very invariant is
what makes recovering `cap` by arithmetic safe.

## How to read it

Read in order — each part builds directly on what the previous one just established.
Each part is about 15 minutes, self-contained, opening where the last one left off
and closing on the question the next one picks up.

## Scope

The design track (1–5) is about *why*, deliberately skipping code details so the model
comes out clearly. The implementation track (6–8) picks up exactly those details —
function signatures, the memory-ordering discipline of the refcount, the bit-packing
trick, the CAS race — and writes them out fully enough to type along. If you only want
to *understand* the design, reading through Part 5 is complete; if you want to *rewrite*
`Bytes`, go on to the last three parts.

## Glossary (quick reference)

We keep the English terms as-is; here are one-line definitions so you don't have to
leave the post to look them up:

- **`deref`** — getting the `&[u8]` slice out of a `Bytes` (via the `Deref` trait). The
  *read* path for the data, cheap and never touching the ownership side.
- **refcount** — the counter of how many handles currently share a buffer; hitting 0
  frees it.
- **CAS** (*compare-and-swap*) — the atomic operation "if you're still X then change to
  Y", with no thread slipping in between. The foundation of lock-free updates.
- **`Release` / `Acquire`** — a pair of *memory-ordering* labels: one side *publishes*,
  the other *subscribes*; they only take effect as a pair on the same variable.
- **UB** (*undefined behavior*) — behavior that isn't defined; once you hit it, the
  compiler is allowed to do *anything*, and the fault is usually silent.
- **`Miri`** — an interpreter that runs Rust code under a weak memory model, to *catch*
  UB in `unsafe` code (use-after-free, double-free, data race) that `cargo test` misses.

*Bản tiếng Việt: [`../vi/00_index.md`](../vi/00_index.md)*
