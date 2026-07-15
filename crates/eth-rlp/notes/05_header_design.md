# W5 Tue Build #2 — RLP `Header` (Phase 1 design doc)

> The prefix codec every RLP value starts with. `Header::encode` writes ONLY the prefix
> (1–9 bytes); `Header::decode` reads the prefix + enforces canonicity. The payload is the
> caller's concern. Fill this before writing `header.rs`. No peeking at `alloy-rlp::header.rs`
> until Phase 3. Target tests: `tests/header.rs`.

## Requirements
- **R1.** `Header { list: bool, payload_length: usize }` is the in-memory form.
- **R2.** Header-less case: a single byte `< 0x80` IS its own complete encoding. Decode of such
  a byte returns `Header { list:false, payload_length:1 }` and **does NOT advance** the cursor
  (the byte is the payload). (Encode-side single-byte short-circuit lives in the *string* impl,
  Wednesday — `Header::encode` just writes the prefix.)
- **R3.** Short form: payload `len ∈ [0,55]` → prefix `0x80+len` (string) / `0xc0+len` (list).
- **R4.** Long form: payload `len ≥ 56` → prefix `0xb7+len_of_len` (string) / `0xf7+len_of_len`
  (list), then `len` in minimal big-endian.
- **R5.** Canonicity on decode — reject: (a) long form when `len ≤ 55` (should be short);
  (b) a length field with a leading zero byte; (c) truncated input.
- **R6.** Decode advances the cursor past the prefix to the payload start; validation happens
  before any payload byte is touched.

## Design walk-through (answer each)
- **D1.** Byte-range table — cover all 256 first-byte values: `[0x00..=0x7f]`? `[0x80..=0xb7]`?
  `[0xb8..=0xbf]`? `[0xc0..=0xf7]`? `[0xf8..=0xff]`?
  > ___
- **D2.** `55` (short-form cap) and `56` (long-form min) each appear twice. Describe the bug where
  you write one predicate `<` and the other `<=`, so round-trip breaks *only* at `len ∈ {55,56}`.
  > ___
- **D3.** `length_of_length(len) -> usize` (bytes to represent `len` in big-endian). Table for
  `len ∈ {0, 255, 256, 65535, 65536}`.
  > ___
- **D4.** The header-less case is a *fixed point*: `0x7f` decodes to `(val=0x7f, remaining=[])`.
  Write 3 other bytes with this property, and 1 that does NOT (e.g. `0x80`).
  > ___
- **D5.** Canonicity attack: `[0xb8, 0x01, 0x42]` decodes as "long-form length-1 string, payload
  `[0x42]`". A naive decoder accepts it. Why is this a **consensus split**, and what is the *one*
  canonical encoding of `0x42`?
  > ___
- **D6 — PRE-MORTEM.** Write the 5-line decode interleaving where a long-form header with a
  leading-zero length byte slips past your check. Where must the validation live — before or after
  the payload read?
  > ___

## 🧮 Paper drill — `notes/05_header_drill.md`
- **D-WALK** on `0x42`, a 55-byte payload, and a 56-byte payload: hand-draw each prefix
  (`[0x42]` headerless / `[0xb7]` short cap / `[0xb8, 0x38]` long min), then decode all three and
  show the cursor landing on the payload start.
- **D-WALK** on the attack `[0xb8, 0x01, 0x42]`: trace decode to the point your `payload_length ≥ 56`
  check must fire, and write the canonical `[0x42]` it forces.

## API to implement (make `tests/header.rs` compile → green)
```rust
#[derive(Debug, PartialEq, Eq)]
pub struct Header { pub list: bool, pub payload_length: usize }

impl Header {
    pub fn encode(&self, out: &mut dyn bytes::BufMut);          // writes ONLY the prefix
    pub fn decode(buf: &mut &[u8]) -> Result<Header, crate::Error>; // prefix + canonicity + cursor
}
```
`pub use header::Header;` from `lib.rs`. A `length_of_length` helper is handy here and reused
Wednesday (list headers) — but it can live inside `header.rs` for now.
