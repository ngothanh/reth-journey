# Part 4 — Fairness: who gets the freed permit?

A permit comes back, and three tasks are waiting for it. Someone has to decide who
proceeds — and every semaphore design decides, including the designs that never
noticed they were deciding. Part 3's code already decided without asking. This part
makes the decision visible, shows what it costs, and replaces it.

## The first answer: let them race

Here's the release path we wrote in Part 3, with its decision underlined:

```rust
state.permits += n;                      // permit goes into the SHARED counter
match state.waiters.pop_front() {
    Some(waker) => to_wake.push(waker),  // wake someone — but promise them nothing
    None => break,
}
```

The woken waiter re-runs `poll` like anyone else: check the counter, take a permit
if one's still there. This design is called **barging**, and its virtues deserve a
fair hearing. The state is minimal — a counter and bare wakers. And it's forgiving:
because the counter is the single source of truth, *sloppy waking is harmless*.
Wake too many waiters and the extras re-check, find nothing, and re-enqueue —
wasteful, never wrong. For workloads with no latency obligations, this is a
defensible place to stop.

## What racing costs

The trouble hides in a gap Part 3 already exposed: *waking a task does not run it.*
`wake()` marks the task runnable; a worker picks it up later. Between those two
moments the permit sits in the counter — visible to everyone. Watch a steady stream
of newcomers work that window:

```
counter = 0.  A is parked in the queue.

release  →  counter = 1, wake A
                 ▲
                 │    A is runnable… but not yet running
newcomer B:      │    poll sees counter == 1  →  takes it.  counter = 0
A finally runs:  └──  poll sees counter == 0  →  A re-enqueues, parks again

release  →  wake A …  newcomer C takes it first  …  A parks again
release  →  wake A …  newcomer D takes it first  …  A parks again
```

The gap is structural — the runtime *has* to schedule A, and a newcomer already on
a CPU beats a waiter that isn't, every time. Under sustained arrivals A can lose
this race *indefinitely*. The waiters barge past the queue — hence the name — and A
is starved by the design itself.

Two properties make this dangerous. Tests can't see it: every operation is
individually correct, and A's starvation is a statistical property of contention —
"does a waiter eventually get a permit?" passes sweetly in any quiet environment.
And production sees it only where people look last: nothing crashes, p50 looks
superb, and p99.9 is a horror, because p99.9 is where the unlucky waiters live. For
the pool and latency-budget use cases of Part 1, "some callers wait unboundedly
under load" is an outage with good manners.

## The second answer: hand it off

The fix inverts one decision: when anyone is waiting, a released permit **never
touches the shared counter**. It goes directly into the front waiter's hands.

For that to be expressible, a queue entry has to grow: a bare `Waker` can't hold a
permit. Each waiter gets a small *record*, shared between the queue and that
waiter's future — the queue reaches it to grant, the future reads it to learn:

```rust
struct Waiter {
    waker: Waker,
    granted: bool,     // ← "a permit is sitting in your name"
}

struct State {
    permits: usize,
    queue: /* Waiter records, in arrival order — physical home decided in Part 6 */,
}
```

Release changes from "bump the counter" to "assign, or bump if nobody's there":

```rust
// hand-off release: one permit
match state.queue.pop_front() {
    Some(waiter) => {
        waiter.granted = true;                  // permit goes INTO the record…
        to_wake.push(waiter.waker.clone());     // …and its owner is woken
    }
    None => state.permits += 1,                 // only spills to the counter
}                                               //   when the queue is empty
```

And `poll`, for a waiter that's already queued, changes in a way that's easy to
miss and essential:

```rust
// re-poll of a queued waiter:
if my_record.granted {
    return Poll::Ready(Ok(permit));    // consume what's already mine
}
if !my_record.waker.will_wake(cx.waker()) {
    my_record.waker = cx.waker().clone();
}
Poll::Pending
// note what is ABSENT: no `state.permits > 0` check. A queued waiter
// never reads the counter — its permit arrives via its record or not at all.
```

Now the wake-to-run gap doesn't matter. However long A takes to get scheduled, its
permit waits in its record; there's nothing in the shared counter for a newcomer to
steal. Waiting time is bounded by queue position, full stop.

The whole design compresses into one invariant, worth stating because the remaining
parts lean on it:

> **If anyone is queued, the counter is zero.**

It holds automatically — queued waiters intercept every released permit before the
counter sees it — and it quietly does the enforcement. A newcomer's `poll` finds
`permits == 0` and has no choice but to join the back of the line. Even
`try_acquire` becomes honest for free: with waiters present there's nothing to
grab, so it can't cut the line either.

## Why the absent check matters

That missing `state.permits > 0` in the re-poll deserves its own paragraph, because
putting it back is the natural mistake. When could a queued waiter even see a
nonzero counter? When `add_permits` mints more permits than there are waiters — the
surplus spills into the counter while granted waiters are still waking up. A queued
waiter that helps itself from the counter *and* later finds `granted == true` in
its record has taken two permits for one release. The semaphore just minted
capacity out of thin air, and no test that doesn't specifically stage this
interleaving will ever notice.

One rule, then: a waiter that has joined the queue takes *only* from its record.

## Choosing, with eyes open

| | barging | hand-off |
|---|---|---|
| state | counter + bare wakers | counter + per-waiter records, in order |
| waking | sloppy is safe | must wake the specific granted waiter |
| worst-case wait | unbounded under load | bounded by queue position |
| cancellation | trivial — waiters own nothing | hard — a granted permit can be orphaned |
| suits | best-effort throttling | pools, budgets, anything with a latency SLA |

Both columns are correct semaphores; the fork is real. This series takes hand-off,
for the same reason tokio does — the use cases that shaped our interface are the
ones that can't absorb unbounded waits. The mistake worth warning against isn't
choosing barging; it's choosing it *by accident*, which is what happens whenever
someone improvises a semaphore from a counter and a condvar and never asks who gets
the freed permit. The p99.9 answers eventually.

## The row in the table with teeth

One cell above deserves a second look: under hand-off, cancellation got *hard*. A
barging waiter owns nothing — if it vanishes mid-wait, nothing is lost. A hand-off
waiter can vanish *while a permit sits in its record* — `granted = true`, woken,
and dropped by a `timeout` one instant before a worker would have polled it.

What happens to that permit? Nothing crashes. Nothing logs. That's the problem —
and it's Part 5.

---

*Next: [Part 5 — Cancellation: when a waiter vanishes](05_cancellation.md) · [Index](00_index.md)*

*Deutsch: [`../de/04_fairness.md`](../de/04_fairness.md)*
