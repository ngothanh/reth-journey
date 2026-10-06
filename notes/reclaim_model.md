# The reclamation model

Output of the teaching plan in `plan/reclaim_modeling.md`. One section per lesson.
§1–§3 are lesson L0, the map; they say what exists and where, not why.

---

## §1 The eight components

| Component | Kind | Role | What it is |
|---|---|---|---|
| `Retire` | trait | object | What an object must offer to be freed by someone else: its link, and its own way of being freed. |
| `RetireLink` | struct | object | A `next` pointer stored inside the object. The scheme uses it to chain retired objects into a list. |
| `Root<T>` | struct | structure | The address of one atomic that readers start reading from. For the queue: `head` and `tail`. |
| `RootRegistry` | struct | scheme | The list of roots a structure has declared. Used only for a check in debug builds. |
| `Reclaim` | trait | scheme | What every scheme offers: hand out a guard, accept a retired object. |
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
      └─ RetireLink × 1 each, implements Retire

        SegQueue calls, through its Domain:
            guard()        "give me something to read with"
            try_protect()  "I am reading this segment"
            retire()       "I am done with this segment"
                 │
                 ▼
Scheme  (Leak now; Hazard and Epoch later)   implements Reclaim
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
