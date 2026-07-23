# W005 Mon/Tue — 5-year failure mode: `Bytes`

**Trigger**: the first 16-bit or otherwise unusually-aligned platform, or the first time
someone constructs `Shared` somewhere other than `Box::into_raw`.

**What breaks**: the odd/even discriminant on `ctx` assumes every `*mut Shared` is at least
2-byte aligned. That is guaranteed by `Box` for a struct containing a `usize` and an
`AtomicUsize`, but it is an *implicit* invariant — nothing in the type system states it,
and a future `#[repr(packed)]` on `Shared`, or an arena allocator handing out odd
addresses, would silently reinterpret a shared buffer as an owned capacity.

**Migration**: add a `debug_assert!(shared as usize & OWNED_TAG == 0)` at every site that
stores a `Shared` pointer into `ctx`, and a static assertion that
`align_of::<Shared>() >= 2`. Both are free in release builds and turn a silent
memory-corruption bug into a loud one.

**Second-order**: the `Relaxed` increment is correct today because frozen buffers are
immutable. If `Bytes` ever grows an interior-mutability path — a copy-on-write `make_mut`,
say — that ordering becomes unsound and every clone needs `Acquire`. Any PR adding mutation
to a shared buffer must revisit `shallow_clone_arc` explicitly; note it in the module doc so
the reviewer sees it.
