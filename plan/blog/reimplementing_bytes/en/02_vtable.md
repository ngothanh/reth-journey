# Part 2 — One type, many behaviours

Part 1 ended on a tension. We need a single `Bytes` type (because the API boundary
demands it), but that type has to know three different ways of cleaning up memory —
do nothing for a constant, free the block for a sole-owned region, count-then-free
for a shared region — and which one applies is only known at runtime. This part
finds the mechanism that makes that possible.

But before inventing anything new, it's worth looking closely at how Rust *normally*
solves this. Because it turns out our design isn't novel at all — it just mimics what
Rust already does, one level down.

## Normally, "how to clean up" already lives in the type

Look at three familiar types, and notice what decides how each gets cleaned up:

```rust
&'static [u8]   // a constant byte string, alive for the whole program
Vec<u8>         // an array of bytes we own
Arc<[u8]>       // a shared array of bytes, with a counter
```

When a `Vec<u8>` goes out of scope and is destroyed, the compiler runs the code that
frees its memory. When an `Arc<[u8]>` is destroyed, the compiler runs the code that
decrements the counter, and only frees if it hits zero. When a `&'static [u8]` is
destroyed, the compiler runs nothing at all — it's a reference to a constant, there's
nothing to clean up.

The notable thing: **you never have to tell the compiler which code to run.** It
looks at the *type* and knows. A `Vec` uses `Vec`'s way, an `Arc` uses `Arc`'s way.
This decision is settled *at compile time*, and it's free — there's no "ask at
runtime what kind this is" step, because the type already answered ahead of time.

Put another way, in Rust a *type* isn't just the shape of the data. It also *carries*
a subroutine — how to clone, how to drop — that the compiler looks up and wires in.
You can think of a type as an invisible, static lookup table of behaviour.

This is precisely why Rust is fast: each way of owning is its own type, every cleanup
decision is settled at compile time, and the running program doesn't have to think.

And this is exactly what Part 1's "three separate types" was trying to exploit — to
have the compiler do it for us. It died for one reason only: the API boundary needs a
single type. So now we have to do by hand what the compiler normally does silently —
but do it at runtime.

## The collision: one type means one `Drop` function

The "one type" constraint runs straight into how Rust attaches behaviour to types.
In Rust, to define "what happens when a value is destroyed", you write a `Drop`
block:

```rust
impl Drop for Bytes {
    fn drop(&mut self) {
        // What do we write here now?
    }
}
```

But this block is written *once* for the `Bytes` type. And you can't write a single
line that's correct for all three cases:

- Write "free the memory" and it's right for a sole-owned region, but for a constant
  it frees memory we don't own — the program crashes.
- Write "do nothing" and it's right for a constant, but for a sole-owned region it
  leaks — the memory is never returned.
- Write "decrement the counter" and it's right for a shared region, but a constant
  and a sole-owned region have no counter to decrement.

There's no *fixed* line of code correct for all three, because which kind applies is
only known at runtime, per value — this `b1` is a constant, that `b2` is sole-owned,
that `b3` is shared. The compiler is out of options: it picks behaviour *by type*,
and we have only one type.

So the "which behaviour" decision has to leave the compiler and move to runtime. The
question is how.

## The idea: put "how to clean up" inside the value itself

If the compiler can't pick at compile time, then let *the value carry the pick
itself*. And to pick at runtime, the thing you pick with has to be *data inside the
struct*, not the type.

Concretely: we take exactly what a "type" carries invisibly — the clone/drop
subroutine — and turn it into a visible field. In Rust, "a subroutine you can store
in a variable" is a *function pointer*. So we gather two function pointers into a
little table:

```rust
struct Vtable {
    clone: /* pointer to a fn: "to clone this kind, run this" */,
    drop:  /* pointer to a fn: "to drop this kind, run this"  */,
}
```

("Vtable" is the traditional name for this kind of table — *virtual table*, a table
of virtual functions.)

Then we make three of these tables ahead of time, one per way of owning, and let
them live for the whole program:

```rust
static STATIC_VTABLE: Vtable = /* constant-style clone, drop */;
static OWNED_VTABLE:  Vtable = /* sole-owned-style clone, drop */;
static SHARED_VTABLE: Vtable = /* shared-style clone, drop */;
```

And every `Bytes` value carries a pointer to one of those three tables:

```rust
struct Bytes {
    /* ... where the bytes are ... */
    vtable: &'static Vtable,   // this pointer decides the value's fate
}
```

Now `Bytes`'s `Drop` function has just one job: *read the `vtable` field, then call
the `drop` function inside it*. A value holding `STATIC_VTABLE` calls into the
do-nothing function; a value holding `OWNED_VTABLE` calls into the free function. One
`Drop` block, three different behaviours, each chosen correctly per value at runtime.
Exactly what we needed.

The cleanest way to see what just happened:

> The vtable *is* the type, merely demoted from the compile-time level down to a
> runtime value. The three types `&'static [u8]`, `Vec<u8>`, `Arc<[u8]>` don't
> vanish — they become three values `STATIC_VTABLE`, `OWNED_VTABLE`, `SHARED_VTABLE`
> that we can assign to a field, compare, and call, at runtime.

### You've actually used a vtable already

If you've ever written `&dyn SomeTrait` in Rust, you've used a vtable — the compiler
just built the table for you. A `&dyn Trait` is internally a pair of pointers: one to
the data, one to the table of the trait's methods. When you call a method through
`dyn`, the program looks the right function up in that table, then calls it. That's
exactly the mechanism we're building by hand.

The one difference between `dyn` and what we're doing:

- With `dyn`, each case is a *different type* (a `u64`, a `String`...), so the
  compiler knows which table to build for which type.
- Here, all three cases are *already the same type* `Bytes`. The compiler has nothing
  left to distinguish, so it can't build the table — we build it by hand, and assign
  which table goes with which value ourselves, at the moment we create the value.

This gives us a rule we can reuse later: when should you hand-write a vtable? Exactly
when *one* type needs *multiple* behaviours, chosen per value at runtime. Multiple
types → use `dyn`. One behaviour → use a plain function. A hand-written vtable fills
exactly the "one type, many behaviours, per-value" slot.

## Why the table has exactly two slots

This is the question anyone designing such a type should be able to answer, because
it's the test of whether you *understand* the design or just copied it. We examine
both directions: why not *fewer* than two, and why not *more* than two.

There's a simple rule for deciding. A function slot earns a place in the vtable only
when that operation's behaviour *changes depending on who owns the memory*. If an
operation does the same thing regardless of ownership, putting it in the vtable is
pointless — worse, it's harmful, as we'll see.

Try listing *everything* a `Bytes` can do, and asking of each "does this depend on
ownership":

- Reading bytes out (getting the length, getting the contents, comparing, printing):
  for all three kinds, the answer is "just look at the pointer and the length and
  read". *Doesn't* depend on ownership.
- `clone`: a constant copies the whole struct; a sole-owned region has to do
  something complicated (Part 4); a shared region bumps the counter. *Does* depend on
  ownership.
- `drop`: a constant does nothing; a sole-owned region frees; a shared region
  decrements. *Does* depend on ownership.

Exactly two operations depend on ownership. So exactly two slots: `clone` and `drop`.

Why not *fewer*? Could we merge `clone` and `drop` into one slot? No — they're two
independent operations at two different moments (one when duplicating a handle, one
when releasing a handle), and neither's behaviour follows from the other's. Drop the
`drop` slot and you don't know how to free; drop the `clone` slot and you don't know
how to duplicate. Each depends on ownership in its own way, so each needs its own
slot.

And the "reading bytes" half — why *no* slot? Because it doesn't depend on ownership.
The pointer and length fully answer "which bytes", identically across all three
kinds. Giving it a vtable slot would charge an *indirect function call* on every
read — for an operation that doesn't care who owns anything. This is the seed of
Part 3, but it's visible already: an operation that doesn't depend on ownership is
*forbidden* from the vtable, because putting it there makes the hot path pay for
nothing.

Why not *more*? There are two natural objections.

First, "add a `slice` slot?" — because taking a sub-range of a `Bytes` *seems* to
depend on ownership (slicing a constant yields a constant; slicing a sole-owned
region can't yield another sole-owned region, or you'd have two owners of one block).
But `slice` **can be written using `clone`**: taking a sub-range is just "duplicate
the handle (so `clone` handles the ownership part), then shrink the pointer and
length to the sub-range". `clone` already knows how to duplicate correctly for all
three kinds. A slot that's *reconstructible from another slot* doesn't earn its
place.

Second, "collapse it all into one slot returning an enum that says which kind it is,
then branch on it?" — i.e. one `kind()` function returning the kind, then `clone` and
`drop` each `match` on it. It works, but you dispatch *twice*: one indirect call
(calling `kind`), *then* one branch (`match`). Whereas the whole appeal of a function
pointer is that *calling it is already the dispatch* — one step. And the enum
approach is *closed*: adding a fourth way of owning means editing *every* `match` in
the codebase; with a vtable, you just add one new `static` table and touch nothing
else.

To sum up: exactly two slots, because exactly two operations depend on ownership and
aren't reconstructible from each other. Fewer loses capability; more is either
redundant (reconstructible) or slow (dispatch twice). The rule to carry: a slot earns
its place if and only if it depends on hidden state *and* isn't reconstructible from
the other slots.

(A practical note for the curious: the real `bytes` crate has five slots, not two.
The three extra ones are *pure optimizations* — each avoids a measured copy. For
example, "turn into a `Vec`" on a buffer that happens to be sole-owned can hand the
memory over directly instead of copying — but only if it can ask "am I sole-owned
right now?", which only the vtable knows. Every extra slot could be rebuilt from
`clone` plus `drop`; they exist because someone *measured* a copy worth avoiding.
Each extra slot costs an indirect call, so each has to *earn its seat* with a
benchmark. This learning build does two slots — that's the correct minimal set.)

## The vtable's companion: a data field

The vtable says *how* to clone/drop. But "how" usually needs an accompanying *piece
of data*. A sole-owned region's free function needs to know how long the allocated
region is, to return exactly that much. A shared region's decrement function needs to
know what address the counter is at. A constant's function needs nothing.

Where does that piece come from? We add one more field, provisionally named `data`.
It has to carry *three different kinds of information* depending on the vtable:

- for a constant, `data` is unused;
- for a sole-owned region, `data` holds the length of the allocated region — a
  *number*, not an address;
- for a shared region, `data` holds the *address* of the counter — a real pointer.

A number one moment, a pointer the next. No "proper" Rust type describes that. So we
declare it with the most shapeless type available: **`*mut ()`**. This is a *raw
pointer* to `()` — Rust's "empty" type; in other words it's C's `void*`: exactly **8
bytes** (on a 64-bit machine), carrying no meaning until someone interprets it. Don't
read `data` as "a pointer"; read it as "8 bytes, meaning deferred". For a sole-owned
region we stuff the length number straight into those 8 bytes (an integer wearing a
pointer's clothes); for a shared region those 8 bytes are a genuine address.

`data` and `vtable` always travel as a pair: `data` is 8 intrinsically-meaningless
bytes, and the `vtable` is the *only* thing that knows what those 8 bytes mean this
time. This is why — as you'll see when you code it — every vtable function takes
`data` as its first argument: we hand the 8 meaningless bytes to the one function
that knows how to read them.

## Summary: where the design stands

Put together, after Part 2 our `Bytes` looks like this — for the first time with real
types, no more placeholders:

```rust
struct Bytes {
    ptr:    NonNull<u8>,      // "which bytes": points at the start of the run
    len:    usize,            // "which bytes": how long
    data:   *mut (),          // "who owns it": 8 bytes, meaning defined by the vtable
    vtable: &'static Vtable,  // "who owns it": which clone/drop set to use
}

struct Vtable {
    clone: /* function pointer */,   // to clone this kind, run this
    drop:  /* function pointer */,   // to drop this kind, run this
}
```

And the three possible `data`/`vtable` combinations:

| vtable          | what `data` holds          | what `drop` does |
|-----------------|----------------------------|------------------|
| `STATIC_VTABLE` | (unused)                   | nothing          |
| `OWNED_VTABLE`  | the region's length (num)  | free             |
| `SHARED_VTABLE` | the counter's address (ptr)| decrement        |

Keep this table in mind; in Part 5 it changes by exactly *one* line — the type of the
`data` field — and we'll see why `*mut ()` isn't enough. (The reason lives in `clone`,
and isn't visible yet.)

## What we have, and what Part 3 solves next

We've now solved the first half of Part 1's question: a single `Bytes` type, carrying
three different cleanups, choosing the right one per value at runtime, thanks to a
`vtable` field pointing at one of three prebuilt tables.

But there's the second half, no less important: *reading bytes must stay as cheap as
`Arc<[u8]>`.* In the "two slots" section we glimpsed it: the read operations *aren't*
in the vtable. But "not in the vtable" isn't enough on its own. We also have to lay
out the struct's fields so that reading *absolutely never* touches `data` or
`vtable`, not even a branch. Why that layout beats an `enum`, and why it makes the
two newly-added fields *free* on the hot path, is Part 3.

Part 3 also introduces a way of thinking that will backbone the two hardest remaining
parts: every way of owning memory, reduced to its essence, is one question — *how
many times does this block get freed?*

---

*Next: [Part 3 — Separating "which bytes" from "who owns them"](03_split_and_counting.md)
· [Index](00_index.md)*

*Tiếng Việt: [`../vi/02_vtable.md`](../vi/02_vtable.md)*
