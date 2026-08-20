# Part 5 — Cancellation: when a waiter vanishes

A thread that starts waiting will, sooner or later, stop waiting. It might block
for a microsecond or an hour, but the operating system promises it will eventually
wake and run its next line. Threaded code leans on that promise everywhere without
naming it: the cleanup, the bookkeeping, the taking of the permit — all of it lives
"after the wait," and after the wait always arrives.

An async task makes no such promise. A waiting task can simply cease to exist — its
future dropped mid-wait, never polled again. And this isn't an exotic failure; it's
a feature in daily use:

```rust
match timeout(Duration::from_millis(100), sem.acquire()).await {
    Ok(permit) => handle(permit).await,
    Err(_)     => return Response::too_busy(),  // acquire's future: dropped, mid-wait
}
```

A `timeout` that gives up, a `select!` that abandons its losing branch, an aborted
handler — each one destroys a future that was parked in our queue, holding a place
in line. The question that organizes this whole part: when a waiter vanishes, *what
did it owe, and to whom?* The answer depends on when it vanished.

## Vanishing while waiting

The milder case: dropped while queued, not yet granted. The waiter held nothing —
but it *left something behind*: its record in the queue.

That record can't be abandoned, because the release side trusts the queue:

```
queue:  [ A ] → [ B✝ ] → [ C ]        B's future was dropped; its record remains

next release:  pop_front… reaches B✝ → granted = true, wake(B's waker)
               → wakes a task that no longer exists
               → the permit sits in a record nobody will ever read
               → capacity −1, silently
```

A handler timing out in a retry loop can bleed a pool dry in minutes this way,
without one line of logging. So the first rule writes itself — and Part 2's named
future is what makes it writable, because a compiler-generated `async fn` future
has no place to put it:

```rust
impl Drop for Acquire<'_> {
    fn drop(&mut self) {
        let mut state = self.semaphore.state.lock().unwrap();
        // I was still waiting → take my record out of the queue with me
        state.queue.remove(my_record);
    }
}
```

Pocket one detail for Part 6: the record being removed sits somewhere in the
*middle* of the queue. Cancellation never politely asks the front.

## Vanishing while granted

The severe case exists because of the gap Part 4 kept circling: waking a task does
not run it. Between "your waker fired" and "a worker polls you" there is a window,
and cancellation can strike inside it:

```
t₀   release:  pop A's record, granted = true, wake(A)

t₁   A is runnable — but no worker has picked it up yet        ← the window

t₂   A's timeout expires first.  A's future is dropped.
```

Now do the accounting, slowly, because this is the entire bug. The permit was
placed in A's record, so it never touched the counter. A never ran again, so it was
never consumed. It isn't in the counter; it isn't held by any live task. It is
*nowhere* — and it is never coming back.

A `Semaphore::new(4)` that suffers this once is a `Semaphore::new(3)` wearing the
old name. Four unlucky timeouts later, the semaphore admits no one, forever, having
never returned an error or logged a word. In production this surfaces weeks later
as "the service mysteriously stops accepting traffic under load."

The rule, once the shape is visible, is almost self-evident: a waiter dropped while
granted must put its permit back through the normal release path. `Drop` grows a
second branch:

```rust
impl Drop for Acquire<'_> {
    fn drop(&mut self) {
        let mut state = self.semaphore.state.lock().unwrap();
        if my_record.granted {
            // a permit is sitting in my name, unconsumed → pass it on:
            // to the next queued waiter, or the counter if nobody waits
            release_one(&mut state, &mut to_wake);
        } else {
            // still waiting → take my record with me
            state.queue.remove(my_record);
        }
        // (wakers collected above are invoked after the lock drops — Part 3's rule)
    }
}
```

Note that the granted branch calls the *same* `release_one` as `add_permits` — and
that release can grant the permit to a second waiter B who is *also* sitting in the
wake-to-run window, who is *also* dropped, whose `Drop` releases again, to C. The
permit ripples down the queue until it reaches someone who actually runs, or the
queue empties and it settles into the counter. Each hop is the same move — the
dying holder returns what it holds — so the chain needs no special handling.

## The bug on the other side of the coin

The two `Drop` branches make leaks impossible. They also create a trap on the
success path, and it's two lines big. Here is `poll` consuming a granted permit —
first the broken version:

```rust
// BROKEN consume:
if my_record.granted {
    return Poll::Ready(Ok(permit));
    //  the future will STILL be dropped eventually — every future is.
    //  Drop will run, see granted == true… and release the permit AGAIN.
}
```

The permit gets consumed once and returned twice. Now the semaphore isn't losing
capacity — it's *minting* it: `new(4)` drifts toward `new(5)`. The fix is to record
that the debt is settled:

```rust
// CORRECT consume:
if my_record.granted {
    my_record.granted = false;              // ← the one line between leak and mint
    detach_from_queue_bookkeeping();
    return Poll::Ready(Ok(permit));
}
```

Leak and mint are the same accounting error with opposite signs, and both come down
to one question asked at drop time: *what does this waiter own right now?*

## One rule instead of three

Set everything side by side and the pattern closes. At any instant a waiter owns
exactly one thing — first a place in line, then a permit, then nothing:

```
 (created) ──enqueue──►  WAITING  ──grant──►  GRANTED  ──consume──►  DONE
                            │                    │
                       dropped here         dropped here
                            │                    │
                            ▼                    ▼
                     return my SLOT       return my PERMIT
```

> **On drop, a waiter returns whatever it owns at that moment.** The three rules
> are one sentence, read at three points on a timeline.

The `granted: bool` from Part 4 is starting to creak — it's really a three-state
value (waiting / granted / done) squeezed into a boolean plus context. Part 7 will
promote it into an honest enum and make every function a `match` over it. But
first, the design owes one final answer.

## The remaining question is physical

Throughout this part, "my_record" has been doing quiet work: `Drop` reaches *its
own* record in the queue; release reaches *the front* record; both mutate it. Every
waiter needs such a record — waker, granted state, a place in line — and that
record has to live at some actual address in memory. Inside the semaphore? Inside
the future? The choice decides whether `acquire` allocates, and the fast answer
sounds, on first hearing, like it shouldn't be legal at all. Part 6.

---

*Next: [Part 6 — Where the waiters live, and what Pin is for](06_memory_and_pin.md) · [Index](00_index.md)*

*Deutsch: [`../de/05_cancellation.md`](../de/05_cancellation.md)*
