# W5 Wed Build — scalar / byte-container / list impls (Phase 1 design doc)

> Most of this is the pattern you already own: `encode` = `Header::encode` + payload;
> `decode` = `Header::decode` + payload + per-type canonicity; `length` = arithmetic.
> The 4 things that are NOT just wiring are below. No peeking at `alloy-rlp` impls until Phase 3.
> Target tests: `tests/scalars.rs`.

## 1. THE hard one — `Vec<u8>` (string) vs `Vec<T>` (list)

Both are `Vec<T>`, but they must encode differently:
- `Vec<u8>` / `&[u8]` / `Bytes` / `[u8; N]` → **string** (one header + raw bytes) → `[0x83, 1, 2, 3]`
- `Vec<u64>` / `Vec<T>` for non-byte `T` → **list** (header + each element RLP'd) → `[0xc3, 1, 2, 3]`

**The trap:** you cannot have both `impl<T: Encodable> Encodable for Vec<T>` AND `impl Encodable for Vec<u8>` — they *overlap*, and stable Rust has **no specialization**. So:

- **D2.** How does alloy resolve this? (Read the public API only — don't copy code.) Sketch the trait
  bound / marker that distinguishes "T is a byte" from "T is a list element."
  > ___
- **D3.** Write down your chosen approach. (Simplest that works: **explicit non-generic impls** for the
  byte containers — `[u8]`, `Vec<u8>`, … as strings — and the generic `impl<T> for [T]`/`Vec<T>` as
  lists. Rust's coherence lets a concrete impl and a generic impl coexist as long as they don't
  literally overlap — `[u8]` is concrete, `[T]` is generic, and `Vec<u8>`/`Vec<T>` need care.)
  > ___

## 2. Integer rules (u8..u64, U256)
- **R1.** encode as **minimal big-endian** (strip leading zeros), then string-encode the bytes.
- **R2.** the integer **`0` → empty string `[0x80]`**, NOT `[0x00]`. (After stripping, value 0 leaves
  `[0x00]`; special-case it to `&[]`.)
- `bool` = the integer `0`/`1`.

## 3. Per-type decode canonicity (beyond Header's prefix checks)
- integer decode rejects a **leading-zero payload** (`[0x82, 0x00, 0x42]` as u64 → `NonCanonical`).
- integer decode rejects `[0x00]` (the byte-zero form of 0 — canonical is `[0x80]`/empty).
- string decode rejects a 1-byte payload `< 0x80` (should've been header-less — the rule you spotted
  that lives *here*, not in Header).

## 4. `length()` is arithmetic (R6 — never scratch-encode)
- integer: `length = length_of(minimal_be_bytes)` — i.e. `header_len(n) + payload_len`, where a
  single byte `< 0x80` has `header_len = 0`.
- list: `header_len(payload) + payload`, where `payload = Σ child.length()`.
- `length_of_length(l)` helper (bytes to write `l` in BE): `0→1`, `1..=255→1`, `256..=65535→2`, …
  Boundary-test at `{0, 255, 256, 65535, 65536}`; the `l == 0` case is the classic off-by-one.

## Order to tackle
1. Migrate `u64` + `bool` off the spike (encode via `put_u8`/`put_slice`, add `length`, add `decode`
   with canonicity). → the integer + bool + length + canonicity tests.
2. `[u8]` / `Vec<u8>` as strings, `[T]` / `Vec<T>` as lists — solve the specialization (D2/D3).
   → the `vec_u8_is_a_string` / `vec_u64_is_a_list` tests.
3. (Later) `U256`, `Address`, `B256`, `Bytes` — need eth-primitives types; add when you wire that dep.
