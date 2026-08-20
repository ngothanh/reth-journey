# Designing an async semaphore

Somewhere inside every busy async program there's a cap — at most N database
connections, at most N heavy requests, at most N messages in flight — and the thing
enforcing that cap is a semaphore. If you've used `tokio::sync::Semaphore`, you've
held one. This series designs one from scratch.

It's a design investigation, not a coding tutorial. We start from the problems a
semaphore exists to solve, read the interface off the use cases, and then work
through the questions anyone building one must answer: where does waiting
physically live? who gets a freed permit? what happens when a waiter is cancelled
mid-wait? where do the waiters' records live in memory? Each part ends on the
question the next one answers, and no decision falls from the sky — every one is
forced by a use case or by the failure of the simpler alternative. Implementation
is the last part, deliberately: once the design is settled, the code follows from
it.

No async-internals background is assumed. The machinery — `futex`, `Waker`,
`Pin` — is introduced at the moment the design collides with it.

## The parts

**[Part 1 — What a semaphore is, and when you reach for one.](01_what_is_a_semaphore.md)**
Every server has a ceiling, and trouble starts when nothing in the code knows it's
there. A doorman with N wristbands; why a semaphore is not a mutex; four problems
that turn out to be the same problem — and the two requirements they only whisper:
fairness and cancellation.

**[Part 2 — The interface, read off the use cases.](02_the_interface.md)**
The two-method sketch everyone would write, and how the use cases dismantle it: the
permit that returns itself, the difference between returning and minting, two error
types that refuse to lie, shutdown as part of the interface — and the quiet decision
(a *named* future) that won't pay off until Part 5.

**[Part 3 — Where does the waiting live?](03_where_waiting_lives.md)**
"Wait" is a physical arrangement, and the kernel — which parks threads beautifully —
has never heard of your tasks. The contract that replaces the futex: polled once,
then silence until the waker fires. What that forces: the semaphore remembers its
own waiters, release must wake, and three rules that everyone eventually learns the
hard way. The surprise at the end: the userspace design is *cheaper*.

**[Part 4 — Fairness: who gets the freed permit?](04_fairness.md)**
A permit comes back and three tasks want it. Let them race, and under load one
unlucky waiter can lose every race indefinitely — invisible to tests, vivid at
p99.9. Hand the permit to the front of the line instead, and one invariant carries
the whole design: *if anyone is queued, the counter is zero.* Both answers are
correct; the fork is real; we choose with eyes open.

**[Part 5 — Cancellation: when a waiter vanishes.](05_cancellation.md)**
A thread that starts waiting will stop waiting; a future can just cease to exist.
Dropped while waiting, a waiter must take its entry with it. Dropped in the gap
between *woken* and *run* — holding a permit it will never consume — it must give
that permit back, or capacity bleeds away silently. One sentence covers every case:
on drop, a waiter returns whatever it owns at that moment.

**[Part 6 — Where the waiters live, and what Pin is for.](06_memory_and_pin.md)**
The waiter's record has to live at some address, and the choice decides whether
`acquire` allocates. The fast answer sounds illegal: inside the future itself, with
the queue threading pointers through memory it doesn't own. It's sound under one
guarantee — a linked future never moves again — and that guarantee is exactly what
`Pin` exists to enforce, not with a runtime guard but by withholding `&mut`.

**[Part 7 — Writing it down, and trusting it.](07_implementation.md)**
Six parts of rules collapse into a four-state lifecycle stored in each waiter's
record; every function becomes a `match`, and the two worst bugs become an arrow
that exists and an arrow that doesn't. Then the trust question: hand-driven tests
for every transition, Miri for the pointer promises the compiler took on faith —
and why loom has nothing to find here *yet*.

## How to read it

In order — each part opens where the previous one stopped and closes on the
question the next one answers. Ten to fifteen minutes each. Parts 1–2 are the
outside; Parts 3–6 are the four design questions; Part 7 is the write-up. Stopping
after Part 2 already changes how you read `tokio::sync::Semaphore`'s docs; the
design meat starts at Part 3.

## Scope

This series designs the ideas inside `tokio::sync::Semaphore` — not a drop-in
replacement. Everything stays behind one `Mutex` throughout: the lock-free fast
path that production implementations add is flagged where it would go and
deliberately not taken, because it changes none of the design questions and
obscures several. Where tokio's `batch_semaphore` diverges (batched wakeups, the
atomic fast path, `acquire_many`), Part 7 points at it.

## Glossary

- **permit** — the unit a semaphore hands out; N permits means N concurrent holders.
- **futex** — the Linux syscall a *thread* uses to sleep until another thread
  signals an address; the kernel's parking primitive.
- **task / future** — a userspace unit of async work; the runtime polls it, the
  kernel doesn't know it exists.
- **`poll` / `Pending`** — the runtime asks "done?"; `Pending` means "not yet —
  and don't ask again until my waker fires."
- **`Waker`** — the handle a future stores so someone can later say "poll me
  again"; the userspace replacement for a futex wake.
- **barging** — freed permits return to the shared counter and anyone may grab
  them; woken waiters re-race newcomers.
- **hand-off** — freed permits are assigned to a specific queued waiter; there's
  nothing to race for.
- **cancellation** — a future being dropped before completion; in async this is
  routine (`timeout`, `select!`), not an error path.
- **intrusive list** — a linked list whose links live *inside* the elements; no
  allocation per node.
- **`Pin`** — a reference type promising its target never moves again, enforced by
  withholding `&mut`; the precondition for pointing into a future.
- **Miri** — an interpreter that catches invalid pointer use (UB) that tests
  execute but cannot detect.
- **loom** — a tool that re-runs a concurrent test under every possible thread
  interleaving; the verifier for lock-free code.

*Deutsch: [`../de/00_index.md`](../de/00_index.md)*
