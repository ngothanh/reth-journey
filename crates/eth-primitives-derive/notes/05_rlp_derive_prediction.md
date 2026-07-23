# W005 Fri — RLP derive pre-predictions

> **Provenance**: reconstructed *after* the build, not sealed before it. Treat the
> "predicted" column as unverified; the "actual" column is what the compiler and tests
> said. Flagged rather than silently presented as a sealed prediction — a fabricated
> prediction record is worse than none, because the whole mechanism depends on the gap
> between the two being real.

## P1 — the codegen fragment reached for from memory

**Predicted**: `quote!( ::eth_rlp::encode_header(self.len(), false, out); out.put_slice(self); )`
with a `use bytes::BufMut;` inside the impl block.

**Actual**: that is verbatim what the first attempt contained, and it was wrong three ways:
the crate path was written `::eth-rlp::` (hyphens are illegal in paths — the crate is
`eth_rlp`), `use` is not allowed inside an `impl` block, and the body encoded the struct as
a byte **string** rather than a list. The `::eth_rlp::` prefix itself was *not* forgotten —
W4's SimpleEncode lesson held.

## P2 — expansion size for a 3-field struct

**Predicted**: ~15 lines.

**Actual**: 21 lines for `impl Encodable` (with the R6 doc-comment), 24 for
`impl Decodable`. The decode side is larger because of the header check, the payload-window
slice, the trailing-bytes check, and the cursor advance — none of which have an encode-side
counterpart.

## P3 — first-attempt output on `#[derive(RlpEncodable)] enum E { A, B }`

**Predicted**: a clean `compile_error!`.

**Actual**: correct. The `Data::Enum` / `Data::Union` / `Fields::Unnamed` / `Fields::Unit`
arms were present from the start (carried over from SimpleEncode), so the message was ours,
not rustc fallout. `tests/compile_fail/` now pins all five messages.

## P4 — does generated decode reject one trailing byte?

**Predicted**: yes.

**Actual**: **it depends on where the byte is, and the plan's own prescription could not
make its own test fail.** A byte appended *after* a complete frame is left in the buffer by
design — that is what lets a derived struct be a field of another. The derive catches only
trailing bytes *inside* the declared payload. Strictness at the outermost layer needed a
separate `decode_exact`. This was the week's sharpest finding and it amended R7.

## P5 — Phase-2 tests predicted to fail first attempt

**Predicted**: the alloy-parity test (byte-order or header-form mismatch).

**Actual**: parity passed first run. The failures were structural instead — no list header
at all in the first body, and the `Map` iterator being consumed by the first `#(...)*`
repetition so it could not be interpolated twice.
