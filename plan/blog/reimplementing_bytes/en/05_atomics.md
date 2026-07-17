# Part 5 — `AtomicPtr`: mutating back, safely

Part 4 closed on one problem and three requirements. The problem: when cloning a
sole-owned `Bytes`, we have to *mutate back* into the original to promote it to
shared, or else double-free. The three requirements for that back-mutation: there
must be a path to the original's `data` field; we have to be able to write through a
read-only reference; and it has to be safe when several threads do it at once.

This part solves all three. And the beautiful thing is that all three, though from
three different worlds, are solved by a single choice of type for the `data` field:
making it an `AtomicPtr` — an atomic pointer. This is the most abstract part of the
series, so we'll dissect one requirement at a time, and with each, the matching
concurrency concept appears as an answer to a *concrete* problem we're actually stuck
on, not as theory in a vacuum.

## Why an atomic pointer

Recall Part 4's three requirements, because the thing worth noticing is that they come
from three unrelated worlds. The first — "there must be a path to the original's `data`
field" — is about *argument passing*: does the function receive a copy or a reference.
The second — "we must be able to write through a read-only reference" — is about the
compiler's *borrowing rules*. The third — "it must be safe when several threads do it
at once" — is about the hardware's *memory model*. Three concerns at three different
layers, none aware of the others.

Recall from Part 2 that the `data` field currently has type **`*mut ()`** — a raw
pointer, 8 bytes, "meaning deferred". That type *cannot* meet the three requirements
above: a `*mut ()` passed into a function is passed *as a copy* (fails the first); even
with a reference to it, Rust *forbids* writing through a read-only reference (fails the
second); and reading/writing a `*mut ()` from several threads at once is a data race,
i.e. undefined behaviour (fails the third).

All three failures are fixed by a single change: change `data`'s type from `*mut ()` to
**`AtomicPtr<()>`** — the same 8 bytes, the same "meaning deferred" role, but now an
*atomic* pointer. It lets you take a reference to it (solves the first). It lets you
write through a read-only reference, thanks to a property called *interior mutability*
(solves the second). And it provides atomic operations so threads don't stomp each
other (solves the third).

This is worth taking away as a lesson in itself: when someone asks "why is this field
atomic?", the correct answer is *not* "because concurrency" in the abstract. Here it's
three specific, distinct requirements that happen to be solved by the same thing.
Recognizing when several different requirements converge on a single mechanism — that's
half of systems design skill.

The next sections dissect each requirement. The first (taking a reference) is trivial —
just change the function signature to pass `&data` instead of `data`. The other two are
where the substance is, and each leads us to a concurrency concept.

## Requirement two — writing through a read-only reference: interior mutability

There's a rule in Rust, simple enough to be worth memorizing: a read-only reference
(`&T`) can *only be read*. To write through it, `T` must contain something called an
`UnsafeCell` inside.

`UnsafeCell` is the *only* thing in all of Rust that permits "mutating data through a
read-only reference". It's a hole the compiler allows, drilled straight through the
borrowing rules. Every other tool you've ever used to "write through a shared
reference" is `UnsafeCell` plus a discipline for using it safely:

- `Mutex` is `UnsafeCell` plus "you must lock before entering".
- `RefCell` is `UnsafeCell` plus "count the borrows at runtime, panic on violation".
- `Cell` is `UnsafeCell` plus "only copy in and out, never lend the insides".
- And an atomic pointer is an `UnsafeCell` holding a pointer, plus "only read/write
  with the CPU's atomic instructions".

So when `clone` has only a read-only reference to `b1` but needs to write to
`b1.data`, that field *must* contain an `UnsafeCell`. An atomic pointer is exactly
what we need: it opens the write-through-a-read-only-reference hole (the second
requirement) while also handling the multithreading part (the third).

There's a memorable symmetry here. `Arc<T>` also gives you only a read-only reference
to its insides. That's precisely *why* everyone has to write `Arc<Mutex<T>>` — `Arc`
handles the *sharing*, and `Mutex` handles the *writing*. `Bytes` faces exactly that
problem, just solved differently: both are "shared, and needs to write", but
`Arc<Mutex<Vec>>` uses a lock (because a `Vec` is big, it can't be made atomic), while
`Bytes` uses an atomic pointer (because the thing to write is exactly 8 bytes).

And here's the point worth pausing on: why does `Bytes` get to use an atomic operation
instead of a lock? Because the thing being protected is *exactly one machine word* (8
bytes on a 64-bit machine). This is a hardware fact, look it up once and remember it,
no derivation needed: a 64-bit CPU has atomic read, write, and "compare-and-swap"
instructions for exactly 8 bytes or fewer. Anything bigger than 8 bytes, *no
instruction* can do atomically — that's when you need a lock (a lock lets you
read/write several machine words in sequence under the protection of one flag). Since
`data` is only 8 bytes, it is itself "both the lock and the data", no separate `Mutex`
alongside. This is what keeps `Bytes` small, lock-free, yet thread-safe. If the thing
to promote were bigger than 8 bytes, this whole design collapses and you'd be back to
a lock.

## Requirement three — multiple threads promoting at once: Send, Sync, and CAS

Rust has two notions about data crossing a thread boundary. A value is "sendable"
(`Send`) if it's allowed to *move* to another thread. A type is "shareable" (`Sync`)
if a reference to it is allowed to be *used from* several threads. The compiler
derives both; and because `Bytes` holds raw pointers (which Rust treats by default as
not-sendable, not-shareable, since it's pessimistic about pointers), `Bytes` by
default *doesn't* have either.

But the codebase *needs* them. Caches are shared across several workers, messages are
sent over channels between threads. If `Bytes` isn't sendable and shareable, the code
won't even compile — you get exactly this error:

```
error[E0277]: `*mut ()` cannot be sent between threads safely
```

So we have to *promise* the compiler that `Bytes` is safe to send and share. The
promise is true because: the payload is immutable (many places reading it don't
collide), and the only mutable state — `data` — is atomic. This is exactly what makes
`Arc<[u8]>` sendable and shareable, and `Bytes` has that same shape.

But the promise has a price. The moment `Bytes` is shareable, two threads can both
hold a reference to `b1` (a sole-owned region) and both call `clone`. If we promote
naively — read `data` out, then write the new value in, as two separate steps — this
scenario happens:

```
Thread 1: reads data, sees "not promoted"
Thread 2: reads data, sees "not promoted"      ← slips in between
Thread 1: allocates counter A, writes data = A
Thread 2: allocates counter B, writes data = B  ← CLOBBERS A
```

Result: two counters get created, one (A) gets abandoned — a leak, or if the counting
logic goes wrong, a use-after-free. This is a classic bug called *lost update* — a
loss from overwriting — of the non-atomic "read then write" kind. The two separate
steps leave a gap for the other thread to slip into.

The fix is an operation that fuses "check" and "write" into one inseparable step,
called *compare-and-swap*, or CAS. Rendered in plain words:

> "Hey `data`, *if* you're still the old value (not promoted), then change yourself
> into my counter's pointer — and do these two things *fused*, so no thread can slip
> in between. But if someone else already changed you, then *don't* change, and tell
> me what you're holding now."

The hardware guarantees that, when several threads charge in, *exactly one* CAS wins.
The winner installs its counter into `data`; at that instant, `b1` becomes shared
(because `b1` and the `data` cell are one). The loser gets the signal "someone else
already changed it", so it throws away the counter it just allocated, and uses the
winner's counter instead. In the end there's one counter, and the memory is freed
exactly once — the counting way of thinking from Part 3 balances again.

One detail to be careful about in the loser's path: when it throws away the extra
counter, it has to do so *without* triggering the freeing of the payload — because the
payload now belongs to the winner's counter. It frees just the *shell* of the extra
block while skipping that block's payload-cleanup. Forget this detail and the payload
gets freed twice.

There's a nice name for CAS's role here: it's the *linearization point*. Even though
the two threads charge in parallel, the CAS is the marker that turns that chaos into a
clear sequence — "whoever wins the CAS is treated as having happened first". Any time
you need "exactly one of several racers gets to do X", CAS is the tool, and the
winning marker is the linearization point.

## Requirement three, continued — memory ordering: "being able to write" isn't enough, you must "see the right order"

CAS solves only half of the multithreading: it guarantees exactly one thread *installs*
the counter. But there's a second danger, subtler and entirely separate — and this is
where most people find things hardest. We'll set up the problem first, then dissect each
individual operation to see what synchronization "strength" it needs.

### The problem: writes get reordered

The winner of the promotion race does two things, in this order *in the code*: first it
initializes the contents of the `Shared` block (writing into it the memory's original
address and the length), then it *publishes* the `Shared` block's address via the CAS
that writes `data`.

The problem is that hardware and the compiler alike are allowed to *reorder* writes to
memory to go faster — they buffer, coalesce, reorder. For a single thread this is
harmless, because the final result still looks right. But for multiple threads, another
thread may see the winner's writes *in a different order* than the code's order.

The concrete disaster: a second thread calls `clone`, reads `data`, sees it's already
the `Shared` block's address, and accesses that block to bump the counter — i.e. reads
its contents. If the second thread sees *the address* but *not yet* the *contents* the
winner just initialized — perfectly legal under the reordering rules — then it reads a
`Shared` block full of garbage, and everything after that is undefined behaviour. The
address has "run ahead of" the contents it points at.

We need a guarantee: *whoever has seen the `Shared` block's address must also see its
finished, initialized contents.* This is the job of *memory orderings* — the "labels"
we attach to each atomic operation to say how far it's allowed to be reordered.

### Four strengths, and how to picture them

Rust has four labels we'll use: `Relaxed`, `Acquire`, `Release`, and `AcqRel`. The
easiest way to picture the two in the middle is "publish" and "subscribe":

- A **write** of the **`Release`** kind is a *publication*: everything I wrote *before*
  this operation, whoever reads the value I just wrote will see all of it.
- A **read** of the **`Acquire`** kind is a *subscription*: once I read the published
  value, I also see everything the publisher wrote *before* it published.
- **`Relaxed`** is "just make this operation atomic, promise nothing about ordering
  relative to other writes" — the cheapest.
- **`AcqRel`** is "both `Acquire` and `Release`", for an operation that *both reads and
  writes* (like CAS, which reads the old value and writes the new one).

The crux: `Release` and `Acquire` only do anything when they come *as a pair*, on the
*same variable*. One side publishes, the other subscribes; the pair builds the ordering
link tying two threads together. Missing one side breaks the pair, and the guarantee
vanishes.

### Dissecting each operation on `data`

Now apply this to the exact places the code touches `data` during promotion, asking of
each: which label does it need, and why.

**The first read of `data`, opening `clone`.** Before deciding whether promotion is
needed, we read `data` to see what it currently is. Label: **`Acquire`**. Why? Because
`data` may already have been promoted by another thread — it may already be a `Shared`
block's address — in which case we go straight to the "already shared" branch and
*access* that `Shared` block to bump the counter. To access it safely, we have to see
its initialized contents — so this read must be `Acquire`, to pair with the `Release`
of whoever promoted it.

**The CAS — this is the answer to "why AcqRel".** Something many people overlook:
`compare_exchange` carries *two* ordering labels, not one — one for the *success* case,
one for the *failure* case. That's because a CAS has two completely different outcomes,
and each needs a different guarantee.

- *When the CAS fails* (someone else promoted first): the CAS returns the current value
  of `data` — which is the winner's `Shared` block address. And right after, we're going
  to *use* that address (bump the other `Shared` block's counter). That means we're
  about to access a `Shared` block another thread initialized — so the failure case must
  be **`Acquire`**, for exactly the same reason as the opening read.
- *When the CAS succeeds* (we're the winner): we've just *published* the `Shared` block
  that *we ourselves* built just above. For another thread to later read this address
  and access the `Shared` block without hitting garbage, this write must be
  **`Release`**.

So the failure case needs `Acquire`, the success case needs `Release`. And here's the
key: Rust requires the *success* label to be no weaker than the *failure* label. But
`Release` alone does *not* include `Acquire` (they're two different directions — one
handles the write side, one the read side). So for the success case to carry `Release`
(for its own publication) *and* be strong enough relative to the failure case's
`Acquire`, it has to carry *both* — and that "both `Acquire` and `Release`" label is
exactly **`AcqRel`**.

Put shortly, `AcqRel` for the CAS isn't chosen "to be safe" — it's the single label
that satisfies two things at once on the same instruction: the loser must *receive* the
winner's `Shared` block (`Acquire`), and the winner must *publish* its `Shared` block
(`Release`).

**The read of `data` in `drop` — no atomics needed at all.** Because `drop` takes an
*exclusive* reference to the value (recall Part 4), it knows for certain no other thread
still holds it — no race, so a plain read suffices. The exclusive reference is itself
proof of no race. This is exactly why `drop`'s signature takes an exclusive reference
while `clone` takes a shared one: not style, but "drop has exclusivity, clone doesn't".

**The edge case: a constant `Bytes`.** `data` is empty, synchronizing nothing with
anyone — so every touch of it needs only **`Relaxed`**, the cheapest label.

### Why not just use `SeqCst` to be safe

`SeqCst` (sequential consistency) is the strongest label Rust has — it forces *every*
`SeqCst` operation in the whole program into one single global order that all threads
agree on. It sounds safe, and many people reach for it "to be sure". But here it's both
*overkill* and *expensive*. Overkill, because all we need is a *pairwise* link between
one publisher and one receiver on a single variable, `data` — not a whole-program
agreement on some global order. Expensive, because `SeqCst` usually has to insert
stronger memory fences, slowing down exactly the path the whole design exists to keep
fast. Picking the minimum strength actually needed — `Acquire`/`Release`/`AcqRel` where
it matters, `Relaxed` where it doesn't — is part of writing lock-free code properly.

The principle to take away: `Release` and `Acquire` always come in a pair on the same
variable, linking a "publication" to a "subscription"; an operation that *both reads and
writes* where both roles matter (like CAS) needs `AcqRel`; and whenever you publish to
another thread a pointer to freshly-initialized data, you *always* need this pair —
otherwise the data can "arrive after" the pointer.

## A compact consequence: why the label is renamed "promotable"

The final detail, and it's a direct consequence of "mutating back" hitting a limit.

When promoting, `clone` can write `b1.data` (thanks to the atomic pointer) but
*cannot* write `b1.vtable` — because `vtable` is an ordinary field, *without* interior
mutability, and `clone` has only a read-only reference. So after promotion:

```
b1.vtable = still the "sole-owned" table   ← stuck, unchangeable, now lying
b1.data   = now the Shared block's address  ← changed (via CAS), telling the truth
```

`b1` forever dispatches through the old table, even though it's actually shared now.
So that table, on its *very first instruction of every call* (both `clone` and
`drop`), has to ask itself: is my `data` a length number (not promoted) or a `Shared`
block address (promoted)? — and branch accordingly. For exactly this reason the label
is *not* "sole-owned" but "promotable" — "this one *may already have* become shared".

Looking a little deeper, this is a general law: the discriminating marker *must live in
the mutable cell* (`data`, atomic), *not in the immutable vtable pointer*. It's an
inevitable consequence of ownership changing mid-life: the vtable pointer freezes the
moment the value is born, so it can't be where dynamic state lives. Whenever a value's
state changes *after* it's born, the marker of that state has to sit in the *mutable*
part, not the *immutable* part — and here, only `data` is mutable.

(An aside, *not* part of the model: how does `data` hold both a length number and a
`Shared` block address in the same 8 bytes, and how does the "promotable" table tell
the two apart? There's a trick that borrows the *lowest bit*: a `Shared` block's
address is always even — due to memory alignment rules — so its lowest bit is always
0; we set the low bit to 1 when we pack in a number, and just checking the low bit
tells us which kind is stored. This is purely a *space optimization* — replacing it
with a separate field to hold the length is perfectly correct too, just one machine
word bigger. The `bytes` crate packs the bit because it counts every byte; a
built-to-learn version needn't.)

## Summary: the complete design

Here is the whole design, side by side, one last time. Compared to the summary at the
end of Part 2, exactly *one* line changed — the type of `data` — but all of Parts 4 and
5 were spent explaining that one line.

```rust
struct Bytes {
    ptr:    NonNull<u8>,      // "which bytes": points at the start of the run
    len:    usize,            // "which bytes": how long
    data:   AtomicPtr<()>,    // "who owns it": 8 atomic bytes; *mut () in P2, now atomic
    vtable: &'static Vtable,  // "who owns it": which clone/drop set to use
}

struct Vtable {
    clone: unsafe fn(&AtomicPtr<()>, ptr, len) -> Bytes,  // & so clone can fix the original
    drop:  unsafe fn(&mut AtomicPtr<()>, ptr, len),       // &mut: exclusive, read non-atomically
}
```

The three ways of owning, and what `data` holds in each:

| vtable              | what `data` holds                   | what `clone` does          | what `drop` does |
|---------------------|-------------------------------------|----------------------------|------------------|
| `STATIC_VTABLE`     | null (unused)                       | copy the struct            | nothing          |
| `PROMOTABLE_VTABLE` | length (number), *or* a counter address once promoted | if not yet promoted: build a counter, CAS up to shared; if already: bump the counter | if not yet: free; if already: decrement |
| `SHARED_VTABLE`     | the counter's address (real pointer)| bump the counter           | decrement        |

Reading the data (`deref`, `len`, compare, hash) touches only `ptr` + `len` — never
`data` or `vtable` — so it's as cheap as `Arc<[u8]>`. `data` + `vtable` come into play
only on `clone` or `drop`. That's the whole design.

The path from the start to here, in one picture:

```
Arc<[u8]>            P1: counter fused to payload ⇒ freeze is FORCED to copy
   │
   ▼ need O(1) freeze ⇒ Bytes must ADOPT the memory, not copy it
Bytes{ ptr, len, data, vtable }
   │  P2: lower "ownership" from the type into a vtable (one type, three behaviours)
   │      data: *mut ()  — 8 bytes, "meaning deferred"
   │  P3: split "which bytes" (ptr,len) from "who owns it" (data,vtable) ⇒ free reads
   │      + the mindset: how many times is each region freed? (0/1/1)
   │
   ▼ P4: cloning a sole-owned region = double-free
   │      ⇒ promotion: mutate BACK into the original to push it up to shared
   │
   ▼ P5: back-mutation needs &data + write-through-& + thread-safety
          ⇒ data: *mut ()  ➜  AtomicPtr<()>
          interior mutability · CAS · Acquire/Release/AcqRel
```

## Five questions to carry into every later problem

Close the series, and forget vtables, atomics, CAS. What's worth carrying — into later
problems about write-ahead logs, about sharing nodes in a skiplist, about caching data
blocks, and into any design touching unsafe, ownership, or optimization — is these five
questions. They are what the whole story distills to.

First: *how many times, exactly, is each region of memory freed?* 0 is a leak, 2 is a
double-free, 1 is right. Every bug reduces to this number.

Second: *what differs between the cases?* Only that needs dispatch. Whatever is the
same, leave alone — that's precisely why the hot path is free.

Third: *does the hot path touch dispatch?* If it does, the design is wrong. Reading
must be as cheap as `Arc<[u8]>`.

Fourth: *is there a write through a read-only reference?* If there is, you need
interior mutability. A bare read-only reference is always read-only.

Fifth: *are there multiple threads?* If there are, use atomic operations instead of
plain read/write; an "exactly one winner" operation calls for CAS; and every time you
publish a pointer-to-data for another thread to read, use a release/acquire pair on the
same variable.

And three closing lines about how to think, scattered across the series. `Drop`
doesn't clean up the struct — the struct dissolves on its own; `Drop` only undoes one
allocation, so no allocation means no `Drop`. A vtable is a type demoted from the
compile-time level to a runtime value, used when one type needs several behaviours
chosen per value. And the scary bug in unsafe isn't the one that crashes the program,
it's the one that runs correctly — the intuition from safe Rust is inverted, the
default for a mistake is silence, so always carry `miri` to make it speak up.

By now you have enough of the model in your head to sit down and reimplement `Bytes`
from scratch — the four behaviours for three ways of owning, the `freeze` operation,
and promotion — and argue for every choice. The remaining code details will reveal
themselves as you write, because you understand why each one exists.

---

*Back to: [Part 4](04_promotion.md) · [Index](00_index.md)*

*Tiếng Việt: [`../vi/05_atomics.md`](../vi/05_atomics.md)*
