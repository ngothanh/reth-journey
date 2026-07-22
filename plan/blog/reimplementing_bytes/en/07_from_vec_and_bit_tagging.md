# Part 7 — The simplest design: zero-copy, zero-alloc `freeze`

Part 6 gave us two reprs — `static` and `shared` — but nothing yet that *creates* a
`Bytes` owning a heap buffer, and the headline requirement of the whole series is still
unmet: **`freeze` must be O(1) — zero-copy, zero-allocation.** This post builds the
**simplest design** that meets that requirement, and *only* that requirement.

This is a deliberate choice: we do *not* build ahead for needs that don't exist yet
(in-place advance, advanced lazy-promote tricks). We start from the minimum that works.
Part 8 is the one that asks "what if we need more?" — and shows that each extra need
*forces* a trade-off.

## The one-owner problem

A `Bytes` fresh out of `from_vec` or `BytesMut::freeze` **owns a buffer, all by
itself**. It has to be able to do two things:

- **drop** → release the buffer. `dealloc` needs the *allocation base* + the *`cap`* (to
  rebuild the exact `Layout::array::<u8>(cap)`).
- **clone** → promote to shared (Part 4): allocate a `Shared` with a refcount.

Both of those need information, and we have only *one* slot to stash it in: `ctx`. And
`ctx` has to be distinguishable from a `Shared` pointer (the already-promoted state). So
what do we put into `ctx`?

## The key simplification: `self.ptr` is already the buffer's base

This is where everything collapses. For an owning handle whose *view never moves off the
base*, **`self.ptr` is exactly the buffer's base (`buf`)**. So `ctx` **doesn't need** to
hold a pointer — it holds the one thing `drop` *can't* recover from `ptr`/`len`:
**`cap`**.

(The "view never moves off the base" condition holds because the only way to move `ptr`
is `slice`, and `slice` will *promote* — see the end of this post. So an OWNED handle
*always* has `self.ptr == buf`. This is the foundational invariant of the whole design.)

## Encoding: `cap` in `ctx`

```rust
const OWNED_TAG: usize = 1;
//   ctx ODD  (bit 0 = 1)  → OWNED: ctx = (cap << 1) | 1;  buf = self.ptr
//   ctx EVEN (bit 0 = 0)  → ARC:   ctx = *mut Shared  (Shared aligned ≥ 8 → always even)
```

One low bit distinguishes the two states. A `Shared` on the heap is always even
(aligned), so we *force* OWNED to always be odd with `(cap << 1) | 1` — `cap` is a
number we control ourselves, shift it left then set the bit and you're done. **A single
`OWNED_VTABLE`.** (No "even/odd buffer", no EVEN/ODD — that's Part 8's story, once we're
forced to store a *pointer* instead of a *cap*.)

## `from_vec` and `from_owned_parts`

```rust
pub fn from_vec(bytes: Vec<u8>) -> Self {
    if bytes.is_empty() {
        return Self::from_static(&[]); // empty → static, 0 allocations (empty Vec drops normally)
    }
    // Keep the Vec's cap AS-IS — NO into_boxed_slice, NO realloc.
    let mut bytes = core::mem::ManuallyDrop::new(bytes);
    let (buf, len, cap) = (bytes.as_mut_ptr(), bytes.len(), bytes.capacity());
    unsafe { Self::from_owned_parts(NonNull::new_unchecked(buf), len, cap) }
}

pub(crate) unsafe fn from_owned_parts(ptr: NonNull<u8>, len: usize, cap: usize) -> Self {
    if cap == 0 { return Bytes::from_static(&[]); } // e.g. BytesMut::new(0) → dangling ptr
    Bytes {
        ptr, len,                                    // self.ptr = buf
        // cap packed into ctx as a provenance-free address (we only ever read .addr() back, never deref)
        ctx: AtomicPtr::new(ptr::without_provenance_mut((cap << 1) | OWNED_TAG)),
        vtable: &OWNED_VTABLE,
    }
}
```

Two points are the whole beauty of this design:

- **No `into_boxed_slice`.** That would shrink the Vec to `cap == len` (a realloc +
  memcpy if the Vec has spare room). We *don't* — we keep the buffer as-is, `cap` may be
  > `len`. Thanks to that, `BytesMut::freeze` on a `cap 1024 / len 7` buffer is
  **zero-copy** (the pointer doesn't change) *and* `from_owned_parts` **allocates
  nothing** (not even a control block) → **zero allocation**. This is the headline
  requirement, met.
- **`without_provenance_mut` + `.addr()`**: we store an *integer* in the `AtomicPtr`
  slot. Because we never deref it as a pointer, this is the correct strict-provenance
  API — clean under Miri `-Zmiri-strict-provenance`.

## `owned_clone` / `owned_drop` — just dispatch

```rust
fn owned_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let raw = ctx.load(Ordering::Acquire); // Acquire: someone may have just promoted & published a Shared
    if raw.addr() & OWNED_TAG == 0 {
        unsafe { shallow_clone_arc(raw as *mut Shared, ptr, len) } // already promoted → like share_clone
    } else {
        let cap = raw.addr() >> 1;                                 // cap READ STRAIGHT, no arithmetic
        unsafe { promote_owned(ctx, raw, ptr, cap, len) }          // first clone → promote
    }
}

fn owned_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, _len: usize) {
    let raw = *ctx.get_mut(); // &mut = exclusive → plain read, no atomics (Part 5)
    if raw.addr() & OWNED_TAG == 0 {
        unsafe { release_shared(raw as *mut Shared) }
    } else {
        let cap = raw.addr() >> 1;
        unsafe { dealloc(ptr as *mut u8, Layout::array::<u8>(cap).unwrap()) } // buf = self.ptr
    }
}
```

`buf` is `self.ptr` (no masking), `cap` is `ctx.addr() >> 1` (read straight). Compared
with Part 8's EVEN/ODD — mask the pointer + recover `cap` by arithmetic — this is far
tighter.

> **Trap:** the KIND branch is inverted. Cling to: `ctx` **even = ARC**, `ctx` **odd =
> OWNED**. Get it wrong and you cast a cap-number to `*mut Shared` then deref it → silent
> UB. `miri` catches exactly this kind.

## `promote_owned` — allocate `Shared`, CAS, handle the loser

The heart of the post: implementing "mutate back into the original" (Part 4) + the CAS
(Part 5).

```rust
unsafe fn promote_owned(
    ctx: &AtomicPtr<()>, tagged: *mut (), ptr: *const u8, cap: usize, len: usize,
) -> Bytes {
    let shared = Box::into_raw(Box::new(Shared {
        buf: ptr as *mut u8, cap, ref_count: AtomicUsize::new(2), // original handle + the clone
    }));
    match ctx.compare_exchange(tagged, shared as *mut (), Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8), len,
            ctx: AtomicPtr::new(shared as *mut ()), vtable: &SHARE_VTABLE,
        },
        Err(actual) => {
            drop(Box::from_raw(shared));                        // free the shell, do NOT free buf
            shallow_clone_arc(actual as *mut Shared, ptr, len)  // use `actual`, NOT `shared`
        }
    }
}
```

- **`ref_count = 2`**: the CAS publishes the `Shared` to *two* handles — the original
  `b1` (whose `ctx` we just CAS'd) + the clone we return. Two drops → down to 0 → freed
  once. Balanced.
- **`Ok` — the beauty**: the CAS writes into the *original handle's* `ctx`, so `b1`
  becomes shared *in place*, even though `b1.vtable` is still `OWNED_VTABLE`; next time it
  reads `ctx` it sees the even bit → takes the Shared branch on its own.
- **`Err(actual)` — the classic bug**: `actual` is the **winner's** `Shared` (different
  from our own `shared`, because each thread `Box::new`s its own heap region). We must
  throw away our own `shared` (`Box::from_raw` frees only the *shell*, it doesn't touch
  `buf` because `Shared` has no `Drop`) then latch onto `actual`. Mistakenly using
  `shared` (already freed) is an immediate use-after-free.

## `slice` — O(1), and it *enforces* the invariant

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    // ... compute start, end, assert in-bounds ...
    if start == end { return Bytes::from_static(&[]); }
    let mut sub = self.clone();  // share the backing (bump the counter / promote if currently OWNED)
    sub.ptr = unsafe { NonNull::new_unchecked(sub.ptr.as_ptr().add(start)) };
    sub.len = end - start;
    sub
}
```

Written *once*, correct for all three reprs because `clone` already handles the
repr-specific part. The crux: **slicing an OWNED `Bytes` clones it → cloning an OWNED
*promotes* it to SHARED.** So a slice result is always SHARED (it uses `Shared.buf` as
the base, cuts freely), while the original OWNED handle *never* has its `ptr` moved. That
is how the `self.ptr == buf` invariant is *enforced structurally*: the only path to move
`ptr` is `slice`, and `slice` promotes. Thanks to that, `owned_drop`'s
`dealloc(self.ptr, cap)` always hits the base.

## Done with the simplest design

We have a complete, correct `Bytes` that **meets the headline requirement**: zero-copy +
zero-alloc `freeze`, O(1) `slice`, lazy-promote `clone`, reads as cheap as `Arc<[u8]>`.
Clean under Miri `-Zmiri-strict-provenance`, and a `freeze` test asserts 0 alloc / 0
dealloc.

```
static  ctx = null                 clone: copy      drop: no-op                (free 0)
shared  ctx = *mut Shared          clone: +refcount drop: -refcount+fence      (free 1)
OWNED   ctx = (cap<<1|1) OR Shared;  buf = self.ptr;  clone: promote/arc  drop: dealloc/arc
```

**But** — this is the design for the *current* requirements. Real life breeds new ones:
*in-place advance* (when, tradeoff, how) and *lazy-promote as a hard
constraint*. Part 8 dissects each: every new requirement *forces* a different encoding,
dragging in EVEN/ODD or refcount-from-birth — and finally the **trilemma** that shows why
"support everything" is impossible in a 4-word struct.

---

*Next: [Part 8 — When requirements grow: advance, lazy-promote, and the trilemma](08_promotable_and_slice.md) ·
[Index](00_index.md)*

*Tiếng Việt: [`../vi/07_from_vec_and_bit_tagging.md`](../vi/07_from_vec_and_bit_tagging.md)*
