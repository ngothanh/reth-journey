# Part 3 — Where does the waiting live?

When a task asks for a permit and none is free, it waits. That sentence slides by
easily, but "wait" is doing enormous work in it. Waiting is a physical arrangement:
somewhere, some piece of machinery has to remember that this particular piece of
work is asleep, keep it from burning CPU while it sleeps, and know how to rouse it
when its turn comes. Every synchronization primitive is, underneath, an answer to
the question of who keeps that memory. The async answer is strange enough that it's
worth looking at the ordinary one first.

## How a thread waits

In a threaded program, the machinery belongs to the operating system. A thread that
must wait tells the kernel so — on Linux, through a syscall called `futex` — and the
kernel takes it off the CPU:

```rust
// a thread-based acquire, in essence:
fn acquire(&self) {
    loop {
        if try_take_permit() { return; }
        futex_wait(&self.counter);   // syscall: kernel parks THIS THREAD
    }                                // woken by futex_wake from a releaser
}
```

The thread costs nothing while parked. When a permit is released, the other side
calls `futex_wake`, the kernel picks a sleeper, and the woken thread continues from
the exact line where it stopped. The hard parts — remembering who's asleep, choosing
whom to wake — are the kernel's problem.

This works for a reason that's easy to miss: the thing that waits and the thing the
kernel schedules are the *same object*. A thread is a kernel resource. Of course the
kernel can park one.

Now break that assumption.

## The kernel has never heard of your tasks

An async runtime runs, say, eight worker threads and fifty thousand tasks. A task is
not a thread — it's a userspace value, a paused state machine that a worker polls
for a moment and sets aside. The kernel schedules the eight workers. The fifty
thousand tasks are invisible to it.

So suppose `acquire`, deep inside some task, calls `futex_wait` anyway. The kernel
can't park a *task*; it parks what it knows — the worker thread the task happened to
be running on. One waiting task just confiscated an entire worker, and the six
thousand other tasks assigned to it are stranded behind a wait that has nothing to
do with them. Eight waiting tasks on an eight-worker runtime freeze the program
solid: every worker parked, a mountain of runnable work, nobody left to run it.

Even without the freeze, two things are broken. The runtime called `poll` as an
ordinary function expecting an answer — ready or not-yet — and a `poll` that parks
its thread doesn't answer; it just never returns. And when the kernel eventually
wakes the parked *thread*, which *task* was supposed to resume? Nobody wrote that
down anywhere the runtime can see.

The conclusion is structural: **waiting has to move up a layer.** The kernel
remembers waiting threads; something in userspace must remember waiting tasks. For
a semaphore, that something is the semaphore itself.

## The contract that makes it possible

Rust's async model gives the semaphore two tools — `Poll::Pending` and the `Waker` —
wrapped in one rule that beginners routinely assume can't be true:

> After a future returns `Pending`, it will not be polled again — at all — until
> its `Waker` is invoked.

There is no periodic re-check, no background sweep, no timeout of last resort. A
`Pending` future whose waker nobody calls sleeps forever, and that's not a defect —
it's the same discipline as the futex, one layer up. *Sleep silently; be woken
explicitly.*

For the semaphore, the contract dictates both halves of the design, and now we can
write them down. The state is a counter plus a queue of wakers, guarded together:

```rust
pub struct Semaphore {
    state: Mutex<State>,
}

struct State {
    permits: usize,
    waiters: VecDeque<Waker>,     // ← the waiting physically lives here
}
```

The acquire side is the named future from Part 2, and its `poll` does the check and,
if needed, the enrollment:

```rust
impl Future for Acquire<'_> {
    type Output = Result<SemaphorePermit<'a>, AcquireError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.semaphore.state.lock().unwrap();

        if state.permits > 0 {
            state.permits -= 1;
            return Poll::Ready(Ok(SemaphorePermit { semaphore: self.semaphore }));
        }
        state.waiters.push_back(cx.waker().clone());   // "wait" = store the waker
        Poll::Pending
    }
}
```

And the release side must wake — adding a permit to a counter nobody will look at
again helps no one:

```rust
pub fn add_permits(&self, n: usize) {
    let mut state = self.state.lock().unwrap();
    state.permits += n;
    for _ in 0..n {
        match state.waiters.pop_front() {
            Some(waker) => waker.wake(),   // ← without this line: deadlock
            None => break,
        }
    }
}
```

That's a working async semaphore in about thirty lines. It also contains two bugs
and one deadlock-in-waiting, all three invisible in a quiet test. Fixing them is a
matter of three rules.

## Three rules, learned the hard way by everyone

**Check and enroll under one lock.** Look at what `poll` above gets right: the
permit check and the `push_back` happen under a single `lock()`. Here's a version
that looks almost identical and is broken — the difference is one `let`:

```rust
// BROKEN — the check and the enrollment run under DIFFERENT locks:

let permits = self.semaphore.state.lock().unwrap().permits;
//            └──────────────┬──────────────────┘
//            this returns a MutexGuard — the lock is held only while
//            the guard lives. But the guard is never stored in a variable,
//            so it's a TEMPORARY: Rust drops it at the end of this
//            statement, at the `;` — and dropping a MutexGuard UNLOCKS.
//            Net effect of this line: lock, copy `permits`, unlock.

if permits == 0 {
    //  ← the lock is NOT held here. A release can land in this gap:
    //    it sees an empty queue, wakes nobody, and its permit sits
    //    in the counter — where we already looked, and won't look again.

    self.semaphore.state.lock().unwrap()          // a brand-new lock acquisition
        .waiters.push_back(cx.waker().clone());   // enroll — too late
    return Poll::Pending;                          // sleep forever
}
```

This is the classic *lost wakeup*, and in Rust it often enters through exactly this
door: a `MutexGuard` that was never bound to a variable dies at the semicolon, and
the lock silently reopens between two statements that were meant to be atomic. The
correct version differs by one binding:

```rust
let mut state = self.semaphore.state.lock().unwrap();
//  └── the guard has a NAME now, so it lives to the end of the scope —
//      everything below happens inside one continuous critical section
if state.permits > 0 { … }
state.waiters.push_back(cx.waker().clone());
Poll::Pending
// `state` drops here → unlock, after both the check and the enrollment
```

One lock acquisition covers the look and the enrollment, so no release can land
between them.

**Refresh the waker on every poll.** The waker the runtime hands you is not
guaranteed to be the same object from poll to poll — combinators like `select!`
wrap and replace wakers; tasks migrate between workers. The contract's fine print:
the waker that counts is the one from the *most recent* poll. So a waiter that's
polled again while still waiting must update its stored waker:

```rust
// on re-poll while still waiting:
if !stored.will_wake(cx.waker()) {
    *stored = cx.waker().clone();   // will_wake just skips a clone when unchanged
}
```

Keep a stale waker and the eventual wake may rouse nothing at all — the waiter
sleeps next to the permit that was meant for it.

**Wake outside the lock.** `waker.wake()` runs foreign code — the runtime's
internals, sometimes more. Some executors poll the woken task *synchronously, inside
the `wake()` call*. If that task immediately calls `acquire`, it takes the
semaphore's lock — which the releaser is still holding. The fix reshapes
`add_permits`:

```rust
pub fn add_permits(&self, n: usize) {
    let mut to_wake = Vec::new();
    {
        let mut state = self.state.lock().unwrap();
        state.permits += n;
        for _ in 0..n {
            match state.waiters.pop_front() {
                Some(waker) => to_wake.push(waker),   // collect under the lock…
                None => break,
            }
        }
    }                                                 // …lock released here…
    for waker in to_wake {
        waker.wake();                                 // …then run foreign code
    }
}
```

## What the move upstairs costs

Nothing — and that's the surprise. It's natural to assume the userspace arrangement
is the budget version of the "real" kernel one. The opposite:

| | thread + futex | task + waker |
|---|---|---|
| going to sleep | one syscall | store a waker, return `Pending` |
| waking someone | one syscall | a function call |
| cost while parked | a full OS thread, stack and all | a paused state machine |

Both directions of the round trip stay in userspace. One flag before moving on:
everything here sits behind a single `Mutex`. Production implementations keep the
counter in an atomic so an uncontended `try_acquire` never locks, at the price of
delicate races between the atomic and the queue. That optimization changes none of
the questions ahead, so the lock stays.

## The question the queue asks next

The semaphore now remembers its waiters. So when a permit comes back and three
tasks are waiting, the release side holds a queue and a decision: *who gets it?*

The code above already made a choice without asking — `pop_front` a waker, wake it,
and let the woken task race everyone else for the permit in the counter. It's
simple, it's even correct, and it hides a failure mode that no functional test will
show you but a production load test will find in an afternoon. That fork is Part 4.

---

*Next: [Part 4 — Fairness: who gets the freed permit?](04_fairness.md) · [Index](00_index.md)*

*Deutsch: [`../de/03_where_waiting_lives.md`](../de/03_where_waiting_lives.md)*
