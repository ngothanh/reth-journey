# Part 8 — full promotable, and O(1) `slice`

We have `from_vec` creating a `promotable` `Bytes` (Part 7), and we understand *why*
promotion exists (Part 4) along with the *concurrency tools* it needs (Part 5). This
final post assembles it all into code: the four `promotable_*` functions, the
`promote_vec` function with its CAS race, the O(1) `slice` function, and the quiet
invariant that props all of them up.

The pleasant part: after all that preparation, the four dispatch functions almost write
themselves. The difficulty all funnels into a single function — `promote_vec` — and a
single branch of it, the *losing* branch.

## The four `promotable_*` functions are just dispatch

Each function does exactly one thing: read `ctx`, check the KIND (by Part 7's sentence
"VEC odd, ARC even"), then branch. The ARC branch delegates to the `shared` helpers
written in Part 6; the VEC branch does its own Vec-specific work.

```rust
fn promotable_even_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let tagged = ctx.load(Ordering::Acquire); // Acquire: someone may have just promoted & published a Shared
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe { shallow_clone_arc(tagged as *mut Shared, ptr, len) } // already promoted → like share_clone
    } else {
        let buf = (tagged as usize & !KIND_MASK) as *mut u8;          // EVEN: mask off the bit
        unsafe { promote_vec(ctx, tagged, buf, ptr, len) }            // first clone → promote
    }
}
```

`promotable_odd_clone` is identical, except its VEC branch recovers without masking:
`let buf = tagged as *mut u8;`. The two drop functions are the same shape, only
swapping the two jobs: ARC → `release_shared` (decrement the counter), VEC →
`free_boxed_slice` (free the buffer directly, no atomics):

```rust
fn promotable_even_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {
    let tagged = *ctx.get_mut(); // &mut = exclusive → plain read, no atomics (recall Part 5)
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe { release_shared(tagged as *mut Shared) }
    } else {
        let buf = (tagged as usize & !KIND_MASK) as *mut u8;
        unsafe { free_boxed_slice(buf, ptr, len) }
    }
}
```

> **A deadly trap:** the easiest mistake is to *invert* the KIND condition. Cling to
> "VEC odd, ARC even": the `== KIND_ARC` branch is the one that goes the `Shared` way;
> the other one (VEC) is the one that promotes / frees the buffer. Writing it as
> `== KIND_VEC` for the `Shared` way casts the buffer to `*mut Shared` → silent UB.
> This is exactly the kind of bug `miri` was built to catch.

Notice the `load` in clone is `Acquire`, while in drop it's a plain read via `get_mut`
— exactly as Part 5 argued: clone shares a reference (can race), drop is exclusive
(can't race).

## `promote_vec`: allocate `Shared`, CAS, and handle the loser

This is the heart. It implements exactly the "mutate back into the original" from Part
4 and the CAS from Part 5.

```rust
unsafe fn promote_vec(
    ctx: &AtomicPtr<()>, tagged: *mut (), buf: *mut u8, ptr: *const u8, len: usize,
) -> Bytes {
    // 1. Recover the allocation size. See "why this arithmetic is safe" below.
    let cap = (ptr as usize - buf as usize) + len;

    // 2. Allocate the Shared block, ref_count = 2 (the original handle + the clone we're about to return).
    let shared = Box::into_raw(Box::new(Shared {
        buf, cap, ref_count: AtomicUsize::new(2),
    }));

    // 3. Publish it: swap ctx from `tagged` to `shared`.
    match ctx.compare_exchange(tagged, shared as *mut (), Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8), len,
            ctx: AtomicPtr::new(shared as *mut ()), vtable: &SHARE_VTABLE,
        },
        Err(actual) => {
            // Someone else promoted first. Throw away OUR Shared, latch onto the winner's.
            drop(Box::from_raw(shared));                          // free the control shell, do NOT free buf
            shallow_clone_arc(actual as *mut Shared, ptr, len)   // use `actual`, NOT `shared`
        }
    }
}
```

Three points to make.

**`ref_count = 2`, not 1.** The CAS publishes the `Shared` to *two* handles at once:
the original handle (`b1`, whose `ctx` we just CAS'd) and the clone we're returning.
Both now point at this `Shared`, so the counter starts at 2. Check it with Part 3's
counting way of thinking: two handles → two drops → down to 0 → freed once. Balanced.

**The `Ok` branch — the beauty of promotion.** The CAS writes into the `ctx` of the
*original handle `b1`* (we received `ctx: &AtomicPtr`, which is `&b1.ctx`). So `b1`
*becomes shared right in place*, even though `b1.vtable` is still `PROMOTABLE_*`
(unchangeable — Part 5). The next time `b1` clones/drops, the `promotable_*` function
reads `ctx`, sees KIND_ARC (bit 0), and takes the `Shared` branch on its own. The new
clone carries `SHARE_VTABLE` directly. Two "flavors" of arc-backed handle coexist,
counting one and the same counter.

**The `Err(actual)` branch — `actual` is DIFFERENT from `shared`.** This is what Part 4
called "you have to be careful throwing away the extra counter", and it's the classic
bug. `compare_exchange(expected, new)` means: "*if* `ctx` still equals `expected` then
change it to `new`, otherwise report the current value". On `Err(actual)`:

- `shared` = the `Shared` block **we ourselves** just allocated (say 0xBBB) — the loser,
  *useless*.
- `actual` = the value actually sitting in `ctx` = the **winner's** `Shared` block (say
  0xAAA) — a *completely different* address, because each thread `Box::new`s once → two
  heap regions.

So we must (a) throw away *our* `shared` — and throw it away *correctly*:
`Box::from_raw(shared)` frees only the *control shell*, it does **not** touch `buf`
(because `Shared` has no `Drop` impl; `buf` now belongs to the winner's `Shared`); then
(b) `shallow_clone_arc(actual)` to bump the winner's counter. Mistakenly using `shared`
(already freed) in step (b) is an immediate use-after-free, *and* abandons the real
`Shared` → counter skew → double-free.

Check the counter in a 3-thread race: the winner A creates the `Shared` with `ref=2`
(original + A); B and C lose, each does `shallow_clone_arc(actual)` +1 → up to `4`? No
— only one of B/C "loses first", but both do +1, making **4**... wait. Let's count
again properly: there's only *one* original handle and *one* winning promote (A). Each
thread's clone creates *one* new handle. 3 threads cloning → 3 new handles + 1 original
= 4 handles. A sets ref=2 (original + A's handle), B +1 = 3 (adds B's handle), C +1 = 4
(adds C's handle). Exactly 4 live handles → 4 drops → freed once. Balanced.

### Why the arithmetic `cap = (ptr - buf) + len` is safe

`promote_vec` isn't given `cap` — it recovers it by arithmetic. `(ptr - buf)` is the
distance from the bottom of the buffer to the start of the view; adding `len` gives the
distance to the *end* of the view. This equals the allocation size only **if the view
always reaches the end of the allocation** — i.e. the buffer has never been shortened
at the tail.

And it is, thanks to an invariant: **a VEC handle is never sliced.** Because `slice`
(next section) goes through `clone`, and cloning a VEC *promotes* it to ARC. So you
never hold a cut VEC — a VEC is always the intact buffer, `ptr == buf`, `cap == len`.
That's why `free_boxed_slice` also recovers `cap` by exactly this arithmetic, instead
of having to store `cap`:

```rust
unsafe fn free_boxed_slice(buf: *mut u8, ptr: *const u8, len: usize) {
    let cap = (ptr as usize - buf as usize) + len;
    drop(Vec::from_raw_parts(buf, cap, cap));
}
```

(By contrast, the `shared` repr *does* store `cap` in `Shared`, because *after*
promotion you're free to cut both ends, so `cap` can no longer be recovered from the
view. One recovers by arithmetic, one stores explicitly — that asymmetry is a direct
consequence of the invariant.)

## `slice`: O(1), and it *enforces* the invariant

The whole point of `Bytes` is cheap `slice`. The secret: **clone, then narrow the
view** — no copying.

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    // ... compute start, end, assert in-bounds ...
    if start == end {
        return Bytes::from_static(&[]); // empty → no need to hold a refcount
    }
    let mut sub = self.clone(); // share the backing (bump the counter / promote if currently VEC)
    sub.ptr = unsafe { NonNull::new_unchecked(sub.ptr.as_ptr().add(start)) };
    sub.len = end - start;
    sub
}
```

The nice thing is you write it *once* and it's correct for *all three* reprs, because
`clone` already handles the repr-specific part:

- **static**: trivial clone (no counter). Narrow into a `'static` slice → still static,
  drop still no-op. No allocation.
- **shared**: clone bumps the atomic counter. Narrow the view; `Shared.buf`/`cap` don't
  change so drop still frees from the bottom. *This* is why `Shared` stores `buf`/`cap`
  separately from the view.
- **promotable**: clone **promotes** it to shared, then narrows that.

That last point is the most beautiful: **slicing a promotable `Bytes` promotes it** —
exactly the "VEC is never sliced" invariant that both `promote_vec` and
`free_boxed_slice` rely on to recover `cap` by arithmetic. `slice` doesn't just *obey*
the invariant, it *enforces* it, by construction: the only path to cutting is through
clone, and clone promotes. A closed loop.

Two small safety points: `ptr.add(start)` is in-bounds because we asserted `start <=
end <= len`; and adding a small offset to a non-null pointer can't produce null, so
`new_unchecked` is still valid.

## Done. Looking back at the whole picture in code

Three reprs, four-plus functions, one invariant:

```
static     clone: copy struct         drop: no-op            (free 0 times)
shared     clone: fetch_add Relaxed    drop: fetch_sub Release + fence(Acquire)  (free 1 time)
promotable clone: not yet promoted → promote_vec (CAS);  already → shallow_clone_arc
           drop:  not yet promoted → free_boxed_slice;   already → release_shared

invariant:  slice ⇒ clone ⇒ (if VEC then promote) ⇒ VEC is never cut
            ⇒ VEC always ptr==buf, cap==len ⇒ recovering cap by arithmetic is safe
```

And the read path — `deref`, `len`, compare, hash — still touches only `ptr` + `len`,
never `ctx`/`vtable`, so it's as cheap as `Arc<[u8]>`. The whole
`ctx`/`vtable`/tag/CAS/ordering machine *only* comes into play on `clone` or `drop`.

## Verification: don't trust, measure

The bugs in this post — inverted KIND, `shared` vs `actual`, wrong ordering — *compile
cleanly* and often *seem to run correctly* on a single thread. They only surface under
a race, or when a tool peers into the memory model. So two things are mandatory:

- **`miri`**: `cargo +nightly miri test` — catches use-after-free, double-free, reads
  of uninitialized memory, and data races. Three of the four bugs above get caught by
  `miri` immediately.
- **A promotion race test**: have N threads all `clone` *one* original handle, forcing
  many `promote_vec` calls to run in parallel to poke the `Err(actual)` branch; run it
  repeatedly. `loom` (if you want to go further) will exhaustively sweep the possible
  reorderings.

Recall the third of Part 5's three closing lines: the scary bug in unsafe isn't the one
that crashes the program, it's the one that *runs correctly* — safe-Rust intuition is
inverted, the default for a mistake is silence. In promotable, that silence is
thickest. Always carry `miri`.

## End of the series

From "a byte comes in off the wire" (Part 1) to `promote_vec` with its losing branch
(this post), every piece was *forced* by the one before it: `Arc<[u8]>` can't do O(1)
`freeze` → lower ownership into a vtable → split reads from ownership → sole-owned clone
is a double-free → promotion mutates back → `AtomicPtr` solves three requirements → and
finally, coding it all up with a tagged pointer, CAS, and a self-enforcing invariant.
No piece fell from the sky.

Now you can not only *read* `bytes`, you can *rewrite* it — and argue for every line.

---

*Back to: [Part 7](07_from_vec_and_bit_tagging.md) · [Index](00_index.md)*

*Tiếng Việt: [`../vi/08_promotable_and_slice.md`](../vi/08_promotable_and_slice.md)*
