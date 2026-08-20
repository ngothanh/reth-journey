# Part 7 — Writing it down, and trusting it

Six parts in, the design has been quietly accumulating rules. Collected in one
place: a queued waiter never takes from the counter. A dropped waiter returns
whatever it owns. A permit is consumed exactly once. A node is unlinked before its
memory goes away. A released permit goes to the front of the line first, the
counter second. Closing fails every waiter exactly once.

Nothing on that list is difficult by itself. The difficulty is who has to obey it:
four different functions — `poll`, the release path, `Drop`, and `close` — each
touching the same shared state from a different direction. Written naively, each
function carries its own private copy of the rules, and correctness lives in the
*agreements between* them: `Drop` must know whether release already granted this
waiter; `poll` must know what `Drop` will do later. Change one function, re-verify
the other three. Code like that can pass every test and still be unmaintainable,
because the invariants exist only in the gaps between functions, where no reader
can see them.

The last idea of this series is what to do about that.

## The lifecycle becomes a field

Part 5 ended with `granted: bool` creaking — a three-way distinction crammed into a
boolean. Promote it. The lifecycle diagram becomes a type, and the type lives in
the node:

```rust
enum WaiterState {
    Idle,      // created, never enqueued
    Waiting,   // in the queue, owns its place in line
    Granted,   // out of the queue, a permit is sitting in its name
    Done,      // owns nothing — consumed or cancelled
}

struct Waiter {
    waker: Option<Waker>,
    state: WaiterState,               // ← replaces `granted: bool`
    prev:  Option<NonNull<Waiter>>,
    next:  Option<NonNull<Waiter>>,
    _pin:  PhantomPinned,
}
```

Every rule from the pile becomes a *transition* — one arrow, owned by one function:

| who | finds the waiter in | does | leaving it in |
|---|---|---|---|
| `poll` | Idle, permit free | takes from the counter | Done |
| `poll` | Idle, none free | enqueues itself | Waiting |
| release | Waiting, at front | assigns permit, wakes | Granted |
| `poll` | Granted | consumes its permit | Done |
| `Drop` | Waiting | unlinks its node | Done |
| `Drop` | Granted | re-releases its permit | Done |
| `Drop` | Idle or Done | nothing — nothing is owned | — |
| `close` | Waiting | wakes it with an error | (its own poll unlinks) |

And each function becomes a `match` that transcribes its rows. `poll`, in full
shape:

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // SAFETY: we never move out of `this` (Part 6's promise)
    let this = unsafe { self.get_unchecked_mut() };
    let mut state = this.semaphore.state.lock().unwrap();

    match this.node.state {
        WaiterState::Idle if state.closed => {
            this.node.state = WaiterState::Done;
            Poll::Ready(Err(AcquireError))
        }
        WaiterState::Idle if state.permits > 0 => {
            state.permits -= 1;
            this.node.state = WaiterState::Done;
            Poll::Ready(Ok(SemaphorePermit { semaphore: this.semaphore }))
        }
        WaiterState::Idle => {
            this.node.waker = Some(cx.waker().clone());
            this.node.state = WaiterState::Waiting;
            let node = NonNull::from(&mut this.node);
            unsafe { state.queue.push_back(node) };      // pinned → address is final
            Poll::Pending
        }
        WaiterState::Waiting if state.closed => {
            unsafe { state.queue.unlink(NonNull::from(&mut this.node)) };
            this.node.state = WaiterState::Done;
            Poll::Ready(Err(AcquireError))
        }
        WaiterState::Waiting => {
            this.node.update_waker(cx.waker());          // Part 3's refresh rule
            Poll::Pending
        }
        WaiterState::Granted => {
            this.node.state = WaiterState::Done;         // ← the anti-mint line
            Poll::Ready(Ok(SemaphorePermit { semaphore: this.semaphore }))
        }
        WaiterState::Done => unreachable!("polled after completion"),
    }
}
```

`Drop` transcribes its three rows:

```rust
impl Drop for Acquire<'_> {
    fn drop(&mut self) {
        let mut to_wake = WakerList::new();
        {
            let mut state = self.semaphore.state.lock().unwrap();
            match self.node.state {
                WaiterState::Waiting => {
                    // I own a place in line — take my node out with me
                    unsafe { state.queue.unlink(NonNull::from(&mut self.node)) };
                }
                WaiterState::Granted => {
                    // I own a permit I'll never consume — pass it on
                    release_one(&mut state, &mut to_wake);
                }
                WaiterState::Idle | WaiterState::Done => {}   // I own nothing
            }
        }
        to_wake.wake_all();     // outside the lock — Part 3's rule
    }
}
```

And release, shared by `add_permits` and the `Granted` arm above:

```rust
fn release_one(state: &mut State, to_wake: &mut WakerList) {
    match state.queue.pop_front() {
        Some(mut node) => unsafe {
            let node = node.as_mut();
            node.state = WaiterState::Granted;       // the permit enters the record
            if let Some(w) = node.waker.take() { to_wake.push(w); }
        },
        None => state.permits += 1,                  // spill only when nobody waits
    }
}
```

Read Part 5's two nightmares directly off the table. The leak — a granted waiter
dropped in the wake-to-run window — is handled because the `Drop`-at-Granted arrow
*exists*. The double-return is impossible because consuming moves the state to
Done, and the `Drop`-at-Done row does *nothing*. One bug is an arrow that exists;
the other is an arrow that doesn't. Anyone can audit that, one row at a time.

The placement detail that carries it all: the state lives in the *node* — not in
the future's private fields — because the release side reaches waiters through
queue pointers and must read and flip states through them. Whatever both sides
must see goes in the node; the node is only touched under the lock; that sentence
is the entire concurrency story.

## Trusting the safe parts

The transition table has a pleasant property: every arrow is a test. Drive `poll`
by hand — no runtime, no threads, no sleeps — and each row becomes a deterministic
assertion:

```rust
#[test]
fn cancelled_after_grant_returns_the_permit() {   // the Part 5 bug, pinned forever
    let sem = Semaphore::new(0);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(&waker);

    let mut fut = Box::pin(sem.acquire());        // Box::pin OWNS the future,
                                                  // so drop() below truly drops it
    assert!(fut.as_mut().poll(&mut cx).is_pending());  // Idle → Waiting
    sem.add_permits(1);                                // Waiting → Granted
    drop(fut);                                         // dropped in the gap!

    assert_eq!(sem.available_permits(), 1);       // Drop@Granted re-released ✓
}
```

Three lines of act, one assert, microseconds to run — and it fails loudly if
anyone ever breaks the re-release arrow. Every behavioral promise in the design
can be nailed down this way, permanently.

## Trusting the unsafe parts

Part 6's pointer surgery left something tests cannot reach, and it deserves
stating without comfort: **`unsafe` code can be wrong and pass every test you
write.** Undefined behavior is not a wrong output you can assert against — it's a
violated assumption that today's compiler, on today's machine, happens to
translate into working code. Your test suite executes one translation of the
program. The violation lives in the translations you didn't run: a different
optimization level, a newer compiler, another platform.

So the unsafe parts get checking matched to the promise. For memory promises —
every dereference hits a live allocation, every write goes through a pointer with
write permission, no forbidden aliasing — the tool is **Miri**:

```
$ cargo +nightly miri test
```

It runs the same deterministic tests above on an abstract machine that tracks what
real execution can't see, and it turns Part 6's pointer argument ("created after
pinning, destroyed before deallocation, touched only under the lock") from an
argument into a mechanical check.

For ordering promises — correctness under every thread interleaving — the tool is
**loom**. Our design gives it little to find, and the reason is worth noticing:
everything shared sits behind one `Mutex`, so interleavings collapse into "who took
the lock first." That's what the single-lock simplification bought. The day the
counter moves out of the lock into an atomic fast path — the tokio optimization
flagged in Part 3 — is the day loom stops being optional.

## The whole thing, walked once

A caller writes one line:

```rust
let permit = sem.acquire().await?;
```

and beneath it, in order of appearance: a permit that returns itself, because
pools leak otherwise (Part 2). A named future, because cancellation needs a `Drop`
(Part 2). A queue of wakers behind a lock, because the kernel can't park a task
(Part 3). Hand-off to the front of that queue, because racing starves under load
(Part 4). A lifecycle that knows what every waiter owns, because futures vanish
mid-wait (Part 5). Nodes living inside the futures themselves, pinned, because the
hot path can't afford the allocator (Part 6). One state field ruling four
functions, checked by hand-driven tests and Miri (this part).

Strip out any layer and some specific use case from Part 1 breaks. No step needed
cleverness — only the refusal to skip a question.

To see this skeleton wearing production muscle, read tokio's `batch_semaphore.rs`
beside this series — batched wakeups, an atomic fast path, `acquire_many` — you'll
recognize every bone. And the next series starts where this one's comfort ends: a
queue with multiple producers, multiple consumers, and no lock to hide behind —
where correctness rests entirely on atomic orderings, and the first question is
one this series never had to ask: without a lock, what does "before" even mean?

---

*That's the series. · [Index](00_index.md)*

*Deutsch: [`../de/07_implementation.md`](../de/07_implementation.md)*
