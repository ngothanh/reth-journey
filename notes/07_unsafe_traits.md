# `unsafe` traits and safety contracts

Notes from writing `reclaim`'s `Retirable` trait (C1.1). Companion to
`04_traits.md`, which covered dispatch; this file is about what a trait
*promises* and who is on the hook for it.

The worked example throughout:

```rust
pub struct RetireLink { next: AtomicPtr<()> }

pub unsafe trait Retirable: Send {
    fn retire_link(&self) -> &RetireLink;
    unsafe fn reclaim(ptr: *mut Self);
}
```

---

## 1. `unsafe fn` and `unsafe trait` are different promises

The one that most people conflate.

| | Constrains | Means |
|---|---|---|
| `unsafe fn f()` | the **caller** | calling it has preconditions you must uphold |
| `unsafe trait T` | the **implementor** | implementing it has invariants, **and other safe code relies on them** |

`reclaim` is already `unsafe fn` — that is where "ptr must be valid, not
already freed" lives. The `unsafe` on the *trait* is separate and additional.

**Litmus test for `unsafe trait`:** could someone write a 100 %-safe impl of
this trait that breaks memory safety in code that never writes `unsafe`? Yes →
the trait must be `unsafe`.

Familiar examples: `Send`/`Sync` (the implementor promises thread-safety and
everything relies on it), `GlobalAlloc`, `TrustedLen` (the implementor promises
`size_hint` is exact, and safe code uses that to skip bounds checks).

---

## 2. The impl is not the exhibit

A clause in a `# Safety` block is justified only when you can **exhibit** the
violation. Two different artifacts:

- **the impl** — code that *could* exist. Shows someone could write this.
  Proves nothing on its own.
- **the exhibit** — the sequence, in order, naming who does what, ending in a
  **named** failure.

Template:

```
<the setup: an impl, or the starting state>
1. thread: action
2. thread: action
...
N. → NAMED FAILURE   (use-after-free / double free / data race / leak)
```

The last line always names the failure. "This is bad" is not a failure;
"use-after-free" is.

**Why be strict:** it is the negative-control rule applied to prose. A test
input with no failing assertion proves nothing, which is why a negative control
is mandatory (`reference_concurrent_test_checklist`). Same rule, different
medium. Without exhibits a contract accumulates clauses that are vague,
redundant or simply wrong, with no way to tell which.

**The payoff:** every exhibit is a test you have not written yet. A two-thread
trace is a loom model with the interleaving already spelled out; a trace ending
in a free is a Miri test. Writing `# Safety` this way drafts the test suite.

---

## 3. Why `Send` — the reclaiming thread is not the retiring thread

`retire(p)` transfers ownership to the scheme. The thread that eventually runs
`reclaim` is whichever thread crosses the reclamation threshold — or, once an
offload reclaimer exists, a dedicated thread that outlives the producer
entirely. The object crosses a thread boundary between retirement and drop, and
`Send` is the license to do that.

The second half kills the comfortable hand-wave. Without an offload executor you
might tell yourself "it is probably the same thread anyway." With one, it
provably never is.

**Exhibit** — `Retirable` without the `Send` bound, implemented for a type holding
an `Rc<U>`:

```
T1: retire(p)              // p holds an Rc<U>, refcount 2
T2: (reclaimer) reclaim(p) // drops the Rc → non-atomic refcount decrement
T1: rc.clone()             // non-atomic increment, concurrently
                           → torn refcount → double-free or leak
```

`Rc` is `!Send` precisely to forbid this. The bound on the trait is what
propagates the prohibition.

---

## 4. Why `*mut Self` — a raw pointer is the only type that promises nothing

Each candidate type carries a different implicit promise, and `reclaim` can keep
none of them.

| Type | Promises | Why `reclaim` breaks it |
|---|---|---|
| `&mut self` | valid, uniquely borrowed, **alive for the reference's lifetime** | `reclaim` deallocates. Freeing memory a live `&mut` points into invalidates it while still in scope — Miri's Stacked Borrows flags exactly this |
| `Box<Self>` | the allocation came from the **global allocator** and may be freed by it | That is the promise this trait exists to deny: pool-allocated objects are the named consumers, so `Box` hard-codes the wrong deallocator into the signature |
| `*mut Self` | **nothing** | The only honest option when the function's purpose is to end the object |

The general form: a raw pointer asserts nothing, so every obligation must be
*stated* in `# Safety` rather than smuggled in through a type's built-in
guarantees. You pay in prose and gain accuracy.

Naming discipline, since the two are one grace period apart: **`retire`** is "I
am done with this" (the domain's method); **`reclaim`** is "actually free it"
(the object's).

Because `reclaim` takes no `self` it is an associated function, which is what
lets the scheme hold a plain `(ptr, reclaim_fn)` pair and never need
`dyn Retirable` — monomorphisation turns `T::reclaim` into a function pointer at
the retire site.

---

## 5. Why `AtomicPtr<()>` — the retired list is heterogeneous

One fact makes it click: a domain's shard is a **single chain**, and what hangs
off it is whatever got retired — a queue segment, a stack node, a pool page, a
price level, in arrival order.

Type the link and that breaks:

```rust
pub struct RetireLink<T> { next: AtomicPtr<T> }   // a segment's next can only point at a segment
```

One list per type, one shard set per type, one threshold per type, one scan per
type. The whole point of a domain is one list and one scan.

```rust
pub struct RetireLink { next: AtomicPtr<()> }     // one chain, any types
```

Two consequences worth holding on to:

- **By the time an object is on the retired list, its static type is gone.**
  What survives is an address plus a function that knows how to free that
  address. **Erase the type from the data, keep it in the code.**
- **A thin pointer fits in an atomic word.** `*mut ()` is one word. For a
  `?Sized` type, `*mut Self` is a *fat* pointer — two words — and no atomic
  holds that.

---

## 6. Deriving the contract: the proof you cannot finish

Three levels of `# Safety`:

1. "the caller must ensure `p` is valid" — unfalsifiable, excludes no bug.
2. read your own implementation and list what would break it — better, but
   derived from *the code*, so it rots when the code changes and is structurally
   blind to global properties the code cannot observe.
3. **write the proof you need, fail to finish it, and harvest each hole as a
   clause.**

Level 3 in practice. State the obligation:

> "Dereferencing `p` through a guard after `retire(p)` is sound because ___."

Try to complete it using only what the implementation controls. You cannot — a
reclamation scheme supplies *grace* (threads that already hold `p` eventually
let go) and cannot supply *unobtainability*, which is a property of a data
structure it has never seen. **The hole is the clause**, and it arrives with its
justification already attached, because it is literally the missing step.

Completeness check, both directions:

- every clause needs an exhibit — no exhibit means decoration;
- every exhibit needs a clause — enumerate the UB classes you can actually
  produce (use-after-free, double free, data race, invalid deref, leak) and name
  which clause excludes each.

Done when the two sets match. That is a termination condition, which is more
than most `# Safety` blocks have.

### Then triage by enforcer

| Enforcer | Catches |
|---|---|
| the compiler | types, lifetimes, `Send`/`Sync` |
| a debug assert | double-retire, a reachability walk |
| Miri | UAF, invalid deref, leaks |
| loom | the ordering clauses |
| **nothing but review** | — |

### And the move that matters

**A clause in the "nothing but review" row is a design bug you have not fixed
yet, not documentation.** The question is never "how do I word this well
enough", it is "what API change moves this to a row with an enforcer?"

Worked instance: requirement (A) landed in the unenforceable row, so
`try_protect`'s signature changed to take a declared `Root<T>` — which let a
debug assert walk the roots and check it. That was not documentation work. The
contract named where the design was weak.

If a clause feels unpleasant to commit to, that feeling is the signal. Do not
polish the sentence; ask what changes so the promise is not needed.

---

## 7. Four properties of one getter

`fn retire_link(&self) -> &RetireLink` looks like it promises "returns a link".
It promises four things, each with its own exhibit.

**(1) Ownership** — the storage belongs to this object alone.

```rust
static SHARED: RetireLink = RetireLink::new();
impl Retirable for Node { fn retire_link(&self) -> &RetireLink { &SHARED } }  // zero unsafe
```

```
T1: retire(a)   // scheme: link.next = head (null); head = a   → SHARED.next = null
T1: retire(b)   // scheme: link.next = head (a);    head = b   → SHARED.next = a
    // a and b read the SAME next; b's push overwrote a's
drain: pop b → next = a → free(b), head = a
       pop a → next = a → free(a), head = a
       pop a → reads the link of a freed object                 ← use-after-free
            → free(a) again                                    ← double free, loops forever
```

**(2) Determinism** — the same link on every call. (Not "idempotent" —
idempotence is about repeating an *effect*; this is a pure getter.)

```rust
fn retire_link(&self) -> &RetireLink {
    if self.hot { &self.link_a } else { &self.link_b }
}
```

```
1. T1 sets p.hot = true
2. T1 calls retire(p) → scheme writes link_a.next = head (= q), head = p
3. T2 sets p.hot = false                       ← ordinary safe code
4. drain pops p → retire_link() → &link_b → next = null (never written)
5. q and everything behind it is unreachable from the chain      ← leak
   (if link_b held stale bytes instead → free(garbage)           ← UB)
```

**(3) Stability** — the link's address does not move, for a precisely bounded
window: **from `retire(p)` until `reclaim(p)` returns.** Interior mutability is
the only way to mutate through `&self`, so that is where to look.

```rust
struct Node { inner: UnsafeCell<Box<Inner>> }   // the link lives inside Inner
```

```
T1: retire(p)                                  // scheme stores a pointer to Inner#1.link at X
T2: *p.inner.get() = Box::new(Inner::new());   // Inner#1 freed; the link now lives at Y
drain: scheme follows its stored pointer to X                     ← use-after-free
```

**(4) Non-interference** — the *contents* are the scheme's, for that same
window. You own the storage; the scheme owns the bytes.

```
T1: retire(p)                  // scheme: p.link.next = q
T2: p.link.next.store(null)    // through interior mutability, caller writes no unsafe
drain: pop p → next = null → q and everything behind it           ← leak
```

Note (3) and (4) share one window with a precise start and end. That is what
makes them statable rather than vague — and worth seeing that **one mechanism
(interior mutability) produces two different violations and therefore two
different clauses.**

---

## 8. A bad default is worse than no default

Open question at the time of writing: should `reclaim` have a default body?

```rust
unsafe fn reclaim(ptr: *mut Self) { drop(unsafe { Box::from_raw(ptr) }) }
```

It does not compile as written — `Self` in a trait is implicitly `?Sized` and
`Box::from_raw` needs a size, so `rustc` says *"the size for values of type
`Self` cannot be known at compilation time."* Two ways out: gate the default
with `where Self: Sized`, or require `Sized` on the trait. The second is
defensible when the scheme stores a thin `*mut ()` anyway.

But the real question is whether the default should exist at all, and the
argument is one already settled elsewhere: `SegQueue<T, R = Leak>` was rejected
because "someone writes `SegQueue<usize>` and silently gets an unbounded leak."

```
someone implements Retirable for a pool-allocated page
forgets to override reclaim
the default runs Box::from_raw on memory the global allocator never handed out
→ UB, silently, on a reclaimer thread, minutes later
```

A `Box` default points a footgun at precisely the consumers the overridable
`reclaim` was designed for. Against that: the common case *is* `Box`, and
forcing every impl to restate it is noise.

---

## Blog seeds

Candidate standalone piece: **"What an `unsafe` trait actually promises"** —
sections 1, 2, 6 and 7 are the spine, with `Retirable` as the single worked example
throughout. The rare content is §2 (impl vs exhibit, and exhibits as
pre-written tests) and §6's closing move (an unenforceable clause is a design
backlog item). Sections 3–5 and 8 are the concrete beats that keep it grounded.

Not yet a series — the blog is an output of finished work, and `reclaim` is at
C1.
