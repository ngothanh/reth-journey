# W005 Mon/Tue — 🧮 Paper drill: `Bytes` representation

## D-LAYOUT — the three states of one `Bytes`

`Bytes { ptr: NonNull<u8>, len: usize, ctx: AtomicPtr<()>, vtable: &'static Vtable }`

```
STATIC     ptr ──────────────► &'static [u8]  (in .rodata)
           ctx = null                          drop = no-op

OWNED      ptr ──────────────► heap buffer, base == ptr
           ctx = (cap << 1)|1  ← odd. capacity lives HERE, not in a control block.
                                 no Shared allocated yet.

SHARED     ptr ──────────────► anywhere inside the buffer (slicing allowed)
           ctx ──────────────► Shared { buf, cap, ref_count }   ← even (aligned)
```

**The discriminant is one bit of `ctx`.** Odd ⇒ owned, even ⇒ shared-or-null. This works
only because a heap-allocated `Shared` is pointer-aligned, so its low bit is guaranteed 0.

## D-WALK — `BytesMut::freeze()` then two clones

| Step | `ctx` | Allocations | Refcount |
|---|---|---|---|
| `BytesMut` with cap 64 | — | 1 (the buffer) | — |
| `.freeze()` | `(64 << 1) \| 1` = `129` | **0 new** | — |
| first `.clone()` | promoted → `*mut Shared` | 1 (the control block) | 2 |
| second `.clone()` | unchanged | 0 | 3 |
| all three dropped | — | both freed at 0 | 3→2→1→0 |

Count the allocations for a freeze that is never cloned: **one**, the buffer itself. Under
the W4 scheme it was two. On a WAL frame handoff at, say, 100k frames/sec, that is 100k
allocations/sec removed from the hot path.

## D-WALK 2 — the overflow guard

`fetch_add` returns the *old* value. The check is `old > isize::MAX / 2`, not
`usize::MAX`, because the guard must fire while there is still headroom — between the
check and the abort, other threads may also be incrementing. Halving `isize::MAX` leaves
2^62 slots of margin, which no real program can exhaust before the abort lands.
