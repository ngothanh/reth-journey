# Part 6 — From the model down to code: `static` and `shared`

The first five parts built the *model*: a `Bytes` made of `ptr` + `len` (which bytes)
and `data` + `vtable` (who owns it), with three ways of owning — `static`,
`promotable`, `shared`. From here on we *sit down and write it*. And the pleasant part
is that two of the three ways of owning are almost trivial to write out. `static` is
the warm-up, `shared` is hard in exactly one place — but that one place turns out to
be the most important memory-ordering lesson Part 5 *didn't* touch.

We'll write the first four vtable functions: `static_clone`, `static_drop`,
`share_clone`, `share_drop`. And we'll answer a question Part 5 left on the table: the
ordering in promotion was for *publishing* a `Shared` block; the ordering in
`share_drop` is for *freeing* a shared buffer — a completely different hazard, named
*free-while-read*.

## Map: read `ctx` → know the repr → run which function

The whole implementation track revolves around one move: each vtable function reads
`ctx`, infers which repr it's in, then branches. Anchor this in your head before diving
into code:

```
vtable = STATIC       ctx = null              clone: copy struct   · drop: no-op        (freed 0 times)

vtable = SHARE        ctx = *mut Shared       clone: +refcount     · drop: -refcount    (freed 1 time)

vtable = PROMOTABLE   ctx ODD  (KIND_VEC)     clone: promote_vec   · drop: free_boxed_slice
                      ctx EVEN (KIND_ARC)     clone/drop: go through Shared (like the SHARE row)

     the ONE state transition, one-way:
        PROMOTABLE/VEC ──(first clone: promote_vec, CAS)──► PROMOTABLE/ARC
```

Part 6 writes the first two rows (`STATIC`, `SHARE`). Part 7 handles how to *encode*
`ctx` for `PROMOTABLE` (the odd/even trick). Part 8 writes the state transition
(`promote_vec`) and the two `PROMOTABLE` functions. Remember: `vtable` is frozen at
birth; only the *KIND bit in `ctx`* changes when we promote — so "PROMOTABLE/ARC" still
uses the promotable vtable, it just branches into the Shared path.

## `static`: the warm-up

Recall: a `static` `Bytes` points into memory that lives forever (`&'static [u8]`), so
there's nothing to count and nothing to free. `data` is left null. Its two functions
are the two shortest answers in the whole series:

```rust
fn static_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    // No refcount. Clone just rebuilds a handle pointing at the same place.
    unsafe {
        Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8),
            len,
            ctx: AtomicPtr::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }
}

fn static_drop(_ctx: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    // Do nothing. The bytes live forever, there is nothing to free.
}
```

`static_drop` is empty *on purpose* — it is the very embodiment of the first of the
five take-away questions: *exactly how many times is this region freed?* For `static`,
the answer is **0**. An empty `drop` function isn't a not-yet-finished one; it is "0
times" written out in code. Notice that `ctx` is null here, so it must absolutely
never be dereferenced — and thankfully, not a single line dereferences it.

## `shared`: the `Shared` block and its three fields

`shared` is a hand-written `Arc<[u8]>`. We need a control block on the heap holding the
counter:

```rust
struct Shared {
    buf: *mut u8,          // the ORIGINAL address of the allocation — to hand back to the allocator later
    cap: usize,            // the allocation's size — together with buf forms the "how to free it"
    ref_count: AtomicUsize,
}
```

A detail Part 4 foreshadowed, now made concrete: `Shared.buf` is the *original*
address of the allocation, **not** the pointer the handle currently holds
(`Bytes.ptr`). For an uncut handle the two are equal; but after a `slice`, `Bytes.ptr`
points into the *middle* of the buffer, while `buf` must still be the start — because
you may only hand the allocator back exactly the pointer it gave you. That's why
`buf`/`cap` live inside `Shared`, separate from the handle's `ptr`/`len`. (Part 8 will
use exactly this property to make `slice` O(1).)

## `share_clone`: bump the counter, and why `Relaxed` is enough

```rust
fn share_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = ctx.load(Ordering::Relaxed) as *mut Shared;
    unsafe { shallow_clone_arc(shared, ptr, len) }
}

unsafe fn shallow_clone_arc(shared: *mut Shared, ptr: *const u8, len: usize) -> Bytes {
    let old = (*shared).ref_count.fetch_add(1, Ordering::Relaxed);
    if old > isize::MAX as usize / 2 {
        abort(); // see "Why abort and not panic" below
    }
    Bytes {
        ptr: NonNull::new_unchecked(ptr as *mut u8),
        len,
        ctx: AtomicPtr::new(shared as *mut ()),
        vtable: &SHARE_VTABLE,
    }
}
```

There are *two* atomic operations here, and both are `Relaxed`. This is where
newcomers get confused, so let's spell it out.

**The `load` of the `shared` pointer: `Relaxed`.** Remember the principle from Part 5
— an ordering doesn't protect *the atomic value itself*, it protects *the other memory
around* that operation. Here the `shared` pointer is a *stable* address: it's set when
the handle is born and doesn't change for the handle's whole life. We aren't using
this read as a *flag* saying some new memory was just published — we're just fetching
an address we *already own*. There's no happens-before edge to build, so `Relaxed` is
the honest minimum.

(A contrast to remember: the `data` read in `promotable_clone` in Part 5 had to be
`Acquire`, because there it *might* be a flag saying "just promoted, here's a new
`Shared`" — and we were going to go *read the contents* of that `Shared` block. Same
`load`, different ordering, because one is "fetch an address I already own" and the
other is "receive memory just published".)

**The `fetch_add` bumping the counter: `Relaxed`.** Bumping the refcount *publishes*
no memory to anyone. To be able to call `clone`, you're already holding a live handle
→ the payload and the `Shared` block already exist and are already visible to you. The
bump is just arithmetic on a counter; there's nothing to synchronize. So `Relaxed`.

**The overflow guard — and why `abort`, not `panic`.** Because `fetch_add` uses
`Relaxed` it's very cheap, and a pathological `mem::forget` loop (or a clone storm)
could *in theory* wrap the `usize` back around to a small number → free too early →
use-after-free. So we guard: if the counter passes a threshold, stop hard. Stop with
`abort`, not `panic`, because by then memory safety is already broken — and a `panic`
*can be caught by `catch_unwind`* and it *unwinds through `Drop`s*, and `Drop` touches
exactly the counter we can no longer trust. `abort` is an unconditional stop. (We
check the threshold with the *return value of `fetch_add`*, not a separate `load` — to
avoid a TOCTOU gap between "read" and "bump".)

## `share_drop`: the free-while-read hazard

This is the part worth the whole post. `share_drop` decrements the counter, and if
we're the last one, frees the buffer + the `Shared` block.

```rust
fn share_drop(ctx: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    let shared = ctx.load(Ordering::Relaxed) as *mut Shared;
    unsafe { release_shared(shared) }
}

unsafe fn release_shared(shared: *mut Shared) {
    if (*shared).ref_count.fetch_sub(1, Ordering::Release) != 1 {
        return; // not the last one yet — done
    }
    core::sync::atomic::fence(Ordering::Acquire);
    let cap = (*shared).cap;
    drop(Vec::from_raw_parts((*shared).buf, cap, cap)); // free the buffer from its ORIGINAL address
    drop(Box::from_raw(shared));                        // free the Shared block
}
```

Notice `release_shared` **doesn't need the handle's `ptr`/`len`** — it frees the whole
allocation from `Shared.buf`/`Shared.cap`. This is precisely what makes `slice` safe:
no matter how far the handle has been cut, drop always hands back the original pointer.
(It uses `cap` for *both* the length and the capacity of `Vec::from_raw_parts` — we're
describing the *allocation*, not the *view*. `u8` has no destructor, so the length only
affects "how many destructors run", but describing the allocation correctly is a habit
to keep: the day the buffer holds a type with `Drop`, mistakenly using the view's
`len` would run the wrong number of destructors.)

Now for the ordering, and why it's **completely different** from Part 5's ordering.

### The problem: freeing while another thread is still reading

Part 5 worried about the *publish-before-read* hazard: publishing the `Shared` block's
address before its contents have appeared. Here the hazard is the reverse: **freeing
the buffer while another thread is still reading it** — free-while-read.

Set the scene: `b1` and `b2` are two handles sharing the same buffer, on two different
threads. Thread A reads a few bytes then drops `b2`; thread B drops `b1`. The counter
goes `2 → 1 → 0`. Your sequential intuition says: "the counter hitting 0 means nobody
is using it anymore → safe to free". Correct — *if there were only one thread*. But
across threads, on hardware that reorders, **"the counter hit 0" and "every read has
finished" are NOT automatically the same moment.** The CPU/compiler is allowed to move
thread A's buffer read to *after* its own counter decrement.

Watch it break with *no* ordering (suppose both decrements are `Relaxed`):

```
Thread A                              Thread B
  fetch_sub → 2→1 (Relaxed)
  ...read b2[0] MOVED down here          fetch_sub → 1→0, sees 0
       │                                 free(buf)         ← the buffer vanishes
       └── read b2[0] RIGHT NOW ←────────────────────────── USE-AFTER-FREE
```

A's read got moved past its own decrement, so B sees the counter at 0 and frees *while*
A's read is still pending. It reads dead memory.

### The fix: `Release` on decrement, `Acquire` fence before freeing

- Every dropper decrements the counter with **`Release`** → "I publish: all my buffer
  accesses are *before* this decrement, they may not slide down past it."
- The last one (the `fetch_sub` that returns 1) runs an **`Acquire` fence** *before*
  freeing → "I subscribe: synchronize with *every* other thread's `Release`
  decrement, so all their buffer accesses now happen-before my free."

This `Release`/`Acquire` pair is exactly what *glues* "the counter hit 0" to "every
reader has truly finished". Without it, the counter is right but memory-visibility is
wrong.

A subtle detail makes *one* fence enough to synchronize with *all* the decrements:
each `fetch_sub` is a read-modify-write, so the last decrement reads a value that sits
in the *release sequence* headed by every prior `Release` decrement — that's what lets
a single `fence(Acquire)` pair with all of them.

### Why a separate `fence(Acquire)` instead of `fetch_sub(AcqRel)`?

You *could* make the decrement `AcqRel` and drop the fence — still correct. But
`AcqRel` forces `Acquire` onto *every* decrement, including the non-final ones (which
just return, freeing nothing). A separate fence makes **only the last one** pay the
price of an `Acquire` barrier; the others just do the cheaper `Release` decrement. This
is a performance matter, not a correctness one — and it's exactly why the real `Arc` is
written this way.

## Contrasting the two kinds of ordering in the series

This is the point to take away, because it separates two hazards people tend to lump
together:

| | Part 5 (promotion) | Part 6 (`share_drop`) |
|---|---|---|
| Hazard | publish-before-read: publish the pointer before the contents appear | free-while-read: free while another is still reading |
| Operation | CAS writing `data` = new `Shared` | `fetch_sub` on the counter |
| The "publish" side | CAS succeeds → `Release` | every decrement → `Release` |
| The "receive" side | the `load`/failed-CAS → `Acquire` | the last one's `fence(Acquire)` |

The same `Release`/`Acquire` pair, two different problems. The general principle still
holds: *whenever one of your atomic operations is used by another thread as the signal
to decide "now I'm allowed to touch (or free) the shared region", the memory accesses
around that operation must be ordered through a Release/Acquire pair.*

## What we have, and what Part 7 does

Four functions done: `static_*` (0 frees), `share_*` (`Relaxed` discipline on the
bump, `Release` + `fence(Acquire)` on the decrement). The core point: `share_drop`'s
ordering is not Part 5's ordering — it fights free-while-read, not publish-before-read.

But we still can't *create* a `shared` `Bytes`. Nothing calls into `SHARE_VTABLE` yet.
The missing piece is `from_vec` — turning a `Vec<u8>` into a `Bytes`. And right when we
write `from_vec`, we hit the thing Part 5 deliberately parked in an aside: how does
*one 8-byte slot* hold both a buffer pointer and a `Shared` pointer, and tell the two
apart? That's the bit-packing trick, and Part 7 dissects it all the way down.

---

*Next: [Part 7 — `from_vec` and the bit-packing trick](07_from_vec_and_bit_tagging.md) ·
[Index](00_index.md)*

*Tiếng Việt: [`../vi/06_static_and_shared.md`](../vi/06_static_and_shared.md)*
