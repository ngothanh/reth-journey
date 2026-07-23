//! Behaviour tests for `#[derive(RlpEncodable)]`.
//!
//! A derived struct encodes as an RLP *list*: a list header whose payload is the
//! concatenated encodings of the fields, in declaration order.

use eth_primitives_derive::RlpEncodable;
use eth_rlp::Encodable;

fn encoded(v: &impl Encodable) -> Vec<u8> {
    let mut out = Vec::new();
    v.encode(&mut out);
    out
}

/// `length()` is the contract that lets `encode` emit a header before the payload
/// without a scratch buffer. If it ever disagrees with `encode`, every nested
/// structure silently corrupts, so assert it everywhere.
fn assert_roundtrip(v: &impl Encodable, expected: &[u8]) {
    assert_eq!(encoded(v), expected, "encoding mismatch");
    assert_eq!(v.length(), expected.len(), "length() disagrees with encode()");
}

#[derive(RlpEncodable)]
struct Pair {
    a: u64,
    b: u64,
}

#[test]
fn short_list_header() {
    // 1 and 2 are single bytes < 0x80, so they encode headerlessly.
    // payload = [01, 02] -> 2 bytes -> list header 0xc0 + 2.
    assert_roundtrip(&Pair { a: 1, b: 2 }, &[0xc2, 0x01, 0x02]);
}

#[test]
fn matches_slice_impl_for_same_payload() {
    // A 2-field struct of u64 must encode identically to a 2-element u64 list.
    assert_eq!(encoded(&Pair { a: 1, b: 2 }), encoded(&vec![1u64, 2u64]));
}

#[derive(RlpEncodable)]
struct Flags {
    flag: bool,
    n: u64,
}

#[test]
fn zero_and_false_are_empty_strings() {
    // false and 0 both encode as the empty string 0x80, not as 0x00.
    assert_roundtrip(&Flags { flag: false, n: 0 }, &[0xc2, 0x80, 0x80]);
}

#[derive(RlpEncodable)]
struct Blob {
    data: Vec<u8>,
}

#[test]
fn long_list_header() {
    // 60-byte string: 0xb8 0x3c + 60 bytes = 62 bytes of payload.
    // 62 > 55, so the list header takes the long form: 0xf7 + 1, then 62.
    let mut expected = vec![0xf8, 0x3e, 0xb8, 0x3c];
    expected.extend(std::iter::repeat(0xaa).take(60));

    assert_roundtrip(
        &Blob {
            data: vec![0xaa; 60],
        },
        &expected,
    );
}

#[test]
fn header_boundary_at_55_bytes() {
    // 54 data bytes -> 0xb6 + 54 = 55 bytes payload -> still the short list header.
    let data = vec![0xaa; 54];
    let mut expected = vec![0xf7, 0xb6];
    expected.extend(std::iter::repeat(0xaa).take(54));
    assert_roundtrip(&Blob { data }, &expected);
}

#[derive(RlpEncodable)]
struct Wrapper<T> {
    inner: T,
    tag: u64,
}

#[test]
fn generic_field_gets_encodable_bound() {
    assert_roundtrip(
        &Wrapper {
            inner: 1u64,
            tag: 2,
        },
        &[0xc2, 0x01, 0x02],
    );

    // Nested: inner is itself a derived struct -> a list inside a list.
    // inner encodes to [c2 01 02] (3 bytes), tag 2 -> [02]. payload = 4 bytes.
    assert_roundtrip(
        &Wrapper {
            inner: Pair { a: 1, b: 2 },
            tag: 2,
        },
        &[0xc4, 0xc2, 0x01, 0x02, 0x02],
    );
}