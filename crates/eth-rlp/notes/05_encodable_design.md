# W5 Tue Build — `Encodable` / `Decodable` traits (Phase 1 design doc)

> Fill this BEFORE writing the traits. No peeking at `alloy-rlp::lib.rs` until Phase 3.
> The spike you built (encode-only, `&mut Vec<u8>`) taught you the *rules*; this Build
> is the *real API* (object-safe `dyn BufMut`, arithmetic `length()`, a `Decodable`
> half, a flat `Error`). Target tests: `crates/eth-rlp/tests/traits.rs`.

## Requirements (the contract you're about to define)

- **R1.** `Encodable` has `encode(&self, out: &mut dyn BufMut)` and `length(&self) -> usize`;
  both object-safe.
- **R2.** `length()` is computed **arithmetically** — no impl may encode into a scratch
  buffer to measure.
- **R3.** `Decodable` has `decode(buf: &mut &[u8]) -> Result<Self, Error> where Self: Sized`;
  on `Ok`, the cursor `*buf` points at the first byte AFTER what was consumed.
- **R4.** Signatures mirror `alloy-rlp` exactly (drop-in compatible).
- **R5.** A blanket `impl<T: Encodable + ?Sized> Encodable for &T` exists so containers
  don't need to own.
- **R6.** `Error` is one flat enum — `UnexpectedString`, `UnexpectedList`, `Overflow`,
  `InputTooShort`, `NonCanonical`, `Custom(&'static str)` — no boxed errors. It is 16 bytes
  and `Copy`.

## Design walk-through (answer each — this is the lesson)

- **D1.** Why `out: &mut dyn BufMut` and NOT generic `encode<B: BufMut>(&self, out: &mut B)`?
  > ___

- **D2.** Why does `decode` take `&mut &[u8]` and not `&[u8]`? What's the alternative
  (`(Self, &[u8])` tuple), and why is the cursor style ergonomic for *nested* decoders?
  > ___

- **D3.** Does `Decodable::Error` need to be an associated type / generic? Justify one flat
  `Error` enum instead.
  > ___

- **D4.** `length()` has no sensible default (each type knows its own math). Could `encode`
  default to calling `length` to pre-allocate? Should it? Why / why not?
  > ___

- **D5.** Where does `Error` live — same module as the traits, a separate `error.rs`, or
  re-exported from `lib.rs`? Justify against alloy's layout.
  > ___

- **D6 — THE PRE-MORTEM (do not skip).** Write the *exact* line of caller code that fails to
  compile if `encode` is generic `<B: BufMut>` instead of `dyn`. (Hint: it involves
  `Box<dyn Encodable>` or `Vec<&dyn Encodable>`.) Write the literal error code too.
  > ```rust
  > // the line that won't compile under a generic signature:
  > ___
  > // error[E00__]: ___
  > ```
  If you can't write this line, you don't yet understand object safety well enough to ship
  this trait — go re-read the E0038 rules, then come back.

## 🧮 Paper drill — `notes/05_encodable_drill.md`
- **D-WALK** on `42u64` and `Vec<&dyn Encodable> = vec![&42u64, &"dog"]`: hand-encode each
  byte-by-byte through `encode(&mut dyn BufMut)`, then decode `[0x2a]` back through
  `decode(&mut &[u8])` and show the cursor advancing past the consumed byte.
- **D-STRUCT** on a 3-row vtable sketch (`u64::encode`, `&str::encode`, `Vec<u8>::encode`
  slots): trace the one dynamic dispatch a `&dyn Encodable` makes per element, and mark why a
  generic `<B: BufMut>` slot *cannot* live in that vtable (the E0038 object-safety boundary).

## Output
This doc with R1–R6 answered + D1–D6 + the D6 pre-mortem snippet; plus
`notes/05_encodable_drill.md` with the two worked paper drills.

---

## After the design doc, write the traits (make `tests/traits.rs` compile → green)

In `src/lib.rs` (or a fresh `src/traits.rs` + `pub use`), define:
1. `pub trait Encodable { fn encode(&self, out: &mut dyn bytes::BufMut); fn length(&self) -> usize; }`
2. `pub trait Decodable: Sized { fn decode(buf: &mut &[u8]) -> Result<Self, Error>; }`
3. `pub enum Error { … }` — the six flat variants (R6), `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`.
4. The blanket `impl<T: Encodable + ?Sized> Encodable for &T` (R5).

> **Reconciliation:** your spike in `encode.rs` (old `Encodable` with `&mut Vec<u8>`, and
> `tests/encode.rs`) is SUPERSEDED by this real trait. Set it aside now — comment out
> `mod encode;` while you build the traits — and migrate the impls (`u64`, `[u8]`, `[T]`) to
> the new `&mut dyn BufMut` / `length()` API on **Wednesday** (the logic transfers verbatim;
> only `out.push`→`out.put_u8`, `out.extend_from_slice`→`out.put_slice`, plus the new
> `length()`). The `Header` type (canonicity checks) is the second Tuesday Build.
