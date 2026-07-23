# W005 Fri — 🧮 Paper drill: RLP derive

## D-MACRO — expansion, token by token

Input:

```rust
#[derive(RlpEncodable, RlpDecodable)]
struct Foo { a: u64, b: Bytes, c: B256 }
```

Encode half — see `05_rlp_derive_design.md` D1 for the full literal. The shape is:
sum three `::eth_rlp::Encodable::length(&self.X)` calls into `__payload_length`, emit
`::eth_rlp::Header { list: true, payload_length: __payload_length }`, then three
`::eth_rlp::Encodable::encode(&self.X, out)` calls in declaration order.

Decode half:

```rust
fn decode(buf: &mut &[u8]) -> ::core::result::Result<Self, ::eth_rlp::Error> {
    let __header = ::eth_rlp::Header::decode(buf)?;
    if !__header.list { return Err(::eth_rlp::Error::UnexpectedString); }
    let __payload_length = __header.payload_length;
    if buf.len() < __payload_length { return Err(::eth_rlp::Error::InputTooShort); }
    let mut __payload = &buf[..__payload_length];
    let __value = Foo {
        a: <u64   as ::eth_rlp::Decodable>::decode(&mut __payload)?,
        b: <Bytes as ::eth_rlp::Decodable>::decode(&mut __payload)?,
        c: <B256  as ::eth_rlp::Decodable>::decode(&mut __payload)?,
    };
    if !__payload.is_empty() { return Err(::eth_rlp::Error::TrailingBytes); }
    *buf = &buf[__payload_length..];
    Ok(__value)
}
```

For `enum E { A, B }` the expansion is a single item:

```rust
compile_error!("RlpEncodable cannot be derived on enums — RLP structs must be structs with named fields");
```

## D-WALK — hand-encoding the golden fixture

`Foo { a: 1, b: Bytes::from(&[0x42][..]), c: B256::ZERO }`

| Field | Value | Rule | Bytes | Len |
|---|---|---|---|---|
| `a` | `1u64` | minimal BE = `[0x01]`; single byte `< 0x80` → no header | `01` | 1 |
| `b` | `[0x42]` | single byte `< 0x80` → no header | `42` | 1 |
| `c` | 32 zero bytes | fixed-width string, **never** zero-trimmed → `0x80 + 32` | `a0 00×32` | 33 |

Payload = 1 + 1 + 33 = **35**. 35 ≤ 55 → short-form list header = `0xc0 + 35` = **`0xe3`**.

```
e3 01 42 a0 00 00 … 00        (36 bytes total)
```

The `c` row is the trap: `U256::ZERO` encodes as `0x80` (the empty string, because integers
are minimal big-endian), but `B256::ZERO` is 33 bytes. Same 32 bytes of storage, opposite
rule — which is why they live in separate impls.

**Field swap `{a, c, b}`:**

```
e3 01 a0 00 00 … 00 42
```

Same 36 bytes, same header — but `42` moves from offset 2 to offset 35. A length-only
assertion would pass. This is exactly why the fixture is a byte literal and why
`reordering_fields_changes_the_bytes` asserts equal length *and* unequal content.

## D-WALK 2 — the long-form boundary (`five_field.rs`)

`Five { a: 1u64, b: true, c: 4 bytes, d: Address, e: B256 }`

1 + 1 + (1+4) + (1+20) + (1+32) = **61** payload bytes. 61 > 55 → long form:
`0xf7 + 1` = `f8`, then the length byte `0x3d` (61). Total 63.

Crossing 55 is where a one-byte header silently becomes two; the 54-byte case
(`header_boundary_at_55_bytes`) pins the other side of the line.
