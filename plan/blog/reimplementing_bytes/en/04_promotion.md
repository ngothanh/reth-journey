# Part 4 — The wall: when clone breaks everything

Three of the four behaviours are done, and they were surprisingly easy. One remains:
cloning a `Bytes` that owns a sole region of memory. It doesn't sound like much. But
this is where the whole design hits a wall, and this wall is hard not because of
syntax — it's hard because it goes against an assumption we implicitly trust the
whole time we write Rust.

We'll go very slowly here, because if you get this, everything in Part 5 (atomics,
CAS, memory ordering) becomes a natural consequence rather than a pile of
disconnected concepts.

## "Sole-owned" is a promise, not a description

Take the counting-frees way of thinking from Part 3, and try the most naive clone:
cloning a sole-owned region returns another sole-owned region, just like we copied a
constant.

Say `b1` is a sole-owned `Bytes`, pointing at memory at address `0xAAAA`. We clone it
into `b2`. The naive way makes `b2` sole-owned too, also pointing at `0xAAAA` — *the
same memory*. Now count:

```
b2 is released → it's sole-owned → free 0xAAAA   ← time 1
b1 is released → it's sole-owned → free 0xAAAA   ← time 2 💥
```

Count is 2. Double-free. Why can't we be "cleverer" to avoid it?

Because "sole-owned" means "I am the *only* owner". This isn't a passive label
describing a state — it's a *promise* that the free function *trusts* in order to
dare to free. A sole-owned region's free function doesn't go check "is anyone else
holding this"; it *assumes* it's the only one, because that's the clause of the
sole-owned label.

Compare to a constant: we copy a constant freely, because "nobody owns it" doubled is
still "nobody owns it". But copying a *sole-owner* gives *two owners*, and both still
wear the "only" label, and both will free.

Here's the crux, and where many people slip: the act of cloning *makes `b1`'s "I'm
the only one" promise false* — even though we never touched `b1`. `b2` merely coming
into existence turns `b1` into a liar. People usually think `clone`'s only job is "make
a copy". But here `clone` has a second, hidden job: *fix the original's state so it
stops contradicting itself.* Whoever causes the problem has to clean it up.

## The only escape: promote to shared

There's no safe way to write a `clone` that returns a sole-owned region. To escape,
`b1` has to *stop being the sole owner*. Concretely, we move both handles to the third
way of owning — shared, with a counter. This process is called *promotion*, promoting
from sole-owned up to shared.

We allocate a small new block to hold the counter — call it a `Shared` block — and
wrap it in an `Arc` so `Arc` handles the atomic counting and the free-at-zero. The
important thing: this `Shared` block only *points at* the payload; the payload itself
*doesn't budge*. So this is still zero-copy — we don't re-copy the byte run, we just
allocate a small extra place to count.

Then we move *both* `b1` *and* `b2` to shared, both pointing at that `Shared` block,
counter set to 2.

```
        payload (stays put)
           ▲              ▲
           │              │
     b1: shared     b2: shared
           │              │
           └──► Shared ◄──┘      counter = 2
                (Arc)
```

Now recount: each handle, when released, decrements the counter by one. The memory is
freed exactly once, when the counter hits zero. The number is back to 1.

Note a detail that matters later: the `Shared` block has to remember the *original
address* of the allocated memory, not necessarily the pointer the handle currently
holds — because taking a sub-range can push the handle's pointer forward, but when
returning it to the allocator we still have to return the exact original pointer it
handed out.

## Promotion is a one-way street

There's a chain of states here:

```
constant ────────────────────────────────────  (never changes)

sole-owned ──(first clone)──► shared ──(more clones)──► shared ──► ...
             promotion
             ◄─── no reverse direction ───
```

Why does *shared never go back to sole-owned*? Because once there are two or more
handles sharing, no handle can tell whether it's the last one without a counter. To
go back to sole-owned you'd have to drop the counter — but dropping it loses the
ability to count, and if there are still two or more handles that's a double-free
waiting to happen. So once you're shared, you stay shared.

(In theory, if the counter drops back to 1, you *could* demote to sole-owned to avoid
the atomic cost. The real `bytes` crate doesn't — the complexity isn't worth the
gain. That's a "decision *not* to do it" worth noting: sometimes good design is
knowing where to stop.)

This also explains a name. `b1`'s label starts as "sole-owned", but it *may become*
"shared". In Part 5 we'll see this label can't fix itself, so instead of calling it
"owned" we'll call it a name reflecting the possibility of change — *promotable*, "can
be promoted". But the deep reason for the name is a technical constraint from Part 5;
for now just understand "sole-owned" here to mean "currently alone, but ready to go
shared".

## The unusual thing: mutating back into an already-existing value

Now the thing that makes this part hard *conceptually*, not code-wise.

In ordinary Rust, a value's way of owning is *fixed the moment it's born*. A `Vec`
born a `Vec` stays a `Vec` until it dies. An `Arc` born an `Arc`. You never "convert"
a live value from one way of owning to another — you create a *new* value and drop
the old one.

Here it's completely different. `b1` was born sole-owned, but *gets turned into shared
mid-life*, by *another value* — `b2` — at the very moment `b2` is being created. `b1`
doesn't change itself; it *gets* changed by `b2`.

This very unusual thing is what spawns all the remaining complexity. For `clone`
(running on behalf of creating `b2`) to fix `b1`, it has to satisfy three *independent*
requirements.

The first requirement is that there must be a *path to* `b1`'s field. Right now, when
`clone` runs, it receives `b1`'s `data` as a *copy* — 8 bytes copied out. Assigning a
new value to that copy leaves the original `b1` none the wiser. To be able to fix
`b1`, `clone` has to receive a *reference to* the real field, not a copy.

The second requirement is that it must be able to *write* through that path. Even with
a reference to `b1`'s field, `clone` has only a *shared, read-only* reference to `b1`
(because `clone`'s signature in Rust is `&self`). Writing through a read-only
reference is something Rust *forbids* by default. It needs a special mechanism.

The third requirement is that it must be *safe when multiple threads do it at once*.
`Bytes` will be forced to be sendable across threads (Part 5 explains why it's
mandatory). When it is, two threads can both hold a reference to `b1` and both call
`clone`, both trying to promote it. Done naively, the two threads allocate two
counters, one gets abandoned — a leak, or worse.

These three requirements are from three completely different worlds — one about how
arguments are passed (a copy or a reference), one about Rust's borrowing rules
(writing through a read-only reference), one about the multithreaded memory model.
They know nothing of each other. And yet, as Part 5 will show, all three point at
*one* single change to the type of the `data` field.

## The signature has to change

Concretely, the vtable's functions have to shift from "take `data` as a copy" to
"take a reference to `data`". And there's a small but interesting detail: the `clone`
function will take a *shared* reference (read-only, since it has `&self`), while the
`drop` function gets an *exclusive* reference (`&mut self`, since when a value is
being destroyed, certainly no other thread still holds it). This difference — "clone
gets shared, drop gets exclusive" — sounds minor, but in Part 5 it has a very concrete
consequence: `drop` reads `data` without needing atomics, while `clone` does.

## What we have, and what Part 5 solves

Part 4 ends here: cloning a sole-owned region is a double-free, so *promotion* —
promoting both handles up to shared — is the only escape; we allocate a `Shared` block
with a counter, move both handles to it, the payload doesn't move. Promotion is
one-way. And the core unusual thing: `clone` has to *mutate back* into `b1` — an
already-existing value — because the act of cloning falsifies `b1`'s promise.

That back-mutation poses three independent requirements: a path to the field, the
ability to write through a read-only reference, and multithread safety. Part 5 shows
all three converge on a single field type, and each requirement corresponds to a piece
of the concurrency puzzle — interior mutability (writing through a read-only
reference), CAS (picking exactly one winner when racing), and memory ordering
(guaranteeing the other thread *sees* what was just promoted). It's the most abstract
part of the series, and we'll go slowly.

---

*Next: [Part 5 — `AtomicPtr`: mutating back, safely](05_atomics.md) ·
[Index](00_index.md)*

*Tiếng Việt: [`../vi/04_promotion.md`](../vi/04_promotion.md)*
