# Part 4 — Reading without UB, and trusting it

Part 3 made the protocol correct on the hardware. But look back at what the reader has
been doing the entire time, on purpose: reading the payload while the writer is
actively overwriting it. We *designed* that in — the whole bet was to let the read tear
and catch it afterward. The trouble is that in Rust's memory model, that read isn't
merely "reading garbage." It's a data race, and a data race is undefined behaviour.

## The read the language forbids

Here's the natural way to copy the payload out — a plain read through a raw pointer:

![Copying the payload with a non-atomic read races the writer](../img/cards/naive_read.png)

Two threads touch the same bytes, one of them writing, neither access atomic. That's
the textbook definition of a data race, and in Rust (as in C++) a data race is not
"you get an unspecified value." It's UB: the compiler is entitled to assume it never
happens and optimise on that assumption — hoist the read out of the retry loop, prove
the value unchanged, delete branches it "knows" are dead. Your careful `s1 == s2` check
can be optimised away by a compiler that has *proven*, under the no-data-race
assumption, that it's always true. The bug isn't the garbage; it's that the compiler is
now reasoning from a false premise.

Miri — an interpreter that runs your code against the memory model — says so directly:

![Miri reports the payload read as undefined behaviour](../img/cards/term_miri.png)

## `volatile` is not the fix

If you come from C, the reflex is `volatile`. That's how the Linux kernel's seqlock
reads its payload, and it works there. But `volatile` answers a different question. It
tells the *compiler* not to elide, duplicate, or reorder this particular access — it
does **not** make the access atomic, and in the language memory model a `volatile` read
racing a write is still a data race, still UB. It works in the kernel because the kernel
is compiled by one known compiler with known flags; it's a deal struck with a specific
implementation, not with the language. (Hans Boehm wrote a whole paper on exactly this
mismatch: seqlocks and language memory models don't get along unless the language gives
you a cheap enough atomic.)

Rust gives you one: a `Relaxed` atomic access.

## Atomic, one word at a time

The fix is to make every payload access atomic. Hardware has no 40-byte atomic, but it
has an 8-byte one, so we walk the value one `usize`-word at a time, each word a
`Relaxed` atomic load or store:

![Reading the payload word by word through Relaxed atomics](../img/cards/atomic_words.png)

`Relaxed` is the key, and it's widely misunderstood. It adds **no ordering** — that was
Part 3's job, done by the fences. What it adds is legality: an atomic access racing
another atomic access is *not* a data race, so it's not UB. The words can still tear
against each other — word 0 from the new value, word 3 from the old — and that's fine,
because the sequence counter catches exactly that. `Relaxed` doesn't stop the tearing;
it makes the tearing *legal*, so the counter is allowed to do its job instead of the
compiler miscompiling around a race.

## The bound this forces: `Pod`, and why it's two gates

To reinterpret an arbitrary `T` as a row of `usize` words, `T` has to actually *be*
plain bytes — no padding, every bit pattern valid (the reader will observe half-written
mixes before rejecting them), a defined layout. That's the `Pod` trait:

![Pod is one gate; the size and alignment asserts are a second, independent gate](../img/cards/pod_bound.png)

Two things about that bound are worth stating out loud. First, `Pod` is a *license the
implementer signs*, not a fact the compiler verifies — `unsafe impl Pod for Foo {}` is
a promise you make and take responsibility for; get it wrong and it's UB, which is why
the trait is `unsafe` to implement. It doesn't make correctness automatic; it *localises*
the proof obligation to one greppable line and makes accidental misuse (a `String`, a
type with padding) fail to compile.

Second — and this is the trap — `Pod` is necessary but **not sufficient**. It says
nothing about size or alignment. `u8` is a perfectly honest `Pod`, and `SeqLock<u8>`
still breaks: one byte isn't a whole `usize` word, and the payload might not be
word-aligned for the atomic load. So the size-multiple and alignment checks are a
*second, independent gate* the type must pass, enforced separately (a `const` assert
that fails at compile time, not run time). Two gates, covering two different things.

## One writer was a convenient lie

The protocol so far assumed a single writer. Real code has several — and if two threads
call `store` at once, they both bump the counter and interleave their payload writes,
and a reader can accept the mess. Nothing crashes (every access is atomic now, so it's
not UB — just wrong), but it's wrong.

The fix reuses the machinery already there. The sequence counter takes a second job: it
becomes the writers' lock. Bumping even → odd isn't a blind increment any more, it's a
compare-and-swap that only succeeds *from an even value*. Odd already means "a write is
in progress"; now it also means "the write slot is taken." A second writer sees odd and
spins until the first releases it back to even.

![The seq counter doubles as the writers' lock: CAS only from an even value](../img/cards/writer_cas.png)

One integer, two meanings, no extra state: to the reader, odd says "don't read"; to
another writer, odd says "wait your turn." Writers serialise; readers stay lock-free and
oblivious.

## Trusting it — because a green test proves nothing here

We have already watched this code pass four times out of five while being wrong. In
lock-free code, a green test is close to meaningless on its own; correctness is a
property of *every* interleaving, and a test exercises a random few. Three tools do the
real work.

**A test designed to catch the specific bug.** Have the writer only ever publish
`[n, n, n, n]` — four identical words. Then any load whose words differ is, by
construction, a torn read, and the assertion names it:

![The torn-read detector: four identical words in, so any mismatch is a tear](../img/cards/torn_test.png)

Run it with one writer and a few readers spinning, and a broken ordering shows itself in
milliseconds. This is a test whose job is to *fail* on the bug — the opposite of a test
that confirms the happy path.

**Miri**, for the undefined behaviour a normal test can execute without detecting — the
data race we spent this part removing. It runs the interleavings it sees against the
memory model and reports UB directly, which is how we knew the non-atomic read was
illegal even on the runs where it happened to produce the right bytes.

**loom**, for the orderings. It re-runs a small scenario — one writer, one reader; then
two writers — under *every* thread interleaving the memory model permits, and checks the
invariants hold in all of them. Where the torn-read test samples a few schedules, loom
is exhaustive over a bounded model; it's the closest thing to a proof that the fences
are placed correctly.

## The payoff, in nanoseconds

All of this — the tearing, the fences, the atomics, the `Pod` bound — buys one thing:
a read path that stays flat as readers pile on, where a `RwLock` degrades. Measured on
an Apple M2, a 32-byte payload, reader latency as the reader count grows:

![Read latency vs reader count: SeqLock stays flat while RwLock climbs to 450× at eight readers](../img/en/chart_scaling.png)

At one reader the SeqLock read is already ~7× cheaper (no reader-counter RMW). The real
story is the shape: add readers and SeqLock stays flat — 0.75 ns at one, ~1.5 ns at
eight — while `RwLock` climbs almost linearly to 680 ns, because every reader keeps
writing that shared counter and bouncing its cache line. At eight readers it's **450×**.
That gap is Part 1's MESI diagram, paid out in nanoseconds.

## What it cost

None of this is free, and the price is exactly the set of constraints that made the read
path free: the payload must be `Pod`, a word-multiple in size and word-aligned; the
reader gets a *copy*, never a `&T` to borrow; and under a hot writer the reader retries
rather than blocking. Accept those, and you get the thing you build once and reuse
everywhere a small value is read far more than it's written — the chain head, the mark
price, the top of an order book. Reject them, and you reach for `RwLock` and pay the
450× the first time eight cores read at once.

That's the SeqLock: it trades *the reader never waits and never writes shared memory*
for *the reader may retry, and the writer always wins*. On the read-mostly problems it's
built for, that's the trade you want.

---

*[Index](00_index.md)*

*Deutsch: [`../de/04_trusting_it.md`](../de/04_trusting_it.md)*
