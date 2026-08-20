# Part 1 — What a semaphore is, and when you reach for one

Somewhere in every busy server there's a number nobody chose on purpose: the largest
amount of expensive work it can do at the same time before it falls over. Maybe it's
how many image resizes fit in memory at once, or how many connections the database
will tolerate, or how many in-flight requests a downstream service will accept before
it starts timing out. The server has a ceiling. The trouble begins when nothing in
the code knows the ceiling is there.

Picture an endpoint that does something genuinely expensive — renders a PDF, runs a
heavy query — costing maybe 50 MB and a burst of CPU while it runs. At ten requests
in flight, you're using half a gig; fine. Then you get linked somewhere, or a client
retries in a loop, and ten thousand arrive in a few seconds. Now watch what an async
runtime does with that: it spawns ten thousand tasks and starts all of them. There's
no natural backpressure — nothing in `spawn` says "hold on." The machine doesn't slow
down gracefully. It reaches for 500 GB it doesn't have, and the kernel kills it,
taking the cheap endpoints down with the expensive one.

The fix isn't a faster endpoint. It's a cap: *at most N of these run at once, and the
rest wait their turn.* The thing that enforces that cap is a semaphore, and this
series is about designing one — a real one, fair and cancellation-safe — for async
Rust. We start here, with what it's for, because every design decision in the later
parts is forced by something on this page.

## The doorman

The clearest way to picture a semaphore is a doorman at a club with a strict fire
code. The room holds exactly N people. The doorman has N wristbands. You want in, you
take a wristband; there are none left, you wait outside; someone leaves and hands
their wristband back, the doorman gives it to whoever's waiting. The doorman never
counts heads inside — the wristbands *are* the count. When they're gone, the room is
full, and that's the whole enforcement mechanism.

Translate the story back and you have the definition:

> A semaphore is a counter of *permits* you can wait on. Taking one is `acquire`;
> giving it back is release. When none are left, `acquire` doesn't fail — it waits.

The counter part is trivial; any integer can count. The waiting is the entire
problem, and it's why you can't build this out of an `AtomicUsize` and a subtraction.
An atomic can count down to zero, but it has no idea how to make a caller *wait* at
zero, no idea how to wake that caller when a permit returns, and — as we'll see three
parts from now — no idea what to do when a waiting caller gives up and walks away.
Everything hard about a semaphore is hidden in the word "wait."

One clarification worth making early, because the two get confused: a semaphore is
not a mutex. A mutex answers *who owns this?* — exactly one holder, protecting some
piece of data from being touched by two threads at once. A semaphore answers *how
many at once?* — up to N holders, protecting nothing, rationing capacity. A mutex is
really just a semaphore that happens to have one permit, but if you reach for a
one-permit semaphore to guard a field, you wanted a mutex; if you reach for a mutex
to mean "only four of these at a time," you're about to hand-roll a semaphore and get
it subtly wrong.

## The same shape, four times

What makes a semaphore worth a whole series isn't the doorman story — it's that four
problems that look unrelated turn out to be the doorman story wearing different
clothes. And each one quietly demands something specific from the design, which is
how we'll end up with an interface in Part 2 that wasn't guessed at.

The first is the one we opened with: **bounded concurrency**, capping how many heavy
operations run together. In practice you often run two semaphores side by side — a
generous cap on ordinary work and a tighter one on the heaviest kind, because a trace
or a big export costs more memory than a normal request and you want a smaller ceiling
for it. So already the design has to be cheap enough that having several instances is
nothing to think about.

The second is **load shedding**. Under a burst you don't always want to wait for a
permit — sometimes the right answer is to give up instantly and return "too busy" so
the caller can retry elsewhere. That's not the blocking `acquire` at all; it's a
different operation, "give me a permit only if one is free *right now*," and the
design has to offer it.

The third is a **connection pool**, and it teaches the sharpest lesson. Here the
permit doesn't merely represent a connection — it may as well *be* the connection: you
hold it exactly as long as you hold the connection, and when you're done it goes back.
But "when you're done" includes the ugly paths — an error, a panic, a future that gets
dropped halfway. If returning the permit is something the programmer has to remember
to do, then every early return you ever write is a leaked connection waiting to
happen, and the pool bleeds to empty. The permit has to return *itself*.

The fourth is the one people don't recognize until it's pointed out: a **bounded
channel** — a queue that makes the producer wait when it's full — is a semaphore on
the inside. Capacity N is N permits; `send` is `acquire`; a consumer taking an item is
release. Every backpressured pipeline you've ever built has a semaphore hiding in it.
And because `send` sits on the hot path of everything, this use case demands that
`acquire` be `async` and genuinely cheap.

Line those four up and the interface almost writes itself: a counter you wait on, a
non-blocking way to try, a permit that returns itself, and an `acquire` that's async
and light.

## The two things the use cases only whisper

There are two more requirements, and they're the reason this series has seven parts
instead of two, because a quiet test never triggers either one but production triggers
both constantly.

The first is **fairness**. Go back to the doorman with a full room and a crowd waiting
outside, and imagine he's a little careless: each time a wristband comes back, instead
of giving it to whoever's been waiting longest, he hands it to whoever happens to be
standing closest — which, at a busy door, is always some newcomer who just walked up.
One unlucky person can stand there all night while people who arrived after them keep
getting waved in. A semaphore can do exactly this, and under steady load one waiter
can be overtaken indefinitely. Nothing crashes. Your p50 latency looks great. Your
p99.9 is a slow-motion disaster, because it's measuring the people stuck at the door.
For anything with a latency budget, "some callers wait an unbounded amount of time" is
an outage, not a statistic — so fairness has to be designed in, and Part 4 is about
what it costs.

The second is **cancellation**, and it's the one with no equivalent in ordinary
threaded code. A thread that's waiting will, eventually, stop waiting and run its next
line — the OS promises it. An async task makes no such promise: a task that's waiting
can simply cease to exist, its future dropped mid-wait, because a `timeout` fired or a
`select!` picked another branch. Most of the time that's exactly what you want. But
for the semaphore holding that waiter's place in line — possibly with a permit already
set aside for it — a waiter that can vanish into thin air is the source of the hardest
bug in the whole thing. Part 5 is where we corner it.

## What we're holding

Before writing a single line of implementation, the use cases have handed us the
complete brief:

```
count a budget                    new(n)
wait for a free permit            acquire().await
return a permit automatically     an RAII permit value
refuse to wait                    try_acquire()
shut down without hanging         close()
never starve a waiter             fairness           (Part 4)
survive a waiter vanishing        cancellation-safe  (Part 5)
```

In code, the thing we're aiming to be able to write:

```rust
static PDF_JOBS: Semaphore = Semaphore::new(10);

async fn render_endpoint(req: Request) -> Response {
    let _permit = PDF_JOBS.acquire().await?;  // waits if 10 are already running
    render_pdf(req).await                     // permit held while we work
}                                             // permit dropped here → next waiter wakes
```

That's the design, stated as needs rather than code. In Part 2 we turn each line into
an actual Rust signature — and discover, pleasantly, that a couple of these
requirements quietly disagree with each other, and the interface has to decide who
wins.

---

*Next: [Part 2 — The interface, read off the use cases](02_the_interface.md) · [Index](00_index.md)*

*Deutsch: [`../de/01_what_is_a_semaphore.md`](../de/01_what_is_a_semaphore.md)*
