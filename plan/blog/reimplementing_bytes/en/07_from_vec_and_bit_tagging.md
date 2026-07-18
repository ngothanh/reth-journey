# Part 7 — `from_vec` and the bit-packing trick: one 8-byte slot, two meanings

Part 6 finished `share_*` but there was no way to *create* a `Bytes` that goes down
that path. The entry point is `from_vec` — it takes a `Vec<u8>` *without* copying. And
right when we write `from_vec`, we hit the thing Part 5 parked in an aside: the `data`
of a `promotable` `Bytes` has to hold *two different kinds* of pointer — a buffer
pointer (not yet promoted) *or* a `Shared` pointer (already promoted) — in the same 8
bytes, and every later function has to be able to tell which kind it holds.

This post dissects exactly that trick, to the bottom. It's the fiddliest corner of the
whole design, so we go very slowly, and at the end give one sentence to remember that
makes it all collapse into place.

## `from_vec`: normalize, then park it

```rust
pub fn from_vec(bytes: Vec<u8>) -> Self {
    if bytes.is_empty() {
        return Self::from_static(&[]); // empty → go straight to the static repr, no allocation
    }
    let boxed: Box<[u8]> = bytes.into_boxed_slice(); // normalize: cap == len
    let len = boxed.len();
    let buf = Box::into_raw(boxed) as *mut u8;       // TAKE ownership — now we're responsible for freeing
    // ... bit-pack, then build the Bytes (below) ...
}
```

Three things, each with a reason:

**`is_empty` → `from_static(&[])`.** `into_boxed_slice` on an empty `Vec` gives a
*dangling* pointer that we don't want to bit-pack or free. Sending empty straight to
the `static` repr (an empty live-forever buffer) is the cleanest — no allocation for 0
bytes.

**`into_boxed_slice()` — normalize `cap == len`.** This is the pivotal detail we will
later *rely on to avoid having to store `cap`*. A `Vec` can have `cap > len` (spare
room); `into_boxed_slice` shrinks it to `cap == len`. The cost: if the `Vec` has spare
room, this operation *reallocates and memcpies*. True, and the real `bytes` does
exactly the same — so just remember that realloc can happen for a `Vec` with leftover
capacity.

**`Box::into_raw` — take ownership.** Before this line, the `Box` would free the buffer
on its own when it leaves scope. After `into_raw`, the `Box` is gone and *nothing*
frees it automatically anymore — **you** have signed up for the free (later via
`free_boxed_slice`/`release_shared`). `buf` is now the heap address of the first byte.
Dropping `buf` on the floor here is a leak.

## The problem: one slot, two meanings

A `promotable` `Bytes` needs `data` (we call this field `ctx`) to hold:

- when **not yet** promoted: a pointer to the raw **buffer**,
- when **already** promoted: a pointer to the **`Shared`** block.

And Part 5 already concluded why this discriminating label *has* to live in `ctx`:
promotion changes the state *mid-lifetime* via a one-word CAS on `ctx` — but `vtable`
freezes at birth, it can't be CAS'd along in the same shot. So we need a way, reading
*`ctx` alone*, to know which kind it currently is.

The way: borrow the **lowest bit** of the pointer as a KIND flag.

```rust
const KIND_ARC: usize = 0b0; // low bit = 0 → ctx is a *mut Shared
const KIND_VEC: usize = 0b1; // low bit = 1 → ctx is a buffer pointer
const KIND_MASK: usize = 0b1;
```

Why is the low bit *free space to borrow*? Because of **alignment**. A value of type
`T` with alignment `A` always sits at an address divisible by `A` — a multiple of 8 in
binary always ends in `000`. The `Shared` block holds a pointer + a `usize` + an
`AtomicUsize`, so its alignment is ≥ 8 → its address **always ends in bit 0**. So
`Shared` is *naturally* `KIND_ARC`, for free.

## The sentence to remember: **VEC is always ODD, ARC is always EVEN**

Everything follows from that one line. When any function **looks at `ctx`** to decode
the state:

- **`ctx` odd (bit = 1) → VEC** (still a buffer, not yet promoted),
- **`ctx` even (bit = 0) → ARC** (already a `Shared`).

- *ARC is always even*: `Shared` is 8-aligned → naturally bit 0. Free.
- *VEC must be odd*: so it **doesn't collide with ARC**. If an even buffer pointer were
  stored straight into `ctx`, a later `clone`/`drop` would see bit 0 → think "already
  promoted, this is a `Shared`" → cast the buffer to `*mut Shared` and read
  `ref_count`... i.e. read some of your data bytes as if they were the counter →
  disaster. So we *force* the VEC state to always read out odd.

## The wrinkle: a `u8` buffer can be even *or* odd

This is what makes this post different from a textbook tagged pointer. The buffer is
`u8`, **alignment = 1**, so its address is **not** guaranteed to have bit 0 = 0 — it
can be even or odd. But we *want* it to always read out odd (KIND_VEC). So:

- **even buffer** (bit 0): we must *set* the bit (`buf | 1`) to mark VEC. To recover
  the real address, we must *clear* that bit (`& !1`). → use
  **`PROMOTABLE_EVEN_VTABLE`**.
- **odd buffer** (bit 1 already): it already reads out VEC, so we do NOT set anything.
  But this bit 1 *is a real part of the address*, so when recovering we absolutely must
  NOT clear it. Store it as-is. → use **`PROMOTABLE_ODD_VTABLE`**.

The rest of `from_vec`'s code is exactly that branch:

```rust
    if buf as usize & KIND_MASK == 0 {
        // EVEN: set the bit to mark VEC; recover later by MASKing the bit off.
        let ctx = (buf as usize | KIND_VEC) as *mut ();
        Bytes { ptr: NonNull::new_unchecked(buf), len,
                ctx: AtomicPtr::new(ctx), vtable: &PROMOTABLE_EVEN_VTABLE }
    } else {
        // ODD: low bit is already 1 == VEC; store the pointer VERBATIM, do NOT mask later.
        Bytes { ptr: NonNull::new_unchecked(buf), len,
                ctx: AtomicPtr::new(buf as *mut ()), vtable: &PROMOTABLE_ODD_VTABLE }
    }
```

Notice `ptr` stores the **clean pointer** (`buf`, not bit-packed); only `ctx` carries
the tag. That way `deref` (reading through `ptr`) never sees the bit — the read path
always uses the real address.

## Why *two* vtables, not one?

This is the best question, and the answer touches a truth about information. Imagine
you had only one vtable and only `ctx`:

```
Case 1 (even buffer 0x1000): set bit → ctx = 0x1001 → recover needs to clear the bit → 0x1000
Case 2 (odd buffer  0x1001): leave it → ctx = 0x1001 → recover must keep the bit    → 0x1001
```

**The two cases have identical `ctx` (0x1001), but different real buffer addresses
(0x1000 vs 0x1001).** Looking at `ctx` alone, you *cannot* tell which the real buf is —
1 bit of information is gone. Packing the tag into the low bit is **lossy for odd
addresses**.

So you need **1 spare bit** stored somewhere to remember "was the original buffer even
or odd" — i.e. "does recovery mask or not". And **the vtable pointer is exactly where
that bit goes**, for free, because you're already carrying it. `EVEN` = "recovery
masks", `ODD` = "recovery keeps as-is". One vtable + `ctx` alone is *missing
information*, full stop.

(It's not 4 different branches: the two ARC cases — whether EVEN or ODD — are
*identical*, both read `ctx` straight into a `*mut Shared` without masking, because
`Shared` is always bit 0. EVEN/ODD differ *only* in the VEC branch.)

The full table, at two different moments — *encode* (`from_vec` looking at `buf`) and
*decode* (`clone`/`drop` looking at `ctx`):

| original buffer | encode (looking at `buf`) | decode (looking at `ctx`) | vtable |
|---|---|---|---|
| even | set bit `\| 1` | `ctx` even → **ARC**, `ctx` odd → **VEC (mask to recover)** | EVEN |
| odd | leave as-is | `ctx` even → **ARC**, `ctx` odd → **VEC (keep as-is)** | ODD |

## A practical note: `ODD` almost never runs

In practice, the system allocator *over-aligns* — Rust's `malloc`/allocator typically
returns pointers aligned to ≥ 16 even for a `u8` buffer (which only needs alignment 1).
So `buf` is almost always even, and `PROMOTABLE_ODD_VTABLE` is nearly dead code on a
normal allocator. But `u8`'s alignment *doesn't guarantee* even (a custom allocator, an
arena, or a sub-allocation could return an odd address), so the ODD branch exists
purely as a *correctness safety net*. To actually run through `promotable_odd_*`, you'd
have to deliberately build a `Bytes` on an odd-address buffer — the normal `from_vec`
path might never reach it.

## An escape hatch: if bit-packing feels like overkill

The "fiddly" feeling is *correct*. And it points at something: bit-packing is a tool
for *generality*, not a requirement for a minimal `Bytes`. The real `bytes` stuffs
`buf` into `ctx` because it supports `advance`/`split` — operations that move `ptr`
away from `buf` *without* promoting, so it's forced to remember the original `buf`
somewhere else → hence the tag + EVEN/ODD.

But if your `Bytes` has the invariant "VEC is never sliced" (Part 8 builds it — `slice`
always promotes), then a VEC handle *always* has `ptr == buf` and `cap == len`. Meaning
`buf`/`cap` already live inside `ptr`/`len` — stuffing them into `ctx` again is
*redundant*. In that case you can collapse to **a single vtable**, distinguishing
VEC/ARC by null:

```
ctx == null  → VEC (get buf from self.ptr, cap from self.len)
ctx != null  → ARC (ctx is a *mut Shared)
```

`null` never collides with a `Shared` pointer, so it's an absolutely safe sentinel, and
the whole EVEN/ODD apparatus vanishes. This is a real design decision: keep the tag to
mirror `bytes` 1:1 and be ready for `advance` later, or drop the tag to stay compact
for the current feature set. Both are correct — knowing what you're paying for is what
matters.

## What we have, and what Part 8 does

`from_vec` done: normalize to a boxed slice (`cap == len`), take on the free
responsibility, then bit-pack by even/odd to pick `EVEN`/`ODD`. The take-away
sentence: **VEC odd, ARC even** — and the buffer's even/odd-ness *only* decides how to
*recover* (mask or not), remembered by which vtable is chosen.

We can now create a `promotable` `Bytes`, but the four `promotable_*` functions are
still empty, and the first `clone` — the *promotion* that Parts 4 and 5 built the whole
model to explain — is still unwritten. Part 8 finishes it: the four dispatch functions,
the CAS race with its losing branch, and O(1) `slice` — which both uses promotion and
*enforces* the "VEC is never sliced" invariant promised above.

---

*Next: [Part 8 — full promotable and `slice`](08_promotable_and_slice.md) ·
[Index](00_index.md)*

*Tiếng Việt: [`../vi/07_from_vec_and_bit_tagging.md`](../vi/07_from_vec_and_bit_tagging.md)*
