//! W5 Tue Build — trait-contract tests (Phase 2). Turn green in Phase 3.
//!
//! These test the SHAPE of `Encodable` / `Decodable` / `Error`, not concrete-type
//! encoding (that's Wednesday). They use tiny local fixtures so they don't depend on
//! the `u64`/`[u8]`/list impls.
//!
//! To make this file COMPILE you must define, in the crate:
//!   - `pub trait Encodable { fn encode(&self, out: &mut dyn bytes::BufMut); fn length(&self) -> usize; }`
//!   - `pub trait Decodable: Sized { fn decode(buf: &mut &[u8]) -> Result<Self, Error>; }`
//!   - `pub enum Error { UnexpectedString, UnexpectedList, Overflow, InputTooShort, NonCanonical, Custom(&'static str) }`
//!     (`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`)
//!   - the blanket `impl<T: Encodable + ?Sized> Encodable for &T`
//! Run: `cargo test -p eth-rlp --test traits`

use bytes::BufMut;
use eth_rlp::{Decodable, Encodable, Error};

// --- tiny fixtures (a 1-byte value 0x01) so the trait-shape tests need no real impls ---

#[derive(Debug, PartialEq)]
struct One;

impl Encodable for One {
    fn encode(&self, out: &mut dyn BufMut) {
        out.put_u8(0x01);
    }
    fn length(&self) -> usize {
        1 // arithmetic — never encodes into a scratch buffer (R2)
    }
}

impl Decodable for One {
    fn decode(buf: &mut &[u8]) -> Result<Self, Error> {
        let &first = buf.first().ok_or(Error::InputTooShort)?;
        if first != 0x01 {
            return Err(Error::Custom("expected 0x01"));
        }
        *buf = &buf[1..]; // advance the cursor past the byte we consumed (R3)
        Ok(One)
    }
}

// ---------------------------------------------------------------------- R1 / D6: object safety

#[test]
fn encodable_is_object_safe() {
    // If `encode` were generic `<B: BufMut>`, this line would not compile (E0038):
    // a generic method can't live in a vtable. `dyn BufMut` is what makes it dyn-able.
    let items: Vec<&dyn Encodable> = vec![&One, &One];
    let mut out: Vec<u8> = Vec::new();
    for it in &items {
        it.encode(&mut out);
    }
    assert_eq!(out, vec![0x01, 0x01]);
}

// ---------------------------------------------------------------------- R2: length ≠ encode

static ENCODE_CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

struct Counting;
impl Encodable for Counting {
    fn encode(&self, out: &mut dyn BufMut) {
        ENCODE_CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        out.put_u8(0);
    }
    fn length(&self) -> usize {
        1 // computed, not measured
    }
}

#[test]
fn length_does_not_call_encode() {
    use std::sync::atomic::Ordering::SeqCst;
    ENCODE_CALLS.store(0, SeqCst);
    let _ = Counting.length();
    assert_eq!(
        ENCODE_CALLS.load(SeqCst),
        0,
        "length() must be pure arithmetic, never a scratch-encode"
    );
}

// ---------------------------------------------------------------------- R3: cursor advances

#[test]
fn decode_advances_cursor() {
    let bytes = [0x01u8, 0x01];
    let mut buf: &[u8] = &bytes;
    let _ = One::decode(&mut buf).expect("decode One");
    assert_eq!(
        buf,
        &[0x01],
        "decode must advance *buf past what it consumed"
    );
}

#[test]
fn decode_input_too_short_is_an_error() {
    let mut buf: &[u8] = &[];
    assert_eq!(One::decode(&mut buf), Err(Error::InputTooShort));
}

// ---------------------------------------------------------------------- R5: blanket &T impl

#[test]
fn blanket_impl_for_ref() {
    // `takes` requires its argument to be `Encodable` BY VALUE; passing `&One`
    // only works if the blanket `impl Encodable for &T` exists.
    fn takes<T: Encodable>(v: T) -> usize {
        v.length()
    }
    assert_eq!(takes(&One), takes(One));
}

// ---------------------------------------------------------------------- R6: flat, Copy Error

#[test]
fn error_enum_is_flat_and_copy() {
    // Largest variant is Custom(&'static str) = a 16-byte fat pointer. The 5 unit variants
    // can't niche-fit into &str's single spare value (null ptr), so the discriminant takes a
    // separate 8-byte-aligned word: 16 + 8 = 24. That's the minimal FLAT layout.
    // A `Custom(String)` variant would be 24-byte payload -> 32 total (and fail the Copy guard
    // below); a `Box<dyn Error>` would fail Copy too. So this size + Copy pins "no heap variant".
    assert_eq!(size_of::<Error>(), 24, "Error must be a flat 24-byte enum (no heap variant)");

    // compile-time guard: any heap-owning variant (String/Box) is not Copy and fails here.
    fn assert_copy<T: Copy>() {}
    assert_copy::<Error>();
}

// ---------------------------------------------------------------------- deferred to Wednesday
//
// #[test]
// fn signatures_match_alloy_rlp() {
//     // needs the concrete u64/Bytes/Vec impls (Wed) + the alloy-rlp dev-dep.
//     fn assert_compat<T: alloy_rlp::Encodable + eth_rlp::Encodable>() {}
//     assert_compat::<u64>();
//     assert_compat::<Vec<u64>>();
// }
