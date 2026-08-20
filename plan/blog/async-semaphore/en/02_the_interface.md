# Part 2 — The interface, read off the use cases

Turning the brief into Rust starts with the two operations everyone already knows a
semaphore has. Ask anyone to sketch the API and you'll get something like this:

```rust
impl Semaphore {
    pub fn new(permits: usize) -> Self;
    pub fn acquire(&self);   // take a permit, waiting if necessary
    pub fn release(&self);   // give it back
}
```

It looks complete. By the end of this part, `release` will be gone entirely, `acquire`
will have grown a return type with a story behind every piece of it, and three methods
will have appeared that this sketch never imagined. None of those changes are matters
of taste — each one is forced by a use case from Part 1, and walking through *why* is
the fastest way to actually understand the interface rather than memorize it.

## The permit returns itself

Start with the connection pool, because it breaks the sketch immediately. A pool hands
out permits that stand for connections, and the deal is strict: hold the permit
exactly as long as you hold the connection, then give it back. The sketch makes
giving-back the caller's job — call `release` when you're done. Now count the ways a
caller is "done": the happy path, yes, but also the early return on a bad request, the
`?` that propagates an error, the panic, the future that gets dropped halfway through.
Miss `release` on *any* of those and one connection slot is gone forever. A pool run
this way doesn't fail loudly; it shrinks quietly, one forgotten path at a time, until
the service is waiting on a pool of zero.

The problem isn't carelessness — it's that the sketch encodes returning as a *duty*,
and duties get forgotten. Rust has a better tool: make the permit a *value*, and make
returning it what the value does when it goes away.

```rust
let permit = sem.acquire().await;   // permit: SemaphorePermit
run_expensive_thing().await;
// permit goes out of scope here — returned automatically, on every path
```

`SemaphorePermit` returns itself in its destructor. The error path, the panic path,
the dropped-future path — all of them run destructors, so all of them return the
permit. This is the same move `Mutex::lock` makes with its guard, applied to
capacity instead of data. And notice what happened to the public API: **`release` is
no longer in it.** Dropping the permit *is* the release. An operation the caller
can't forget is an operation the caller can't get wrong.

## But something like release creeps back

With permits returning themselves, is a release-shaped method ever needed? Once, for
a different job. Part 1's brief included a gate that starts closed: a semaphore
created with *zero* permits, where waiters queue up until some event opens the flow.
Opening it means conjuring permits that were never taken out:

```rust
pub fn add_permits(&self, n: usize);
```

It's worth being precise about the distinction, because the two ideas usually get
smeared together under the word "release." Dropping a permit *returns what was
borrowed* — routine, automatic, happens constantly. `add_permits` *mints new
capacity* — deliberate, rare, changes what the semaphore is. Keeping them separate
means the routine operation stays impossible to misuse, and the rare one looks
appropriately unusual at the call site.

## Two ways to fail, and the honesty to say which

The load-shedding use case asked for a different acquire: don't wait, just tell me
*now*. That's `try_acquire`, and its return type is where the design starts talking:

```rust
pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, TryAcquireError>;

pub enum TryAcquireError {
    NoPermits,   // busy right now — routine; back off or shed load
    Closed,      // shut down — permanent; stop trying
}
```

Why an enum, rather than a simple `Option`? Because the two failures ask the caller
to do opposite things. "No permits right now" is Tuesday — retry later, shed the
request, take the fallback. "Closed" means the semaphore is gone for good and looping
on retry would spin forever. A caller who can't tell these apart has to guess, and a
load-shedding path that guesses wrong turns shutdown into a retry storm.

Which raises the question that enum smuggled in: closed? Where did that come from?

## Shutdown is part of the interface

Imagine the server from Part 1 shutting down while forty tasks sit parked inside
`acquire`. The work that would have released permits to them is being torn down —
those permits are never coming. Without help, forty tasks wait forever, and "graceful
shutdown" becomes a hang that someone eventually resolves with `kill -9`.

Someone has to be able to tell the semaphore: it's over — everyone out.

```rust
pub fn close(&self);
```

`close` wakes every parked waiter with an error instead of a permit, and makes every
future `acquire` fail fast. It's one method, but its existence ripples backward into
the type of `acquire` itself — because if waiting can end in "the semaphore closed,"
then `acquire` has a failure mode, and the signature has to say so:

```rust
pub struct AcquireError;   // means exactly one thing: closed

// acquire's eventual output:
Result<SemaphorePermit<'_>, AcquireError>
```

Note what `AcquireError` is *not*: it has no `NoPermits` variant, and that's not an
oversight. `acquire` cannot fail for lack of permits — waiting for permits is its
entire job. The only thing that can end the wait unhappily is closure. So `acquire`
and `try_acquire` get *different* error types, each listing exactly the outcomes its
method can produce, and a caller matching on either one is never asked to handle an
outcome that can't happen.

## The signature that pays off three parts from now

There's one decision left, and it's invisible until you look closely. The natural
way to write an async `acquire` is:

```rust
pub async fn acquire(&self) -> Result<SemaphorePermit<'_>, AcquireError>;
```

An `async fn` compiles into a future type the compiler invents — anonymous, and
untouchable. You cannot implement a trait for a type you can't name, and one trait is
about to matter enormously: `Drop`. Part 1 promised that a waiting task can be
cancelled — its future dropped mid-wait — and Part 5 will show the semaphore has real
cleanup to do at that moment. Cleanup when a future is dropped means custom `Drop`
logic *on the future*. So the future must be a type we own:

```rust
pub fn acquire(&self) -> Acquire<'_>;    // a named future...

pub struct Acquire<'a> { /* … */ }        // ...that we can implement Drop for
```

To callers, nothing changes — `sem.acquire().await` reads exactly the same. But
somewhere to put cancellation cleanup now exists. The general lesson travels beyond
semaphores: any async primitive that must react to a waiter *disappearing* needs a
named future, and if you discover that after shipping the `async fn`, it's not a
refactor — it's an API break.

## One permit, two lifetimes

The permit as designed borrows the semaphore — `SemaphorePermit<'a>` holds
`&'a Semaphore` so its destructor knows where to return the permit. For the "cap this
section" use case, perfect: zero overhead, scope-shaped. But hand a permit to a
spawned task and the borrow breaks down, because `tokio::spawn` demands `'static` —
a spawned task may outlive the function that created it, so it can't carry borrows of
that function's locals.

The pool use case does exactly this — permits riding inside spawned connection
handlers. So the interface grows an owned variant:

```rust
pub fn acquire_owned(self: Arc<Semaphore>) -> AcquireOwned;
// yields OwnedSemaphorePermit — holds an Arc, is 'static, crosses spawn freely
```

Borrowed for scoped caps, owned for permits that outlive their scope; the cost of
owned is one refcount bump. Server code that spawns per request ends up using
`acquire_owned` almost everywhere.

## The assembled interface

```rust
impl Semaphore {
    pub fn new(permits: usize) -> Self;
    pub fn available_permits(&self) -> usize;

    pub fn acquire(&self) -> Acquire<'_>;
    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, TryAcquireError>;
    pub fn acquire_owned(self: Arc<Self>) -> AcquireOwned;

    pub fn add_permits(&self, n: usize);
    pub fn close(&self);
}

pub struct SemaphorePermit<'a>;                  // RAII: drop = return
pub struct AcquireError;                         // closed — acquire's only failure
pub enum TryAcquireError { NoPermits, Closed }   // busy is routine, closed is final
```

Every method now has a paper trail back to Part 1. But look at what the signatures
*don't* say: nothing here promises fairness, and nothing promises that cancellation
is safe. Those two live below the interface, as properties of the machinery inside —
and both of them hang on a question the interface politely refuses to answer. When
`acquire` has no permit to give and the caller must wait: *where, physically, does
that waiting happen?* A thread can stop and wait. A task is not a thread. Part 3.

---

*Next: [Part 3 — Where does the waiting live?](03_where_waiting_lives.md) · [Index](00_index.md)*

*Deutsch: [`../de/02_the_interface.md`](../de/02_the_interface.md)*
