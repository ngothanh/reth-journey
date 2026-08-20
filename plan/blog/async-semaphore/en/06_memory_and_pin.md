# Part 6 — Where the waiters live, and what Pin is for

Every waiter carries a small bundle of facts: its waker, its lifecycle state, its
place in line. Part 5 called it "my record" and carefully avoided saying where it
lives. That dodge ends here, because the address decides whether `acquire`
allocates — and Part 1 put a semaphore inside every bounded channel, on the hot
path of every `send`. A heap allocation is the wrong guest there: usually fast,
occasionally very slow, always unpredictable. Latency tails are made of exactly
that.

There are two possible homes for the record.

## The obvious address: inside the semaphore

Let the semaphore own its waiters' records — a growable collection of them, plus a
list maintaining arrival order, and each future carries an index to find its own:

```rust
struct State {
    permits: usize,
    records: Slab<Waiter>,      // storage: all waiter records, heap-backed
    order:   VecDeque<usize>,   // arrival order: indices into `records`
}

pub struct Acquire<'a> {
    semaphore: &'a Semaphore,
    key: Option<usize>,         // "my record is records[key]" — None until enqueued
}
```

This is honest, entirely safe Rust, and it works. Its costs are the quiet kind.
Enqueueing can allocate — and occasionally reallocate the whole backing store,
exactly the unpredictable spike the hot path can't host. Storage and order are two
structures that must never disagree, so every operation touches both. And Part 5's
pocketed detail comes due: cancellation removes an entry from the *middle*, which
against `order` means a linear scan:

```rust
// Drop, cancelled while waiting:
state.order.retain(|&k| k != my_key);   // O(n) scan to remove myself
state.records.remove(my_key);
```

There's also a subtler wrongness that points at the fix. The record's natural owner
isn't the semaphore — it's the *waiter*. A record is born when a waiter enqueues
and dies when that waiter leaves: identical lifetimes. Housing it in the semaphore
makes the semaphore a landlord for tenants whose leases it doesn't understand.

## The strange address: inside the future itself

Follow the ownership. The record lives exactly as long as the wait — and a piece of
memory with precisely that lifetime already exists: the `Acquire` future. The
caller is *already holding* the wait's memory. So put the record inside the future,
and let the queue be nothing but pointers threading through the waiters:

```rust
struct Waiter {
    waker: Option<Waker>,
    granted: bool,
    prev: Option<NonNull<Waiter>>,   // ← the queue's links live IN the record
    next: Option<NonNull<Waiter>>,
}

pub struct Acquire<'a> {
    semaphore: &'a Semaphore,
    node: Waiter,                    // ← the record lives IN the future
}

struct State {
    permits: usize,
    head: Option<NonNull<Waiter>>,   // the entire "queue": two pointers
    tail: Option<NonNull<Waiter>>,
}
```

This is an *intrusive* list — links inside the payload instead of nodes allocated
around it. The bookkeeping from option one evaporates: allocation per waiter,
none — the future's memory exists either way; structures to synchronize, one — the
pointer chain *is* both storage and order; removal from the middle, two pointer
writes:

```rust
// Drop, cancelled while waiting — O(1), no scan:
unsafe {
    (*prev.as_ptr()).next = my.next;   // my neighbors now point past me
    (*next.as_ptr()).prev = my.prev;
}
```

(Doubly linked precisely so this works: an unlinking node must reach both
neighbors. Cancellation is the operation that chooses the data structure.)

tokio's semaphore does exactly this. But something alarming just happened, and it
deserves plain words: **the semaphore now holds pointers into the interiors of
futures it does not own.**

## The problem: futures move

A future is an ordinary Rust value, and ordinary values *move*. All of this is
safe, everyday code:

```rust
let fut = sem.acquire();          // Acquire on the stack, node inside it
let boxed = Box::new(fut);        // MOVE: every byte, node included, copied to the heap
let fut2 = returns_a_future();    // MOVE: out of the callee's frame into ours
tokio::spawn(async move { … });   // MOVE: the whole async block into the task
```

A move is a memcpy. The bytes relocate and no one notifies whoever memorized the
old address. Run the failure in slow motion:

```
1.  fut.node is linked:      head ──► &fut.node   (address 0x7ffd_1000, on the stack)
2.  fut moves into a Box:    node's bytes now live at 0x5561_2000 (heap)
3.  the queue still points at 0x7ffd_1000 — dead stack memory
4.  next release: (*head).granted = true    ← WRITE through a dangling pointer
```

Not a leak, not a logic bug — memory corruption, surfacing far from its cause. The
intrusive design is sound only under one guarantee: **once a future's node is
linked into the queue, that future must never move again.**

Rust has a type whose entire purpose is to provide that guarantee. You've been
writing it in every `poll` signature since Part 2, probably without needing it
once: `Pin`.

## What Pin actually does — watched closely

Start from the default. Every type in Rust is automatically `Unpin`, which means:
"pinning me is meaningless, I'm free to move." For an `Unpin` type, `Pin` is a
no-op wrapper — which is why `poll` implementations casually escape it:

```rust
// Acquire as defined so far is Unpin (all its fields are), so this works:
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this: &mut Acquire = self.get_mut();   // ✓ compiles — Unpin lets you out
    …
}
```

`&mut Acquire` is precisely the power to move the value (`mem::swap`,
`mem::replace`, plain assignment all take `&mut`). So as long as `get_mut` works,
the "never moves again" guarantee doesn't exist. To make it exist, we declare that
our type must not be unpinned — one marker field:

```rust
use core::marker::PhantomPinned;

struct Waiter {
    waker: Option<Waker>,
    granted: bool,
    prev: Option<NonNull<Waiter>>,
    next: Option<NonNull<Waiter>>,
    _pin: PhantomPinned,      // ← Waiter is now !Unpin; Acquire, containing it, too
}
```

Nothing about the bytes changed — `PhantomPinned` is zero-sized. What changed is
what the compiler will let us write. The `get_mut` above now fails:

```
error[E0277]: `PhantomPinned` cannot be unpinned
   --> src/semaphore.rs:88:31
    |
 88 |         let this = self.get_mut();
    |                         ^^^^^^^ within `Acquire<'_>`, the trait `Unpin`
    |                                 is not implemented for `PhantomPinned`
    |
    = note: consider using `Box::pin`
    = note: required because it appears within the type `Waiter`
    = note: required because it appears within the type `Acquire<'_>`
```

This is `Pin`'s entire mechanism, visible in one error: it doesn't lock memory or
install a runtime guard — it **withholds `&mut`**. Every safe way to move a value
needs `&mut`; `Pin` refuses to produce one for a `!Unpin` type; therefore, safe
code cannot move a pinned `Acquire`. The guarantee is enforced the way the borrow
checker enforces everything: the violating program doesn't compile.

Inside `poll` we still need to touch our fields, so we take the escape hatch and
accept its condition:

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // SAFETY: we use `this` to read/write fields and to take the node's address.
    // We never move out of it — that's the promise unsafe is asking us to keep.
    let this: &mut Acquire = unsafe { self.get_unchecked_mut() };

    let node = NonNull::from(&mut this.node);   // safe to STORE now:
    …                                           // a pinned future's address is final
}
```

## Why the timing works out

The guarantee has a shape worth seeing end to end, because each phase is doing a
job:

```
created ──── moves freely ────► pinned ──── polled, polled… ────► dropped
             (legal: it has            (poll may now link            (Drop unlinks
              never been polled,        node into the queue —         BEFORE the
              so it's linked            the address is frozen)        memory goes —
              nowhere; no pointers                                    Part 5 already
              into it exist)                                          wrote this!)
```

Before its first poll a future has never run, so it can't have linked itself
anywhere — no pointers into it exist, and moving it is harmless. Rust permits
exactly that: `Box::new(fut)`, `spawn(fut)`, all fine. The runtime then pins every
future *before its first poll* — spawned futures into the task's allocation,
`.await`ed ones inside their parent — and from that point `poll` receives
`Pin<&mut Self>` as proof the address is final. Pointers created after pinning
point at memory that can no longer walk away.

And the exit is already guarded: Part 5's `Drop` unlinks the node — for accounting
reasons — before the future's memory is released. Pointers into the future are
created only after pinning and destroyed before deallocation. The window in which
they exist is exactly the window in which they're valid. Every clause of that
argument is enforced by the language except unlink-on-drop, which is ours — and
which we'd already written before `Pin` ever came up.

## The honest trade

Option one pays at runtime: allocations, scans, two structures kept agreeing.
Option two pays in obligations: the link surgery is `unsafe` the compiler won't
check, records may only be touched under the semaphore's lock, and the pinning
contract threads through everything. Identical behavior; the second is what you
ship when the semaphore sits under a channel moving a million messages a second.

Obligations the compiler won't check need something else to check them. That — plus
finally writing the whole thing down — is Part 7.

---

*Next: [Part 7 — Writing it down, and trusting it](07_implementation.md) · [Index](00_index.md)*

*Deutsch: [`../de/06_memory_and_pin.md`](../de/06_memory_and_pin.md)*
