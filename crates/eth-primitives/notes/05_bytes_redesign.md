# W005 Mon/Tue — `Bytes` redesign: vtable + packed capacity

## Problem

W4's `Bytes` used a tagged *buffer pointer* (EVEN/ODD low bit) to distinguish
representations, which meant `BytesMut::freeze` had to allocate a control block eagerly
just to hand ownership across. On the RLP hot path — and later the W26 WAL frame handoff,
W37 skiplist node sharing, W38 SSTable block cache — that allocation is per-handoff, not
per-share.

## Design

Three static vtables, dispatched through `&'static Vtable` stored in the value:

| Vtable | `clone` | `drop` |
|---|---|---|
| `STATIC_VTABLE` | bitwise copy | no-op — `'static` bytes live forever |
| `OWNED_VTABLE` | promote to `Shared`, then share | free the buffer |
| `SHARE_VTABLE` | `fetch_add` the refcount | `fetch_sub`, free at zero |

The key move is what lives in `ctx: AtomicPtr<()>`:

- **odd** → OWNED. `ctx = (cap << 1) | 1`. The capacity is packed *into the word*, and the
  buffer base is `self.ptr` (owned is never sliced). No control block exists yet.
- **even** → promoted. `ctx` is an aligned `*mut Shared`, whose low bit is necessarily 0.

That is what makes `freeze` **zero-allocation**: a `BytesMut` becomes a `Bytes` by writing a
tagged word, not by allocating. The `Shared` control block is created lazily on the *first
clone* — pay for sharing only when sharing actually happens.

## Why the tag moved from the pointer to the context word

The old scheme tagged the buffer pointer, which forced masking on every deref and made
slicing hazardous (a slice's `ptr` is not the allocation base). Packing `cap` into `ctx`
instead leaves `ptr` a clean `NonNull<u8>` that can point anywhere inside the allocation,
which is what `slice()` needs, and moves the discriminant onto a word that is already
atomic for the promotion CAS.

## Safety notes

- `unsafe impl Send/Sync` are justified by the refcount being atomic and the buffer being
  immutable once frozen.
- `shallow_clone_arc` aborts if the refcount exceeds `isize::MAX / 2`, the standard
  `Arc` overflow guard — a leaked-clone loop must abort rather than wrap to zero and
  double-free.
- `Ordering::Relaxed` on `fetch_add` is sound because the increment only publishes the
  *existence* of another owner; the buffer contents are already immutable and were
  published by the release on the original store. The decrement side needs the
  acquire/release pair to order the final free after all readers.
- Equality and hashing are **by content**, via the deref'd slice — never by pointer
  identity — so two `Bytes` over equal bytes are interchangeable as `HashMap` keys.

## Output

`bytes.rs` + `bytes_mut.rs`, `benches/freeze.rs`, and the `bytes_no_alloc` test asserting
`freeze` performs no allocation.
