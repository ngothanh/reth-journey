# Part 3 — Separating "which bytes" from "who owns them"

Part 2 left a debt. We have the mechanism for one `Bytes` type carrying three
cleanups, but we haven't answered the second half of the requirement: reading bytes
has to stay *fast*. A `Bytes` now has four fields — pointer, length, `data`, `vtable`
— many more than `Arc<[u8]>`, which had just one pointer. Does all that "more" make
reading slower?

The answer is no, and *why* it's no is the prettiest idea in the whole design. It's
also what makes this design *beat* a more obvious approach we'll compare at the end.

## A handle answers two questions that have nothing to do with each other

Look again at a `Bytes` and notice that its four fields actually split into two
groups, answering two entirely separate questions.

Question one: *which bytes?* — where they are, how long they are. Answered by the
pointer and length.

Question two: *who owns them?* — is this run of bytes a constant, sole-owned, or
shared; and that in turn decides what to do on clone or on release. Answered by
`data` and `vtable`. (Note: `clone` doesn't "free" anything — it duplicates a
handle; only `drop` releases. Both live in this group because both *depend on who
owns the memory*, not because both clean up.)

```rust
struct Bytes {
    ptr:    /* pointer */,       // ┐ "which bytes"
    len:    /* length */,        // ┘
    data:   /* 8 extra bytes */, // ┐ "who owns them"
    vtable: /* table pointer */, // ┘
}
```

The crux: the byte contents are *always* just a raw run of bytes, whether they came
from a constant, a sole-owned region, or a shared region. There's no "hidden shape"
to discover later. So the pointer and length *fully* answer question one, and
question one *never needs* the answer to question two.

The reverse isn't symmetric: `clone` and `drop` need `data`/`vtable`, but they *also*
need the pointer and length (to build a new handle, to free the right address). So
the "who owns them" group reads both groups; but the "which bytes" group reads only its
own. That asymmetry is exactly what we exploit.

One way to picture it: the pointer and length are like a book's *shelf position and
page count*. `data` and `vtable` are like the *lending card* stuck inside the back
cover: who's borrowing this copy, what to do on return. The lending card tells you
nothing about what's written in the book — and you read the whole book without
glancing at the card once.

## And so reading is free

The direct consequence: every read operation — getting the contents, the length,
comparing, hashing, printing — touches only the pointer and length. No `vtable`
lookup. No branch on ownership kind. No counter touched. Getting the contents out is
just "from this pointer, read this many bytes" — one line, and *exactly* what
`Arc<[u8]>` also compiles to.

This is the entire reason reads stay cheap. The two newly-added fields (`data`,
`vtable`) cost nothing on the hot path, simply because the hot path only reads, and
reading never sees them. The price of the flexibility — one type carrying three
cleanups — is dumped *entirely* into `clone` and `drop`, two *cold* operations that
rarely run; and it *doesn't leak* into reading, the *hot* operation that runs
constantly.

This is a design principle usable everywhere, not just for `Bytes`: when you add
state for the sake of flexibility, lay it out so the state sits *off the hot path*.
If the hot path is forced to *look at* the new state — even one branch — then the
flexibility has leaked its cost into the most expensive place.

## Why not just use an `enum`

By now many people will ask: why go to all this trouble with a vtable and function
pointers, when Rust has a built-in `enum` to represent "one of three possibilities"?

```rust
enum Bytes {
    Static { /* ... */ },
    Owned  { /* ... */ },
    Shared { /* ... */ },
}
```

This is *correct*. It's even *safer* (no `unsafe` code). So why doesn't the real
design pick it?

Because an `enum` puts the discriminating tag (the "who owns them" question) *together*
with the data (the "which bytes" question). Every read, you have to `match` that tag
— a branch — to pull out the pointer and length, *even though reading bytes has
nothing to do with the tag*. You pay for "who owns them" on *every* asking of "which
bytes".

With our flat layout — pointer and length always at the same fixed position for all
three kinds — reading pulls them straight out, no branch. The `vtable` pointer sits
off to the side, touched only by `clone` and `drop`.

The trade-off here is real, and worth stating plainly: choosing the flat layout means
you lose the `enum`'s static safety (you write `unsafe` code and uphold the "`data`
must match `vtable`" invariant by hand), in exchange for branch-free reads. For a
type whose read operations are called constantly in hot loops, that trade is worth
it. For a type that's rarely read, the `enum` is the correct choice. Knowing where
you sit on that spectrum is part of the design skill — "faster" doesn't always win.

## The backbone way of thinking: counting frees

Now to the "who owns them" group, laying the foundation for the two hardest parts. The
three ways of owning sound different, but they're really three answers to *the same*
question:

> How many times, exactly, does this region of memory get freed, and by whom?

- A constant: freed **0** times. It was never allocated; you can't return what you
  never borrowed.
- A sole-owned region: freed **1** time, by the handle itself, when it's released.
- A shared region: freed **1** time, by the *last* handle, when the counter hits
  zero.

The correct number is always as above. And here's what turns this question into a
tool rather than a slogan: *every bug in this design reduces to miscounting that
number.* Counting **0** when it should be 1 is a memory leak. Counting **2** when it
should be 1 is a double-free or use-after-free. Throughout Parts 4 and 5, whenever
you're unsure "is this right", you just ask: *for this exact region, what did I just
make the count?*

From this way of thinking, two important things emerge.

## `Drop` doesn't clean up the struct — it undoes one allocation

Look again at `Bytes`'s four fields: a pointer, a number, `data` (a pointer or a
number), `vtable` (a static reference). *None of them owns anything.* If you deleted
the `impl Drop for Bytes` block entirely, releasing a `Bytes` would already be a
perfect no-op — the struct vanishes off the stack, no help needed.

So what does `Drop` exist for? *Only* to return the region of heap memory the struct
points at. This is where intuition often goes wrong: `Drop` isn't there to "clean up
the value itself" — the value dissolves on its own. `Drop` exists solely to *undo a
prior allocation*. If nothing was ever allocated, there's nothing to undo.

This is exactly why a constant's `drop` function is an *empty* one, and that
emptiness is *correct*. A byte constant was never allocated, so it must be freed 0
times, so its cleanup function does nothing. Put another way: no allocation, no
`Drop`. Leaking a constant is *correct* — it lives for the whole program's lifetime
no matter what you do.

## The silent trap: the biggest lesson about unsafe code

A sole-owned region's free function must return exactly the number of bytes that were
*allocated*, not the number that were *written*. Recall from Part 1: a `BytesMut`
might have allocated 1024 bytes but only written 7. When freeing, the allocator wants
back the exact 1024-byte block it handed out — it matches on the *allocated size*,
not on the contents.

What happens if you accidentally return based on bytes written (7) instead of bytes
allocated (1024)?

On Linux and macOS, the final free operation calls down to C's `free(ptr)` — which
takes *one* argument, the pointer; it looks the size up from metadata hidden just
before the block, and *discards* the size you passed in. The result: the program
**doesn't crash**. Tests all pass. Runs ten million times, still passes. Runs two
years in production, still passes.

But it's undefined behaviour. The free operation's contract requires the
size-at-free to equal the size-at-allocation. The day it blows up is the day someone
swaps the default allocator for another — say `jemalloc` or `mimalloc` — the kind
that *trusts* the size you pass and uses it to pick a bucket. It returns the
1024-byte block into the bucket for 8-byte blocks; a few thousand allocations later,
two parts of the program write to the same region; and you have heap corruption in a
completely unrelated place, with no way to trace it back.

This is the lesson most worth burning in about unsafe code, and it's the inverse of
intuition:

> The scary bug in unsafe Rust isn't the one that crashes the program — it's the one
> that runs *correctly*. The intuition from safe Rust — "wrong means panic
> immediately" — is inverted here: the default for a mistake is *silence*.

The tool that catches it is `miri` — an interpreter that doesn't run real `free` but
*checks the contract*: it remembers the size at allocation, compares it at free, and
shouts "incorrect layout on deallocation" immediately, at the right line. This is
also why any dual-purpose field — like `data` being a number one moment and a pointer
the next — must be documented right at the spot and checked with a test: the mistake
doesn't surface itself at runtime.

## Three "easy" behaviours fall out

Holding the counting-frees way of thinking, three of the four behaviours reveal
themselves — and the nice thing is *none of them touches multithreading yet*. This is
why you build them first; Parts 4 and 5 are the hard part.

For a **constant**, freed 0 times. Creating a constant `Bytes` is just building a
handle pointing at the existing byte string, tagging its `vtable` as the constant
table, leaving `data` empty. Its cleanup function is empty. Done.

For a **sole-owned region**, this is where we finally kill Part 1's memcpy. We want
the memory freed 1 time, not 2. The problem: the old `BytesMut` *will* free the
memory when it's destroyed (it has its own built-in cleanup); then the new `Bytes`
*will also* free it. That's 2 — a double-free. To get to 1, we have to prevent the
`BytesMut` from running its cleanup.

Rust has a tool for exactly this: `mem::forget`. The name makes it sound like it
"erases a variable", but it's really a declaration:

> "I have handed this memory to someone else. Don't run my cleanup anymore."

That's the *definition* of a zero-copy handoff: the recipient adopts the sender's
memory (no copy), and one region gets exactly one cleaner. The buffer itself doesn't
budge; only the *responsibility to free it* moves from `BytesMut` to `Bytes`. So
`freeze` goes: read the pointer / length / allocated-size out of the `BytesMut`; call
`mem::forget` so the `BytesMut` won't clean up anymore; then build a sole-owned
`Bytes` pointing at that same buffer, with `data` holding the allocated size.

A subtle point: `mem::forget` is *normally a memory leak* — that's its primary (and
dangerous) purpose. Here it *doesn't* leak only because we read the pointer out and
handed it to `Bytes` *first*. `mem::forget` doesn't check that for you; *you* have to
ensure someone takes over. So the order read-out-first-then-forget is mandatory;
reverse it and the compiler stops you (you'd have handed `self` into `forget`
already). This is hard to get wrong — exactly the kind of code we like.

And the sole-owned region's free function, as discussed above: return the memory by
the allocated size read out of `data`. Remember the silent trap — allocated, not
written.

## What we have, and the wall ahead

After Part 3, we have a `Bytes` that works for *two of the three* kinds, with a free
read path, and no multithreading yet. A constant clones by copying the struct and
cleans up with an empty function. A sole-owned region's `freeze` is a constant-time
handoff — Part 1's memcpy is dead, meaning the hard requirement we set in Part 1 is
met — and it cleans up by freeing at the allocated size. The counting-frees way of
thinking is ready to serve as a diagnostic tool.

Exactly one behaviour remains: cloning a sole-owned region. And it breaks everything.
Take the counting way of thinking and try: clone a sole-owned region, and leave
*both* handles sole-owned, and both free — count is 2 — double-free.

Why it's unavoidable, and why its escape forces something very unusual in Rust —
mutating back into an already-existing value — is Part 4, and it's the hardest part
of the whole series.

---

*Next: [Part 4 — The wall: when clone breaks everything](04_promotion.md) ·
[Index](00_index.md)*

*Tiếng Việt: [`../vi/03_split_and_counting.md`](../vi/03_split_and_counting.md)*
