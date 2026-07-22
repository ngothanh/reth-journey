# Part 8 — When requirements grow: `advance`, lazy-promote, and the trilemma

Part 7 built the simplest design — cap-in-ctx — meeting exactly the *current*
requirements: zero-copy, zero-alloc `freeze`, O(1) `slice`, lazy-promote. But real
software rarely stands still. This post adds requirements *one at a time*, watches what
breaks, and lays out **every way to encode `ctx`** with the price of each. It closes with
an impossibility theorem: in a 4-word struct, you can't have everything.

## Requirement A: `advance` in place

**What `advance` is.** `bytes::Buf::advance(n)` — "swallow" the first `n` bytes by
*moving the view pointer* (`self.ptr += n`, `len -= n`) **in place**, on a
still-sole-owned handle, *without cloning*. This is the knife of a *consuming cursor*.

**When you need it.** Reading network frames by a running pointer; some streaming
decoders walk straight over an owned buffer. (Note: Ethereum RLP usually *doesn't* need
it — you walk a cursor over a borrowed `&[u8]`, you don't move an owned `Bytes`'s
pointer. That's why Part 7 fits this `Bytes`.)

**Why cap-in-ctx breaks.** After `advance(3)`, `self.ptr = buf + 3 ≠ buf`. But
`owned_drop` frees via `dealloc(self.ptr, cap)` = `dealloc(buf + 3, ...)` — freeing a
pointer in the *middle* of the allocation → **UB / heap corruption**. Root cause:
cap-in-ctx *assumes* `self.ptr == buf`, and `advance` breaks exactly that assumption.

With `advance`, `self.ptr` is no longer trustworthy as `buf`. We have to store `buf`
elsewhere. There are two paths, each with a price.

## Path 1 for `advance`: store `buf` in `ctx` → gives birth to EVEN/ODD

If `self.ptr` isn't trustworthy, stuff the *buffer pointer* into `ctx`. But now `cap` has
nowhere to live in `ctx` (the slot is busy holding a pointer). We recover `cap` by
**arithmetic**: `cap = (ptr - buf) + len` = the distance from the base to the *end* of
the view. Correct *only if* the view always reaches the end of the allocation — and
`advance` only trims the front (the view-end stays put), so the arithmetic holds... **on
condition that `cap == len` at creation.** Forcing `cap == len` = `into_boxed_slice`
(shrink the Vec) → **you lose zero-copy-from-Vec** (a realloc + memcpy if the Vec has
spare room).

Then comes pointer tagging — because `ctx` now holds a *pointer*, we need a bit to
distinguish OWNED from ARC. A `u8` buffer pointer (align 1) has *no* guaranteed free low
bit:

```
Even case (buf 0x1000): set the bit → ctx = 0x1001 → recover must CLEAR the bit → 0x1000
Odd case  (buf 0x1001): leave as-is → ctx = 0x1001 → recover must KEEP  it     → 0x1001
```

**The two cases have identical `ctx` (0x1001) but different `buf`** → tagging the low bit
is *lossy*. You need **1 spare bit** to store "was the original even or odd" — and the
*vtable pointer* is where that goes: **`EVEN`** ("mask on recovery") vs **`ODD`** ("keep
as-is"). This is the moment **the two EVEN/ODD vtables are born — as *the price of
storing a pointer*, i.e. the price of `advance`.**

This is exactly the real `bytes`'s "from Vec" path. **Tradeoff: you get `advance` + keep
lazy-promote, but lose zero-copy-from-Vec (shrink) + carry EVEN/ODD.**

## Path 2 for `advance`: refcount from the start

Store **both `buf` and `cap`** in a `Shared` block on the heap, with a refcount *from
birth*. `ctx` is *always* a `*mut Shared`. `self.ptr` is the view (advance freely),
`Shared.buf` is the base, `Shared.cap` is the size. Every operation goes through
`Shared`:

- `advance`: `self.ptr += n`. `slice`: clone (ref++) + narrow. Both simple.
- `freeze`: *reuses* the existing `Shared` → **0 alloc** — but only if the `Shared`
  *already existed before freeze* → **`BytesMut` must refcount from `new()`**.

**Tradeoff: you get `advance` + zero-alloc-freeze, but lose lazy-promote** — every heap
buffer pays for a `Shared` + an atomic *from birth*, even if it's never cloned.

## Requirement B: lazy-promote as a hard constraint

**What it is.** A sole-owned buffer that's never been cloned pays for **no** atomic and
allocates **no** `Shared`. **When it matters.** RLP decode mints *millions* of
single-use blobs; an atomic + an alloc *per blob* is the biggest avoidable cost on the
hot path. cap-in-ctx (Part 7) and EVEN/ODD *have* lazy-promote. Refcount-from-birth
*doesn't*.

## Every `ctx` encoding, side by side

| approach | not-yet-promoted `ctx` holds | `buf` from | `cap` from | `advance` | zero-copy freeze | lazy-promote | complexity |
|---|---|---|---|---|---|---|---|
| **cap-in-ctx** (Part 7) | `cap` | `self.ptr` | `ctx` | ❌ | ✅ | ✅ | 1 vtable |
| **buf-in-ctx EVEN/ODD** (`bytes`) | buf pointer (tagged) | `ctx` (mask) | arithmetic (`cap==len`) | ✅ | ❌ (shrink) | ✅ | 2 vtables |
| **refcount-from-birth** | *always* `*mut Shared` | `Shared` | `Shared` | ✅ | ✅¹ | ❌ | 2 reprs, simplest logic |

¹ zero-copy freeze needs `BytesMut` to refcount from birth.

## The trilemma: why "support everything" is impossible

Look at the three columns `advance` / zero-copy-freeze / lazy-promote: **no row gets all
three.** This isn't an implementation limit — it's a theorem:

> In a 4-word struct, you get only **2 of 3** {lazy-promote, `advance`, zero-alloc-freeze
> with `cap>len`}.

The concrete proof: `advance` moves the view off the base → you *must* store `buf`.
freeze-with-`cap>len` → you *must* store the true `cap`. Those are **two independent
values**, and the `ctx` slot holds only *one*. Keeping both → you need a `Shared` block
on the heap → for freeze to *not* allocate, `Shared` must exist *before* freeze →
`BytesMut` refcounts from birth → **you lose lazy-promote.**

The whole trilemma boils down to **one question**: *does the view move off the buffer's
base while **not yet** promoted (i.e. is there an `advance`)?*
- **Yes** → you must store `buf` → a pointer in `ctx` → EVEN/ODD, and `cap` must be
  arithmetic (lose zero-copy-from-Vec) *or* refcount (lose lazy-promote).
- **No** → `ctx` is free → pack `cap` → one vtable, keep both lazy-promote and
  zero-alloc-freeze.

## Conclusion: the "right" design = *your* requirements

There is no absolute best. Pick the point that fits your real requirements:

- **`Bytes` for Ethereum/RLP** (this series): slice + clone + freeze, *no* advance on an
  owned handle → **cap-in-ctx** (Part 7). Keep lazy-promote (cheap hot path) +
  zero-alloc-freeze, trading away the `advance` this kind never uses. This is the right
  choice.
- **`bytes` as a `Buf`** (networking): needs `advance` → **EVEN/ODD** (accept
  shrink-from-Vec) + `BytesMut` refcount-from-birth for zero-copy freeze. *That's* why
  the real `bytes` is more complex — it pays for a wider feature set.
- **Most general / easiest to reason about**: **refcount everything** (drop lazy-promote)
  — two reprs STATIC + SHARED, no tag, no promotion.

The takeaway: "rewriting `bytes`" is *not* copying it line by line. It's understanding
the whole **design space** and picking the right point for your requirements — then being
able to argue why. `bytes` picks EVEN/ODD because it's a `Buf`; we pick cap-in-ctx
because this `Bytes` slices, it doesn't advance. Both are *right* — for their own
problem.

## Verification, and the end of the series

Bugs in all three designs — inverted KIND branch, `shared` vs `actual`, wrong ordering,
dealloc with the wrong `cap`/`buf` — *compile cleanly* and *seem to run correctly* on one
thread. Mandatory: **`miri`** (`cargo +nightly miri test`, add `-Zmiri-strict-provenance`
for cap-in-ctx), and **a promotion race test** (N threads all `clone` one handle → poke
the `Err(actual)` branch; `loom` to exhaustively sweep the interleavings).

From "a byte comes in off the wire" (Part 1) to the trilemma (this post), every piece was
*forced* by the one before it, and the final piece shows: even "how to encode an 8-byte
slot" has no absolute answer — only *named* trade-offs, chosen by requirements. Now you
can not only *read* `bytes`, you can *redesign* it at any point on the trade-off space,
and argue for your choice.

---

*Back to: [Part 7](07_from_vec_and_bit_tagging.md) · [Index](00_index.md)*

*Tiếng Việt: [`../vi/08_promotable_and_slice.md`](../vi/08_promotable_and_slice.md)*
