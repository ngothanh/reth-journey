# W005 Wed/Thu — scalar impls pre-predictions

> **Provenance**: reconstructed from the resulting code, not sealed beforehand.

- **P1 — how does `0u64` encode?** Predicted: `[0x00]`. Actual: **`[0x80]`** — the empty
  string. Integers are minimal big-endian, and the minimal encoding of zero is *no bytes*.
  This is R2 and the single most-repeated RLP mistake.
- **P2 — is `Vec<u8>` a string or a list?** Predicted: string. Actual: string — but only
  because `u8` deliberately does **not** implement `Encodable` here. In alloy it is a list.
  See `../../eth-primitives/notes/05_alloy_diff.md` D2.
- **P3 — does `bool` need its own logic?** Predicted: yes. Actual: no — it delegates to
  `u64`, so `false` → `0x80` and `true` → `0x01` fall out for free.
- **P4 — canonicity on integer decode.** Predicted: a length check. Actual: one check does
  both jobs — rejecting a leading zero byte catches `[0x82, 0x00, 0x42]` *and* `[0x00]`.
- **P5 — tests predicted to fail.** Predicted: `encode_u64_zero_is_empty_string`.
