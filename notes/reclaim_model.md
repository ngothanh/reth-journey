# The reclamation model

Output of the teaching plan in `plan/reclaim_modeling.md`. One section per lesson.
§1–§3 are lesson L0, the map; they say what exists and where, not why.

---

## §1 The eight components

| Component | Kind | Role | What it is |
|---|---|---|---|
| `Retirable` | trait | object | What an object must offer to be freed by someone else: its link, and its own way of being freed. |
| `RetireLink` | struct | object | A `next` pointer stored inside the object. The scheme uses it to chain retired objects into a list. |
| `Root<T>` | struct | structure | The address of one atomic that readers start reading from. For the queue: `head` and `tail`. |
| `RootRegistry` | struct | scheme | The list of roots a structure has declared. Used only for a check in debug builds. |
| `Reclaimer` | trait | scheme | What every scheme offers: hand out a guard, accept a retired object. |
| `Guard` | trait | scheme | What a reader holds while it reads. It keeps one object safe to read. |
| `Domain<R>` | struct | scheme | A handle to one scheme. The structure stores it and makes all its calls through it. |
| `Leak` | struct | scheme | The simplest scheme: it accepts retired objects and never frees any. |

Three roles:

- **object** — the thing that gets freed. In the queue: `Segment`.
- **structure** — the thing that contains objects. In the queue: `SegQueue`.
- **scheme** — a third party that knows when nobody is reading an object any more. The
  structure does not free; it hands objects to the scheme.

## §2 Who contains whom, who calls whom

```
SegQueue  (structure)
 ├─ head, tail          the two atomics readers start from
 ├─ Root × 2            one for head, one for tail
 ├─ Domain              the handle to the scheme
 │
 └─ Segment → Segment → Segment      (objects)
      └─ RetireLink × 1 each, implements Retirable

        SegQueue calls, through its Domain:
            guard()        "give me something to read with"
            try_protect()  "I am reading this segment"
            retire()       "I am done with this segment"
                 │
                 ▼
Scheme  (Leak now; Hazard and Epoch later)   implements Reclaimer
 ├─ hands out Guards
 ├─ keeps the list of retired segments, chained through their RetireLinks
 ├─ keeps the RootRegistry
 └─ decides when a retired segment can be freed, then calls Segment::reclaim
```

Counts: each `SegQueue` has 2 `Root`s and 1 `Domain`. Each `Segment` has 1 `RetireLink`.

## §3 The life of one segment, against today's `seg_queue.rs`

Today's code predates the crate, so some steps exist under other names, some are fused
together, and some are missing.

| # | Who | What happens | In `seg_queue.rs` today |
|---|---|---|---|
| 1 | queue | Created with a `Domain`; declares roots `head` and `tail`. | `SegQueue::new` creates `head` and `tail`. **No domain, no roots yet.** |
| 2 | queue | Allocates segment `A`; `A` carries a `RetireLink`. | `Box::into_raw(Box::new(Segment::new()))` in `SegQueue::new` and in `advance_tail`. **No link yet** — `Segment` has `ref_count` instead. |
| 3 | reader | Gets a guard. | `SegGuard::acquire(..)` at the top of `push` and `pop`. |
| 4 | reader | Protects `A` through a root; `A` is now safe to read. | The same `SegGuard::acquire`: it calls `acquire_ref`, a `fetch_add` on `A`'s own `ref_count`. Steps 3 and 4 are one call today. |
| 5 | queue | Moves `head` past `A`; new readers cannot reach `A`. | In `pop`: `self.head.compare_exchange(guard.as_ptr(), next, ..)`. |
| 6 | queue | `retire(A)`: hands `A` to the scheme, which chains it. | **Does not exist as its own step.** `pop` calls `try_reclaim`, which uses the trailing `reclaim` cursor instead of a list. |
| 7 | scheme | Debug only: checks `A` is unreachable from the roots. | **Not there.** |
| 8 | old readers | Finish with `A` and let go. | `Drop for SegGuard` → `release_ref`. |
| 9 | scheme | Decides nobody still holds `A`. | In `try_reclaim`: `s.ref_count.load(..) != 0`. This is the check that was proven unsound. |
| 10 | scheme | Calls `A::reclaim`; `A` frees itself. | In `try_reclaim`: `drop(Box::from_raw(seg))`. |

Three things to notice:

- **Steps 6, 9 and 10 are fused** inside `try_reclaim` today. There is no moment where `A` is
  "retired but not yet freed" in someone else's hands — the queue does all three itself.
- **The queue is its own scheme.** The role the crate gives to `Leak` / `Hazard` / `Epoch` is
  played by `ref_count` plus `try_reclaim`.
- **Step 4 protects by writing into `A` itself** (`A.ref_count`). Hold on to that; it is where
  L1 starts.

Words:

- **retire** (step 6) — "I am done with it." Does not free.
- **reclaim** (step 10) — actually free it.
- **grace period** — the wait between 6 and 10, so readers that took `A` before it was retired
  can finish. During it `A`'s memory is still alive and the scheme holds it.
- With `Leak` the life stops after step 8: 9 never happens, so 10 never does, and memory grows.
- The queue runs steps 1–6 the same way whatever the scheme. Only 9 and 10 differ.

---

## §4 The crash, and the two rules  (lesson L1)

### The crash, step by step

Two threads. `T1` is a reader in `pop`. `T2` is another `pop` that moves `head` and then
tries to free the old segment `A`.

```
1. T1 gets the address of A.            head.load()
2. T2 moves head from A to B.           head.compare_exchange(A, B)
3. T2 checks A's counter. It is 0.      try_reclaim: A.ref_count
4. T2 frees A.                          drop(Box::from_raw(A))
5. T1 uses A.                           (*A).acquire_ref()  — writes into freed memory
6. Crash: use-after-free.
```

Two facts make this possible:

- Between line 1 and line 5, the address of A lives only in T1's own local variable. Nothing
  shared has changed, so no other thread can know T1 has it. That is why the counter reads 0.
- T1's first use of A is `acquire_ref`, which writes to a counter stored **inside A**. So the
  act of saying "I am reading A" already needs A to be alive.

### The two rules

Freeing A is safe only when both are true.

| | Rule | Who does it | How, in the queue |
|---|---|---|---|
| **Rule 1** — called (A) in the plan | No **new** reader can get A's address. | the **structure** (the queue) | it moves `head` away from A |
| **Rule 2** — called (B) in the plan | Every reader who **already has** A's address has finished, before A is freed. | the **scheme** | it waits, then frees |

In the crash above, Rule 1 held (line 2 moved `head`). **Rule 2 was broken**: T2 freed A
while T1 still had its address.

### What follows

- The queue must **not free A itself**. It does not know who else has A's address.
- The queue only says "I am done with A" and hands A to the scheme. That is `retire`.
- The scheme waits until readers like T1 have finished, and only then frees A. That is
  `reclaim`. The wait in between is the grace period.

### Why the scheme cannot do Rule 1

1. Making A unreachable means changing the structure's own pointers (here: moving `head`).
2. How to change them correctly is the structure's own algorithm, different for a queue, a
   stack and a list.
3. The scheme only sees the addresses it is handed, so it has nothing to change.

The scheme can **check** Rule 1 if it is told where readers start from (the roots — next
lesson). It can never **make** Rule 1 true.

---

## §5 Roots  (lesson L2)

This section answers: how does a structure make Rule 1 true?

### What a root is

A reader cannot make up an address. It always reads the address from some place in shared
memory.

> A place is a **root** only if a reader takes an address from it **and then opens that
> object**.

Two different things a reader can do with a place:

- **read a number from it** — fine anywhere, as long as the object that contains the place is
  protected. Having a number in a local variable is not the same as reading memory at that
  address.
- **protect through it** — take an address from it and then open that object. That makes the
  place a root.

### The queue today: why nothing could ever be freed

Today `pop` gets addresses from two kinds of places: `head`, and the `next` field inside a
segment (`SegGuard::acquire(next)`). So every segment's `next` is a root.

```
        head
         │
         ▼
   A ──► B ──► C
   ▲
   T1 still holds A
```

`head` has passed A and B. Both were handed to the scheme. T1 still holds A, which is
allowed. T1 reads `A.next` and gets B's address.

- T1 got B's address *after* B was retired, so T1 is a **new** reader of B.
- So Rule 1 is not true for B.
- The queue cannot fix this: `next` is written once and never changed. The scheme cannot fix
  it either; it can only check Rule 1.

While `A.next` is a root there is always a path to B, so B can never be freed safely. The
same holds for every segment. This was not a bug in one line: the set of roots made Rule 1
impossible.

### The fix: roots are `head` and `tail` only

When A is used up, the reader moves on like this:

| Step | What the reader does | Memory it touches |
|---|---|---|
| 1 | holds A, protected | A |
| 2 | reads `A.next`, gets a number (B's address) | inside A |
| 3 | `head.compare_exchange(A, B)` — copies the number into `head` | `head` |
| 4 | reads `head`, protects what it finds, opens it | `head`, then that segment |

Between steps 2 and 4 the reader never touches B. The segment it opens in step 4 got its
address from `head`. So `A.next` is read but is **not** a root.

With roots `{head, tail}`: once `head` has passed a segment, no new reader can get its
address. Rule 1 becomes true just by moving `head`.

### Three structures

| Structure | Roots | Can it make Rule 1 true just by moving its own pointers? |
|---|---|---|
| queue, as the code is today | `head`, `tail`, and every segment's `next` | **No.** Nothing can ever be freed. |
| queue, fixed | `head`, `tail` | **Yes.** Moving `head` is enough. |
| stack (`head ──► N3 ──► N2 ──► N1`) | `head` | **Yes.** Popping a node is enough. |
| sorted linked list | `head` and every node's `next` | **No**, not alone. |

The list is different from the fixed queue because a reader searching it walks node by node
and **opens** each one: it takes N2's address from `N1.next` and opens N2. So every `next` is
a root. If a reader stands on a node that was already removed, that node's `next` is never
updated again, and the reader can still get the address of a later node that has been retired.
The list needs extra help to be reclaimable; that comes much later in the build (edit C10).

### The pattern

**When a root lives inside an object that can itself be removed, Rule 1 breaks.**
`head` and `tail` are safe as roots because they are fields of the structure, and the
structure itself is never retired.

---

## §6 The guard, and how the scheme knows the wait is over  (lesson L3)

This section answers: how does a scheme make Rule 2 true?

### The reader has to tell the scheme

A reader's address sits in its own local variable; nobody else can see it. So the reader must
**tell** the scheme. The old code told by writing into `A.ref_count`, which is inside A — and
that write was itself the crash in §4.

The fix: write the message **outside** the object. Each thread gets a small shared cell, a
**slot**. Slots belong to the scheme and stay alive the whole time.

A **guard** is what the reader holds while its slot is filled in. Dropping the guard clears
the slot.

There are two things a reader can write in its slot. They are the two schemes.

### Way 1 — write the address  (hazard pointers)

The reader writes "I am reading the object at this address". Before freeing A, the scheme
looks through all slots; if none holds A's address, it frees A.

Writing the slot alone is not enough. The message can arrive too late:

```
1. T1 reads head. Gets A's address.
2. T2 moves head to B.
3. T2 hands A to the scheme.
4. The scheme looks through all the slots.   T1's slot is still empty.
5. The scheme frees A.
6. T1 writes A's address into its slot.      The write is fine: the slot is alive.
7. T1 opens A.
8. Crash: use-after-free.
```

So the reader does **three** actions, not two:

1. read the root (`head`), get an address;
2. write that address into the slot;
3. **read the root again and compare.** Same address: open the object. Different: start over
   with the new address, overwriting the slot.

Why the second read is enough — the two cases:

- **The check fails.** `head` changed, so the reader does not open A. Safe.
- **The check passes.** `head` still held A *after* the slot was written. The scheme only
  receives A once `head` has moved away, so the slot was filled in before the scheme could
  look. It is certain to see it:

```
1. T1 reads head.               Gets A.
2. T1 writes A into its slot.
3. T1 reads head again.         Still A.  -> opens A.
4. T2 moves head from A to B.
5. T2 hands A to the scheme.
6. The scheme looks through all the slots.   Finds A's address in T1's slot.
7. The scheme does not free A.               T1 is safe.
```

The slot is reused on every retry: the reader overwrites the old address with the new one.
One slot that keeps being re-pointed is what a guard is in the crate.

### Way 2 — write the time  (epoch)

The scheme keeps one global clock, a number it moves forward from time to time.

```
1. T1 writes "inside since time 5" in its slot.
2. T1 reads head, gets A, opens A.           No message about A itself.
3. T1 reads more: B, C, D.                   Still no new message.
4. T2 moves head, hands A to the scheme.     The scheme labels A "retired at time 5".
5. The scheme looks through the slots.       T1's slot says 5 -> T1 might hold A -> wait.
6. T1 finishes and clears its slot.
7. The scheme looks again.                   No slot says 5 or earlier -> free A.
```

The scheme never knows which objects a reader touches. It only asks one thing: **was this
reader already inside when the object was retired?**

| Case | Reasoning | Scheme does |
|---|---|---|
| reader entered before or at the retire time | it was inside while the object was still reachable; it **might** hold the address | wait |
| reader entered after the retire time | Rule 1 was already true for the object; it **cannot** hold the address | ignore this reader |

Example: a thread stuck inside since time 5, an object retired at time 7. The thread was
inside at 7, so the object is not freed — even if that thread never touched it. The object
*is* retired (handing over always succeeds); only the free is held back.

### The two ways compared

| | Way 1: hazard pointers | Way 2: epoch | Leak |
|---|---|---|---|
| the slot says | one address | a time | nothing |
| one guard protects | **one object** | **every object** opened while the guard is held | everything, forever |
| reader opens A, B, C — messages written | 3 | 1 | 0 |
| cost for each object opened | write the address, read the root again, compare | nothing extra | nothing |
| a stuck reader keeps alive | 1 object | everything retired since it entered | — |

Way 1 knows exactly what each reader holds, so it waits only for that; the reader pays on
every object. Way 2 knows nothing exact, so it over-waits; the reader pays almost nothing.

A shared interface has to be shaped for the stricter one, Way 1: "protect *this* object, got
from *this* root". Way 2 fits it trivially, by doing nothing per object.

---

## §7 The retired object  (lesson L4)

This section answers: between "handed to the scheme" and "freed", where does the object live,
and what makes the scheme actually free it?

### The pile

After `retire(A)` the scheme holds A in a list of retired objects — the pile. One pile holds
objects from every structure that uses that scheme:

```
   A (queue segment) ──► N3 (stack node) ──► P (pool page) ──► ...
```

Each arrow is stored **inside the object itself**: one `next` field per object. That field is
`RetireLink`. The pile needs no extra memory.

### The life of a retired object

1. The structure calls `retire(A)`. From here the scheme owns A.
2. The scheme chains A into the pile through A's own link.
3. A waits. Its memory is alive; readers that already held it may still be reading.
4. Something starts a **round**. The scheme looks through the slots once.
5. The scheme goes down the pile. For each object: can anyone still hold it?
   Yes → it stays in the pile for the next round. No → go to 6.
6. The scheme calls the object's own `reclaim`. The object frees itself.

### Why the scheme frees in rounds

With hazard pointers, deciding about one object means looking through every slot. With 64
threads:

| Approach | Work to retire 1000 objects |
|---|---|
| check at every retire | 64 × 1000 = **64,000** slot-looks |
| let 1000 pile up, look through the 64 slots **once**, remember what was seen, check the 1000 against that | 64 + 1000 ≈ **1,064** steps |

About 60 times cheaper. So objects are allowed to pile up and are freed together.

### When a round starts: two rules

A round runs when **either** is true.

| Rule | Fires when | What it limits |
|---|---|---|
| count | the pile reaches N objects | **how many** retired objects can pile up |
| time | T seconds have passed since the last round | **how long** a retired object can wait |

The count rule alone is not enough. A quiet queue retires 1 segment per minute, reaches 500,
and then stops being used while the program runs for days: 500 never reaches 1000, so those
segments are never freed, although every reader finished long ago. Only the time rule frees
them.

### Three facts about objects in the pile

| Fact | What it forces |
|---|---|
| A round is run by whichever thread triggered it, often not the thread that retired the object. So an object is created on one thread and freed on another. | the object's type must be **`Send`** |
| The pile mixes kinds: a segment made with `Box::new`, a page that must go back to a pool. One line of code cannot free both. | **the object knows how to free itself** — `reclaim` |
| The pile is built from links stored in the objects. | every object carries a **`RetireLink`** and can hand it over — `retire_link` |

Those three are exactly the `Retirable` trait: `Retirable: Send`, `fn retire_link`, `fn reclaim`.

---

## §8 The model as an interface  (lesson L5)

Every trait and struct, with its signatures and no bodies. Each line traces back to a step of
the life in §3 or a rule in §4–§7.

### The six operations

| Step of the life | Operation | Offered by |
|---|---|---|
| 1 | declare a root | `Domain` |
| 3 | give me a guard | the scheme (`Reclaimer`), through `Domain` |
| 4 | "I am reading this object, got from this root" | the `Guard` |
| 6 | "I am done with this object" — `retire` | the scheme (`Reclaimer`), through `Domain` |
| 6 | give me your link | the object (`Retirable`) |
| 10 | free yourself — `reclaim` | the object (`Retirable`) |

The queue calls `retire` on the scheme. Later, the scheme calls `reclaim` on the object.

### Object side

```rust
/// One arrow of the pile, stored inside the object.            (§7)
pub struct RetireLink { next: AtomicPtr<()> }

/// "I can be retired."  Implemented by Segment, a stack node, a pool page.
pub unsafe trait Retirable: Send {            // Send: freed on another thread   (§7)
    fn retire_link(&self) -> &RetireLink;     // the scheme chains me through this
    unsafe fn reclaim(ptr: *mut Self);        // I know how to free myself       (§7)
}
```

### Roots

```rust
/// The address of one place readers start from. One word.      (§5)
pub struct Root<T> { place: *const AtomicPtr<T> }

/// The list of declared roots of one domain. Used by the debug check (step 7).
struct RootRegistry { /* addresses */ }
```

### Reader side

```rust
/// What a reader holds while its slot is filled in.            (§6)
pub trait Guard {
    /// Write `addr` into my slot, read `root` again, compare.
    /// Ok(addr)     — still the same: `addr` is protected and may be opened.
    /// Err(current) — the root now holds `current`: do not open `addr`; retry with `current`.
    fn try_protect<T>(&mut self, addr: *mut T, root: &Root<T>) -> Result<*mut T, *mut T>;

    /// The retry loop, written once on top of `try_protect`.
    fn protect<T>(&mut self, root: &Root<T>) -> *mut T { /* read root; loop on try_protect */ }
}
```

- `&mut self` — the guard changes its own slot, and the same slot is re-pointed on each try.
- `root: &Root<T>`, never a bare `AtomicPtr` — so an atomic inside a removable object (like
  `A.next`) cannot be protected through unless it was declared a root on purpose.
- A normal `fn` — writing an address into a slot hurts nothing. Opening the object is the
  dangerous part, and that happens in the caller's own `unsafe` block.
- For epoch and `Leak`, `try_protect` writes nothing and always returns `Ok`, so `protect`
  runs its loop once.

### Scheme side

```rust
/// "I am the one who reclaims."  Implemented by Leak, later Hazard and Epoch.
pub unsafe trait Reclaimer {
    type Guard: Guard;                        // each scheme has its own kind of guard  (§6)

    fn guard(&self) -> Self::Guard;

    unsafe fn retire<T: Retirable>(&self, obj: *mut T);
}
```

### The handle

```rust
/// The one thing a structure holds: a scheme plus its root list. Cheap to clone.
pub struct Domain<R: Reclaimer> { /* shared: the scheme R, a RootRegistry */ }

impl<R: Reclaimer> Domain<R> {
    unsafe fn declare_root<T>(&self, place: &AtomicPtr<T>) -> Root<T>;
    fn remove_root<T>(&self, root: Root<T>);

    fn guard(&self) -> R::Guard;                              // passes on to the scheme
    unsafe fn retire<T: Retirable>(&self, obj: *mut T);       // passes on to the scheme
}
```

A structure removes its roots in its own `Drop`. Otherwise the domain, which can outlive the
structure, keeps the address of a field that no longer exists, and the debug check reads it.

### `Leak`

The simplest scheme. Its guard holds nothing. `try_protect` returns `Ok` at once. `retire`
chains the object into the pile through its link and never runs a round, so `reclaim` is never
called. It exists so everything else can be built and tested before a real scheme does.

### Every `unsafe`, and the promise behind it

| Where | Who promises | The promise |
|---|---|---|
| `unsafe trait Retirable` | the object's author | the link is mine alone and stays put; `reclaim` really frees me, once |
| `unsafe fn reclaim` | the scheme (caller) | nobody holds this object any more |
| `unsafe trait Reclaimer` | the scheme's author | Rule 2: never free an object while a guard protects it |
| `unsafe fn retire` | the structure (caller) | Rule 1 holds for the object; it is retired once; it is not used again |
| `unsafe fn declare_root` | the structure (caller) | the place stays alive until `remove_root` |

### Not modelled yet — comes up during the build

- **Two guards at once.** Walking a list holds two objects at a time and swaps the guards as
  it moves; `Guard` will need a way to swap.
- **A guard's states.** A guard can be holding no slot, holding an empty slot, or pointing at
  an object; what each call does in each state.
- **A root that skips the list.** A structure whose roots cannot be listed (the sorted list)
  needs a way to make a `Root` without registering it, taking Rule 1 fully on itself.
- **How a round is run**: the count and time rules as code, who pays for a round. That is the
  build ladder's C2–C4, after C1.

### Names

`Retirable` (object) and `Reclaimer` (scheme) replace the earlier `Retire` and `Reclaim`. The
old names put the method `reclaim` inside a trait called `Retire` and the method `retire`
inside a trait called `Reclaim`, which made the two words easy to swap.

---

## §9 The model against the earlier decisions  (lesson L5)

Before the model was built, twelve design decisions were recorded in
`plan/reclaim_ladder.md`, step 1. This checks each against §1–§8. "Same" means the model
reached the same answer from its own reasons.

| # | Earlier decision | In the model | Result |
|---|---|---|---|
| 1 | The pile belongs to the scheme, not to the structure. | §7: after `retire` the scheme owns the object and keeps the pile. | same |
| 2 | A guard is re-pointed, not made new for each object. | §6: one slot, overwritten on each try; §8: `try_protect(&mut self, ..)`. | same |
| 3 | Protecting gives back a plain address (`*mut T`), not a reference. | §8: `try_protect` and `protect` return `*mut T`; opening it is the caller's `unsafe`. | same |
| 4 | `try_protect` is the one required call; the retry loop `protect` is written once on top. | §8, exactly. | same |
| 5 | `Ok`/`Err` mean "protected or not", so epoch never returns `Err`. | §8: for epoch and `Leak` it writes nothing and always returns `Ok`. | same |
| 6 | The structure's code is identical under every scheme. | §3: the queue runs steps 1–6 the same way; only 9 and 10 differ. | same |
| 7 | Each scheme writes its own `retire`; what they share is when a round starts. | §8: `retire` is a method of `Reclaimer`; §7: the count and time rules. | same |
| 8 | Roots are declared in code, not just described in a comment. | §5 and §8: `declare_root`, and `try_protect` accepts only `&Root<T>`. | same |
| 9 | `Domain` is a cheap handle the structure stores. | §8: `Domain<R>`, the one thing a structure holds. | same |
| 10 | The object frees itself; `retire` takes no "how to free" argument. | §7 and §8: `Retirable::reclaim`. | same |
| 11 | Guards can be held several at a time, with no fixed limit. | Not modelled. §8 lists "two guards at once" as open. | **gap** |
| 12 | `SegQueue<T, R>` gets no default scheme, and never `Leak` as a default. | Not part of this model; it is a decision about the queue. | outside |

### Where the model differs

- **Names.** `Retire` → `Retirable`, `Reclaim` → `Reclaimer`. The model is right: the old
  names made `retire` and `reclaim` easy to swap. The plan files now use the new names.
- **A root that is not registered.** The earlier decision included a second way to make a
  root (`assume_root`) for structures whose roots cannot be listed. The model has only
  `declare_root`. The model is incomplete here, not wrong: nothing in the queue needs it. It
  is on the "not modelled yet" list in §8 and arrives with the sorted list.
- **Several guards at once** (row 11). Same status: needed by the sorted list, not by the
  queue or the stack.

### What the model adds

- **`remove_root`, called from the structure's `Drop`.** The earlier decision said removing
  roots was needed; the model says who calls it and why (the domain can outlive the structure).
- **The reason for each `unsafe`**, as a table of who promises what (§8). The earlier
  decisions gave signatures; the model gives the promise behind each one.

Nothing in the model contradicts an earlier decision. Two earlier decisions (the unregistered
root, several guards) are not covered yet, and both are needed only by the sorted list.

---

**Modeling ends here.** Next is Stage 2 in `plan/reclaim_modeling.md`: the core types, L6.
