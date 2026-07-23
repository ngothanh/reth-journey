# W005 Thu — 5-year failure mode: lists

**Trigger**: the first hot path that RLP-encodes a large or deeply nested list — W6 block
bodies, W22 staged-sync header batches.

**What breaks**: nothing correctness-wise. `[T]::encode` scratch-buffers the whole payload
into a `Vec` before writing the header, so encoding an N-element list of M-byte elements
allocates N·M bytes of scratch *per nesting level*, on top of the output buffer. A block
body with 300 transactions, each a nested list, pays this at every level of the tree.

**Migration**: adopt the derive's shape — sum the children's `length()` first, write the
header, then encode children straight into `out`. Zero scratch allocation, one extra pass
over the children to compute lengths. That trade is unambiguously right because `length()`
is arithmetic: the extra pass is arithmetic-only, while the current scratch is
allocate-and-memcpy.

**Trigger condition**: measure first. Add a criterion bench encoding a 300-element list of
20-byte items and compare against the arithmetic version before changing anything — the
whole point of `length()` being arithmetic is that the win should be visible, and if it is
not, the current code is simpler.

**Separately**: `Option<T>`, tuples and `[T; N]` remain unimplemented (see
`05_lists_design.md`). W7 `Authorization` is the first consumer that needs `Option<T>`, and
the `None`-encoding question — empty string vs empty list vs omit-from-parent — must be
answered against alloy's behaviour, not chosen freely, or derived structs will diverge on
the wire.
