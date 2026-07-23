# W005 Fri — `#[derive(RlpEncodable, RlpDecodable)]` design

## Problem

Hand-writing `Encodable`/`Decodable` for ~20 consensus structs (Header W6, AccessListItem
/ Withdrawal / Authorization W7, Receipt / Log W8, TxEnvelope W9) is rote and a bug magnet:
forget a field, swap two, miss the trailing-bytes check. A derive does it mechanically —
list header, fields in declaration order, arithmetic length — and mirroring
`alloy-rlp-derive`'s output means every derived struct matches alloy byte-for-byte.

## Requirements

- **R1** — `#[derive(RlpEncodable)]` on a named-field struct writes a list `Header` whose
  payload is the sum of the children's `length()`, then encodes each field in declaration
  order.
- **R2** — `#[derive(RlpDecodable)]` is the inverse: read the list header, decode each
  field, assert the cursor reached the payload end.
- **R3** — generated code uses absolute paths (`::eth_rlp::Encodable`, `::eth_rlp::Header`),
  never bare names.
- **R4** — generated `length()` is pure arithmetic. No scratch encode.
- **R5** — enums, unions, tuple structs, unit structs produce a `compile_error!` pinned to
  the derive site.
- **R6** — the generated impl carries a doc-comment flagging that field order is wire format.
- **R7** — trailing bytes are an error.

## Design walk-through

**D1 — the literal generated impl** for `struct Foo { a: u64, b: Bytes, c: B256 }`:

```rust
#[automatically_derived]
impl ::eth_rlp::Encodable for Foo {
    fn encode(&self, out: &mut dyn ::eth_rlp::BufMut) {
        let __payload_length = 0usize
            + ::eth_rlp::Encodable::length(&self.a)
            + ::eth_rlp::Encodable::length(&self.b)
            + ::eth_rlp::Encodable::length(&self.c);
        ::eth_rlp::Header { list: true, payload_length: __payload_length }.encode(out);
        ::eth_rlp::Encodable::encode(&self.a, out);
        ::eth_rlp::Encodable::encode(&self.b, out);
        ::eth_rlp::Encodable::encode(&self.c, out);
    }

    fn length(&self) -> usize {
        let __payload_length = 0usize + /* ...same sum... */;
        ::eth_rlp::Header { list: true, payload_length: __payload_length }.length()
            + __payload_length
    }
}
```

**D2 — payload length without re-encoding.** Children's lengths are unknown at expansion
time, so the generated code sums them at *runtime* via `Encodable::length`. This is why R4
matters recursively: if any leaf's `length()` scratch-encodes, every enclosing struct pays
for it once per nesting level. Contrast `impl Encodable for [T]` (`encode.rs:104`), which
still double-buffers into a `Vec` because it does not use `length()`.

**D3 — decode and the cursor target.** Fields are decoded from `&buf[..payload_length]`, a
*slice of the payload*, not from `buf` directly. Two reasons: a field whose own header
overruns the list cannot reach into sibling data (it runs out of input instead), and the
trailing-bytes check reduces to `!__payload.is_empty()`. Then `*buf = &buf[payload_length..]`
advances the caller's cursor exactly one frame. This mirrors `Vec<T>::decode`
(`decode.rs:57`), which already worked this way.

**D4 — hygiene.** One binding, `__payload_length`, plus `__header`/`__payload`/`__value` on
the decode side. Double-underscore-prefixed so they cannot collide with a field named
`payload_length`; field names are only ever used as `self.#name` or as struct-literal keys,
never as locals, so the collision surface is small but not zero.

**D5 — error message.** Literal string, pinned by trybuild:

> ``RlpEncodable cannot be derived on enums — RLP structs must be structs with named fields``

with the derive name and shape interpolated. `tests/compile_fail/*.stderr` holds all five.

**D6 — versus W4's `SimpleEncode`.** Reused: the `DeriveInput` parse, the shape rejection
arms, absolute-path discipline. Added: list `Header` emission, arithmetic length, the whole
decode side, the trailing-bytes check, generic support via `split_for_impl` plus an injected
`::eth_rlp::Encodable` bound on every type parameter.

**D7 — pre-mortem: a contributor reorders `{a, b, c}` to `{a, c, b}`.**
`derive_rlp_field_order_is_wire_format_fixture` catches it, because the fixture is a
**literal `&[u8]` pasted into the source**, not a runtime-generated value. A regenerated
fixture would move with the code and mask the break. `reordering_fields_changes_the_bytes`
backs it up by asserting the reordered encoding has the same *length* but different *bytes*
— proving the test is sensitive to order, not just to size.

## Deviation from the plan, recorded

R7 as originally written could not be satisfied by its own prescribed fix. See
`05_rlp_derive_prediction.md` P4 and the amended R7 in `plan/W005.md`: intra-frame trailing
bytes are the derive's job, post-frame bytes are `decode_exact`'s.

## Output

21 tests: 16 in `tests/rlp_derive.rs`, 5 in `tests/compile_fail.rs`. Plus 5 in
`tests/five_field.rs` covering the long-form list header.
