//! W005 Friday — `#[derive(RlpEncodable, RlpDecodable)]` behaviour tests.
//!
//! Run: `cargo test -p eth-primitives-derive --test rlp_derive`
//!
//! Exercises R1 (list header + declaration order), R2 (inverse decode), R3 (absolute
//! paths), R4 (arithmetic length), R6 (wire-format doc) and R7 (trailing bytes).
//! R5 (unsupported input shapes) lives in `compile_fail.rs`.

use eth_primitives::{Bytes, FixedBytes, B256};
use eth_primitives_derive::{RlpDecodable, RlpEncodable};
use eth_rlp::{decode_exact, BufMut, Decodable, Encodable, Error};

#[derive(RlpEncodable, RlpDecodable, Debug, PartialEq)]
struct Foo {
    a: u64,
    b: Bytes,
    c: B256,
}

fn sample() -> Foo {
    Foo {
        a: 1,
        b: Bytes::from(&[0x42][..]),
        c: FixedBytes([0u8; 32]),
    }
}

/// The wire-format contract for `sample()`, hand-derived from the RLP spec and pasted
/// as a literal. It is deliberately NOT regenerated from the code: a literal is the only
/// kind of fixture that can catch the code changing. See `field_order_is_wire_format`.
///
/// ```text
/// e3                    list header, payload = 35 bytes (0xc0 + 35)
///   01                  a = 1        -> single byte < 0x80, no header
///   42                  b = [0x42]   -> single byte < 0x80, no header
///   a0 00 * 32          c = B256(0)  -> 32-byte string (0x80 + 32); NOT zero-trimmed
/// ```
#[rustfmt::skip]
const FOO_FIXTURE: &[u8] = &[
    0xe3,
    0x01,
    0x42,
    0xa0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn encoded(v: &impl Encodable) -> Vec<u8> {
    let mut out = Vec::new();
    v.encode(&mut out);
    out
}

// ---------------------------------------------------------------- R1: encode

#[test]
fn encode_three_field_struct_matches_alloy() {
    // The alloy-side mirror uses the structurally identical leaf types: `alloy_rlp::Bytes`
    // is a string like ours, and `[u8; 32]` is a fixed 32-byte string like `B256`.
    #[derive(alloy_rlp::RlpEncodable)]
    struct AlloyFoo {
        a: u64,
        b: alloy_rlp::Bytes,
        c: [u8; 32],
    }

    let alloy = alloy_rlp::encode(&AlloyFoo {
        a: 1,
        b: alloy_rlp::Bytes::from_static(&[0x42]),
        c: [0u8; 32],
    });

    assert_eq!(encoded(&sample()), alloy, "diverged from alloy-rlp-derive");
    assert_eq!(alloy, FOO_FIXTURE, "the pinned fixture disagrees with alloy");
}

#[test]
fn encode_matches_pinned_fixture() {
    assert_eq!(encoded(&sample()), FOO_FIXTURE);
}

// ---------------------------------------------------------------- R2: decode

#[test]
fn decode_three_field_struct_round_trip() {
    let bytes = encoded(&sample());
    let decoded: Foo = decode_exact(&bytes).expect("round trip");
    assert_eq!(decoded, sample());
}

#[test]
fn decode_reads_the_pinned_fixture() {
    let decoded: Foo = decode_exact(FOO_FIXTURE).expect("fixture decodes");
    assert_eq!(decoded, sample());
}

#[test]
fn decode_advances_the_caller_cursor_exactly() {
    // Composability: after decoding one struct the cursor must sit on the next item,
    // which is what lets a derived struct be a field of another derived struct.
    let mut bytes = encoded(&sample());
    bytes.push(0xff);

    let mut cursor = &bytes[..];
    let decoded = Foo::decode(&mut cursor).expect("decodes");
    assert_eq!(decoded, sample());
    assert_eq!(cursor, &[0xff], "cursor must stop at the end of the frame");
}

#[test]
fn decode_rejects_a_string_header() {
    // 0x83 = 3-byte string. A derived struct is always a list.
    let err = Foo::decode(&mut &[0x83, 0x01, 0x02, 0x03][..]).unwrap_err();
    assert_eq!(err, Error::UnexpectedString);
}

// ---------------------------------------------------------- R7: trailing bytes

#[test]
fn decode_rejects_trailing_bytes() {
    // Bytes after a complete frame. `Decodable::decode` leaves them for the next item
    // by design, so the strictness lives in `decode_exact` at the outermost layer.
    let mut bytes = encoded(&sample());
    bytes.push(0xff);

    assert_eq!(decode_exact::<Foo>(&bytes).unwrap_err(), Error::TrailingBytes);
}

#[test]
fn decode_rejects_trailing_bytes_inside_the_frame() {
    // The stricter case, and the one the derive itself must catch: the list header
    // claims 36 payload bytes but the three fields only account for 35. Without the
    // payload-window check this byte is silently swallowed.
    let mut bytes = FOO_FIXTURE.to_vec();
    bytes[0] = 0xe4; // 0xc0 + 36
    bytes.push(0xff);

    assert_eq!(
        Foo::decode(&mut &bytes[..]).unwrap_err(),
        Error::TrailingBytes
    );
}

#[test]
fn decode_rejects_short_input() {
    let bytes = &FOO_FIXTURE[..FOO_FIXTURE.len() - 1];
    assert_eq!(
        Foo::decode(&mut &bytes[..]).unwrap_err(),
        Error::InputTooShort
    );
}

#[test]
fn decode_rejects_a_field_overrunning_the_payload_window() {
    // `c`'s header claims 32 bytes but the list only has room for 31 of them. Fields are
    // decoded from a slice of the payload, so this runs out of input instead of reading
    // into whatever follows the list.
    let mut bytes = FOO_FIXTURE.to_vec();
    bytes[0] = 0xe2; // shrink the list to 34 bytes...
    bytes.pop(); // ...and the buffer with it

    assert!(Foo::decode(&mut &bytes[..]).is_err());
}

// ------------------------------------------------------- R4: arithmetic length

/// An `Encodable` that reports a length but panics if anything actually encodes it.
struct HostileLeaf;

impl Encodable for HostileLeaf {
    fn encode(&self, _out: &mut dyn BufMut) {
        panic!("length() must be arithmetic — it must never call encode()");
    }

    fn length(&self) -> usize {
        1
    }
}

#[derive(RlpEncodable)]
struct WithHostileLeaf {
    a: u64,
    hostile: HostileLeaf,
}

#[test]
fn length_is_arithmetic_not_encode() {
    let value = WithHostileLeaf {
        a: 1,
        hostile: HostileLeaf,
    };

    // 1-byte list header + 1 byte for `a` + 1 claimed byte for the hostile leaf.
    assert_eq!(value.length(), 3);
}

#[test]
fn length_agrees_with_encode() {
    assert_eq!(sample().length(), encoded(&sample()).len());
    assert_eq!(sample().length(), FOO_FIXTURE.len());
}

// ------------------------------------------------------- R3: path hygiene

/// No `use eth_rlp::…` anywhere in this module — only the derive names are imported.
/// If the expansion leaned on a caller-side import, this module would not compile.
mod no_rlp_imports {
    use eth_primitives_derive::{RlpDecodable, RlpEncodable};

    #[derive(RlpEncodable, RlpDecodable, Debug, PartialEq)]
    pub(crate) struct Bare {
        pub(crate) a: u64,
        pub(crate) b: u64,
    }
}

#[test]
fn works_without_caller_use_import() {
    let value = no_rlp_imports::Bare { a: 1, b: 2 };
    assert_eq!(encoded(&value), &[0xc2, 0x01, 0x02]);
    assert_eq!(
        decode_exact::<no_rlp_imports::Bare>(&[0xc2, 0x01, 0x02]).unwrap(),
        value
    );
}

// -------------------------------------------------- R6: field order is the wire

#[test]
fn field_order_is_wire_format_fixture() {
    // Guards against a "cosmetic" field reorder. The fixture is a literal, so this test
    // cannot be silently repaired by regenerating it alongside the struct change.
    assert_eq!(
        encoded(&sample()),
        FOO_FIXTURE,
        "wire format changed — reordering, adding, or removing a field of `Foo` is a \
         consensus-breaking change, not a refactor"
    );
}

#[test]
fn reordering_fields_changes_the_bytes() {
    // The same three values declared as { a, c, b } instead of { a, b, c }.
    #[derive(RlpEncodable)]
    struct Reordered {
        a: u64,
        c: B256,
        b: Bytes,
    }

    let reordered = encoded(&Reordered {
        a: 1,
        c: FixedBytes([0u8; 32]),
        b: Bytes::from(&[0x42][..]),
    });

    assert_eq!(reordered.len(), FOO_FIXTURE.len(), "same payload, same length");
    assert_ne!(reordered, FOO_FIXTURE, "a field swap must move the bytes");
}

// -------------------------------------------------------------- generics

#[derive(RlpEncodable, RlpDecodable, Debug, PartialEq)]
struct Wrapper<T> {
    inner: T,
    tag: u64,
}

#[test]
fn generic_struct_round_trips() {
    let value = Wrapper {
        inner: Wrapper { inner: 1u64, tag: 2 },
        tag: 3,
    };

    let bytes = encoded(&value);
    // inner = c2 01 02 (3 bytes), tag = 03 (1 byte) -> payload 4 -> header c4.
    assert_eq!(bytes, &[0xc4, 0xc2, 0x01, 0x02, 0x03]);
    assert_eq!(decode_exact::<Wrapper<Wrapper<u64>>>(&bytes).unwrap(), value);
}
