# W005 Wed/Thu — 🧮 Paper drill: scalars

## D-WALK — integers, by hand

| Value | Minimal BE bytes | Rule | Encoding |
|---|---|---|---|
| `0u64` | `[]` (empty!) | empty string ⇒ `0x80` | `80` |
| `1u64` | `[0x01]` | single byte < 0x80 | `01` |
| `127u64` | `[0x7f]` | single byte < 0x80 | `7f` |
| `128u64` | `[0x80]` | single byte ≥ 0x80 ⇒ header | `81 80` |
| `1024u64` | `[0x04, 0x00]` | 2-byte string | `82 04 00` |
| `u64::MAX` | `[0xff; 8]` | 8-byte string | `88 ff×8` |

The `127 → 128` step is the interesting one: one increment adds a whole byte, because 128
crosses out of the header-less fixed point.

## D-WALK 2 — strings vs lists on the same data

`vec![1u64, 2u64]` (list) vs `vec![0x01u8, 0x02u8]` (string):

```
list:    c2 01 02      ← 0xc0 + 2 payload bytes, each element self-encoding
string:  82 01 02      ← 0x80 + 2 payload bytes, raw
```

Identical payload, one bit of difference in the header, completely different meaning. This
is why `u8: Encodable` is omitted — with it, `Vec<u8>` would silently take the first form.

## D-WALK 3 — the one canonicity check that catches two bugs

Decode rule: *an integer's minimal big-endian bytes never start with `0x00`.*

- `[0x82, 0x00, 0x42]` — claims 2 bytes `00 42`; the value is `0x42`, which should encode as
  `42`. Rejected.
- `[0x00]` — the fixed point says this is the one-byte string `00`; as an integer it is `0`,
  which should encode as `80`. Rejected by the same test, since `payload[0] == 0`.

One line, both cases: `if n > 0 && payload[0] == 0 { return Err(NonCanonical) }`.
