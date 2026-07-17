# Part 1 — A byte's journey from the wire into your program

Let's start with a very ordinary program: a running Ethereum node. It opens a
socket, and on the other end the whole world sends it data — new blocks,
transactions, messages between peers. The node's job, at the lowest level, is to
read those bytes in, make sense of them, and either store them or forward them to
another peer.

Sounds simple, but even at the "read the bytes in" step there's a problem that this
whole series revolves around. We'll go slowly, because everything later grows out of
here.

## Bytes don't arrive neatly

When you read from a socket, the data doesn't arrive as one tidy block of known
length. It comes in chunks — 40 bytes here, 1500 there, depending on the network. A
complete message — say, a block header — might have to be assembled from five or six
of those reads. And you **don't know in advance** how long the message is until
you've read most of it.

So you need somewhere to catch it: a region of memory you keep appending bytes to,
that **grows on its own** when it fills up. In Rust's standard library, the closest
thing is `Vec<u8>` — a growable array of bytes. You push bytes in, and when it runs
out of room it asks for more memory automatically.

In this series we'll call that writable catch-buffer **`BytesMut`** (`Mut` for
*mutable* — changeable). Picture it as roughly a `Vec<u8>`: a pointer to a region of
memory on the heap, plus two numbers — how many bytes are written so far (`len`), and
how much the region currently holds (`cap`, short for *capacity*).

```
BytesMut catching a block header, 7 bytes written, region holds 1024:

   pointer ──────────► [ 68 65 61 64 65 72 21 · · · · · · · · ]
                       └─ 7 bytes written ─┘└─ free space left ─┘
   len = 7
   cap = 1024
```

The key thing about `BytesMut`: it's **writable**, and it **grows**. That's exactly
what you need while you're still reading a message off the socket.

## But once you're done reading, `BytesMut` gets in the way

Say the message is fully read now. What do you want to do with it?

- hand it to a decoder to make sense of the contents,
- put it in a cache to look up later,
- forward it to another peer.

And usually it's **all three at once**: the same block header, the decoder holding a
copy to parse, the cache holding one, the send queue holding one.

Here `BytesMut` becomes a nuisance, for two reasons.

First, it's **writable**. Once the message is complete, we *don't want* anyone
mutating it anymore. If the decoder and the cache both hold one `BytesMut` and one
of them accidentally overwrites it, the other reads corrupted data. Immutable data
is the precondition for safe sharing: if nobody can write, many readers can never
collide.

Second, letting many places hold one `BytesMut` is **expensive**. A `BytesMut` owns
its memory; to give it to three places, the only safe way is to copy it three times.
And copying costs — we'll see how much at the end of this part.

In other words, `BytesMut` is good at *building* a message, but bad at *sharing* one
that's already built.

## `Bytes`: the read-only, shareable handle

So we need a second type, for the later phase: when the message is done and just
needs to be read and passed around. We'll call it **`Bytes`** (no `Mut` — not
changeable).

`Bytes` is a *read-only handle* to some bytes. It has the three properties we need:

- **immutable** — nobody can write through a `Bytes`, so sharing is safe;
- **cheap to clone** — adding one more place that holds it is nearly free, with no
  copy of the contents;
- **self-cleaning** — when the last place holding it lets go, the memory is freed
  automatically.

And the operation that turns a (finished) `BytesMut` into a (shareable) `Bytes` has
a name: **`freeze`**. You stop writing, freeze the buffer, and from then on it's
read-only.

```
   BytesMut  ──freeze──►  Bytes
   (writable,             (read-only,
    still building)        shareable)
```

The whole life of one inbound message, drawn out, is:

```
socket ──► BytesMut ──► freeze ──► Bytes ──► decoder / cache / forward
          (catch &        (freeze     (passed around
           grow)                       everywhere)
```

Now we have enough vocabulary to ask the central question.

## What `freeze` does inside, and why it can be very slow

`freeze` sounds like a harmless operation — just "relabelling" a buffer from
writable to read-only. But it's where the entire performance of the receive path is
decided, for one simple reason: **every inbound message goes through it.** A busy
node calls `freeze` hundreds of thousands, millions of times a second.

The question is: when `freeze` runs, does it have to *copy* the buffer's contents
somewhere new?

The most naive implementation **does**. It allocates a new region of memory, copies
every byte from the `BytesMut` over, and returns a `Bytes` pointing at the new
region. Let's work out what that copy costs.

Say a block-header burst is about 1 MiB. Memory-copy speed (memcpy) on modern
hardware is roughly 5 GiB/s. So copying 1 MiB takes:

```
1 MiB ÷ 5 GiB/s ≈ 200 microseconds
```

200 microseconds sounds small, but that's **200 microseconds of the CPU standing
still**, doing nothing but moving bytes from one place to another — for *one*
`freeze`. Multiply by the messages per second, and your node is burning a
substantial fraction of its time just *re-copying* data it only just read in.

For a system where throughput is everything, a copy per `freeze` isn't
"unoptimized" — it's what removes a design from the running entirely. So we set a
hard requirement, and this whole series is the story of what it costs to hold it:

> `freeze` must run in **constant** time, no matter how big the buffer. The memory
> holding the payload **must not move**; the only thing allowed to transfer is
> *ownership* of it — from `BytesMut` to `Bytes`.

"Don't move the memory, just transfer ownership" is the idea we'll dig at for the
rest of the series. But before building, it's worth asking: why does the most obvious
approach *fail* to achieve it? Because answering that shows where the real problem
lies.

## Attempt #1: `Bytes` wrapping a `Vec<u8>`

First idea: have `freeze` just hand the buffer back as a `Vec<u8>`.

This *does* avoid the copy — `Vec<u8>` and `BytesMut` are both a flat block of heap
memory, so a `Vec` can "adopt" `BytesMut`'s memory without re-copying. So on the copy
front it works. But it breaks in two other places, and those two places shape
everything to come.

First, `Vec<u8>` is **writable**. We're right back to `BytesMut`'s problem: no
guarantee the contents are immutable, so it can't be shared safely.

Second, `Vec<u8>` is **not cheap to clone**. `Vec::clone()` copies the whole
contents. And "let many places hold it" happens constantly. If every "also hold
this" is a memcpy, then we've merely *relocated* the copy from `freeze` to `clone` —
we haven't killed it.

Lesson from attempt #1: we need a type that is simultaneously **immutable**, **cheap
to clone**, and **able to adopt existing memory**. These three properties are the
rubric for scoring every later candidate.

## Attempt #2: `Bytes` wrapping an `Arc<[u8]>`

This is nearly every Rust programmer's second reflex, and it's where plenty of
real-world designs begin.

`Arc` (short for *Atomically Reference-Counted*) is Rust's standard tool for "many
places co-owning one piece of data". Inside, it keeps a **counter** of how many
places currently hold it. Each `clone` increments the counter; each time a copy is
dropped it decrements; when the counter hits zero, the data frees itself.
`Arc<[u8]>` means "an array of bytes, reference-counted".

```rust
struct Bytes(Arc<[u8]>);
```

This gives us **two of the three** required properties right away:

- **immutable** — `Arc<[u8]>` only lends out a read view, nobody can write;
- **cheap to clone** — `clone` just bumps the counter by one, no copy of the
  contents. This is exactly the "many places holding one message" we need.

And it **works**. Plenty of codebases start with exactly this design. The problem
only surfaces at the third property — `freeze` must not copy — and to see why it
breaks you have to look at *how the memory is laid out*.

### Why `Arc<[u8]>` forces a copy at `freeze`

`Arc<[u8]>` is a **single** block of memory, in which the counter sits **fused**
right in front of the bytes:

```
Arc<[u8]>:   [ counter | b0 b1 b2 ... bN ]
             └─ header ─┘└──── payload ────┘
                one single block
```

`BytesMut`'s buffer is payload-only, **no counter** at the front:

```
BytesMut:    [ b0 b1 b2 ... bN ]
             └──── payload ────┘
```

These two layouts are differently shaped, and can never match. To turn `BytesMut`'s
buffer into an `Arc<[u8]>`, you'd have to put the counter *immediately before* the
current pointer. But the memory right before it **doesn't belong to you** — the
allocator never handed it over, and writing there stomps some other part of the
program. There's no way to "retrofit" a header in front of an already-allocated
block.

So when you hand the buffer to `Arc`, it is **forced** to:

1. ask the allocator for a *new* block, big enough for `counter + N bytes`;
2. copy the N payload bytes from the old buffer into the new block;
3. free the old buffer.

Step 2 is the memcpy we swore to kill. And here's the important part: it's **not a
code bug** — it's an inevitable consequence of `Arc<[u8]>`'s *shape*. `Arc<[u8]>`
only knows one kind of ownership — reference counting — and that kind requires the
counter to live *inside* the same block as the payload. A type whose counter is
fused to its payload **cannot adopt** a payload that already sits elsewhere.

Stated as one sentence, here is the hinge of the whole series:

> `Arc<[u8]>` cannot *adopt* an existing region of memory. The only way to get bytes
> into an `Arc<[u8]>` is to allocate a fresh block and copy into it. But what we need
> is the opposite: a handle that points straight into *someone else's* memory (here,
> `BytesMut`'s buffer), and takes on the responsibility of cleaning it up.

## What surfaces when a handle points into "someone else's memory"

The moment we accept the idea of "a handle pointing into an existing buffer", a new
question appears — one that `Arc<[u8]>` never had to answer.

`Arc<[u8]>` always knows exactly what to do when a copy is dropped: decrement the
counter, free at zero. Always, no exceptions — because it only has one kind of
ownership. But a free-pointing handle might be pointing into three kinds of memory
with three opposite fates:

- It points into a **constant baked into the executable** (say, a hard-coded byte
  string in the program). This region was never allocated; when we let go, we must
  do **nothing** — freeing it would be freeing memory we don't own.
- It points into a **buffer just taken from `BytesMut`**. This region *was*
  allocated, and exactly one handle owns it; when we let go, we must free it.
- It points into a **region being shared across several places**. Now we need a
  counter; whoever lets go last is the one who frees it.

This is the central tension of the whole problem, stated for the first time:

> The *same* `Bytes` type, but *three* different ways of cleaning up, and which one
> applies is only knowable at runtime, per individual value.

`Arc<[u8]>` dodges this tension by supporting only *one* cleanup discipline (and pays
for it with the `freeze` memcpy). We don't get to dodge: we need all three *in one
type* — to have a no-copy `freeze` (the "sole ownership" discipline), a cheap `clone`
(the "shared" discipline), and to pay nothing for fixed byte constants (the "static"
discipline, for hard-coded byte strings like genesis constants or precompile
bytecode).

## Why we can't just use three separate types

At this point you might think: "So make three separate types — `StaticBytes`,
`OwnedBytes`, `SharedBytes`, one cleanup each, and let the compiler handle it."

As far as ownership goes, this is actually the *correct* approach — and in Part 2
we'll see that Rust normally *wants* you to do exactly this. It dies for a completely
different reason: the **API boundary**.

Look at a typical function that consumes bytes:

```rust
fn decode(data: Bytes) -> Header;
```

This `decode` — and hundreds of functions like it — has to swallow bytes *regardless
of their source*. With three separate types:

- you'd write `decode` three times (or make every byte-consuming function generic —
  an explosion);
- a `Vec<Bytes>` couldn't hold a mix of the three;
- a channel sending `Bytes` between threads couldn't send a mix;
- a struct with a `Bytes` field would have to hard-pick *one* of the three, losing
  all flexibility.

The entire infrastructure beneath `Bytes` implicitly assumes there is only **one**
type. The "one type" constraint isn't something we impose for fun — it comes from
the very code that *uses* `Bytes`.

And here's the trap: one type means one cleanup function (`Drop`), i.e. one *fixed*
behaviour. And we just proved we need *three* behaviours, chosen per value at
runtime. The tension between "one type, mandated by the API" and "three cleanups,
mandated by no-copy-freeze plus cheap-clone plus free-constants" — that's the real
problem.

## So what Part 2 solves

To recap, we have three approaches and three reasons they die:

- `Vec<u8>`: achieves no-copy freeze, but isn't immutable and clones by copying.
- `Arc<[u8]>`: immutable and cheap to clone, but freeze is forced to memcpy because
  the counter is fused to the payload.
- three separate types: achieves all three cleanups, but the API boundary demands
  one type.

Boiled down to one question, this is what Part 2 must answer:

> How can a single `Bytes` type carry three different cleanup behaviours, picking the
> right one per value at runtime — while reading bytes stays as cheap as `Arc<[u8]>`?

The phrase "one type, many behaviours, chosen at runtime" should sound familiar:
it's precisely the problem *dynamic dispatch* exists to solve. Part 2 shows how to
use it correctly, and a sub-question that matters just as much — why the dispatch
table (the vtable) we're about to build has exactly *two* slots, no more and no
fewer.

---

*Next: [Part 2 — One type, many behaviours](02_vtable.md) · [Index](00_index.md)*

*Tiếng Việt: [`../vi/01_the_problem.md`](../vi/01_the_problem.md)*
