//! RLP impls for the primitive types (the `rlp` feature).
//!
//! Run: `cargo test -p eth-primitives --features rlp --test rlp`

#![cfg(feature = "rlp")]

use eth_primitives::{Address, Bytes, FixedBytes, B256, B64};
use eth_rlp::{decode_exact, Decodable, Encodable, Error};

fn encoded(v: &impl Encodable) -> Vec<u8> {
    let mut out = Vec::new();
    v.encode(&mut out);
    out
}

// ------------------------------------------------------------- FixedBytes

#[test]
fn b256_is_never_zero_trimmed() {
    // THE pitfall. `U256::ZERO` is the empty string 0x80 because integers are minimal
    // big-endian; `B256::ZERO` is 32 explicit zero bytes because it is a fixed-width
    // string. Same 32 bytes of storage, opposite rule.
    let zero: B256 = FixedBytes([0u8; 32]);
    let bytes = encoded(&zero);

    assert_eq!(bytes[0], 0xa0, "must carry a 32-byte string header");
    assert_eq!(bytes.len(), 33);
    assert!(bytes[1..].iter().all(|&b| b == 0));
}

#[test]
fn b256_round_trips() {
    let value: B256 = FixedBytes([0xab; 32]);
    assert_eq!(decode_exact::<B256>(&encoded(&value)).unwrap(), value);
}

#[test]
fn b64_and_address_round_trip() {
    let b64: B64 = FixedBytes([0x11; 8]);
    assert_eq!(encoded(&b64)[0], 0x88);
    assert_eq!(decode_exact::<B64>(&encoded(&b64)).unwrap(), b64);

    let addr = Address::with_last_byte(0x11);
    assert_eq!(encoded(&addr)[0], 0x94);
    assert_eq!(decode_exact::<Address>(&encoded(&addr)).unwrap(), addr);
}

#[test]
fn fixed_bytes_length_is_arithmetic_and_agrees_with_encode() {
    let value: B256 = FixedBytes([0xab; 32]);
    assert_eq!(value.length(), encoded(&value).len());
}

#[test]
fn b256_rejects_a_wrong_width_string() {
    // 31 bytes where 32 are required. This is not a "short B256" — it is malformed.
    let mut short = vec![0x9f];
    short.extend([0u8; 31]);
    assert_eq!(
        B256::decode(&mut &short[..]).unwrap_err(),
        Error::UnexpectedLength
    );

    // And 33 bytes.
    let mut long = vec![0xa1];
    long.extend([0u8; 33]);
    assert_eq!(
        B256::decode(&mut &long[..]).unwrap_err(),
        Error::UnexpectedLength
    );
}

#[test]
fn b256_rejects_a_list() {
    assert_eq!(
        B256::decode(&mut &[0xc1, 0x01][..]).unwrap_err(),
        Error::UnexpectedList
    );
}

#[test]
fn b256_rejects_a_truncated_payload() {
    // Header promises 32 bytes, only 10 follow.
    let mut truncated = vec![0xa0];
    truncated.extend([0u8; 10]);
    assert_eq!(
        B256::decode(&mut &truncated[..]).unwrap_err(),
        Error::InputTooShort
    );
}

#[test]
fn fixed_bytes_of_one_uses_the_headerless_single_byte_form() {
    // The only width where the `[u8]` short-circuit fires. 0x42 < 0x80 -> bare byte.
    let one: FixedBytes<1> = FixedBytes([0x42]);
    assert_eq!(encoded(&one), &[0x42]);
    assert_eq!(one.length(), 1);
    assert_eq!(decode_exact::<FixedBytes<1>>(&[0x42]).unwrap(), one);

    // 0x80 and above still needs a header.
    let high: FixedBytes<1> = FixedBytes([0x80]);
    assert_eq!(encoded(&high), &[0x81, 0x80]);
}

// ------------------------------------------------------------------ Bytes

#[test]
fn empty_bytes_is_the_empty_string() {
    let empty = Bytes::from_vec(Vec::new());
    assert_eq!(encoded(&empty), &[0x80]);
    assert_eq!(decode_exact::<Bytes>(&[0x80]).unwrap(), empty);
}

#[test]
fn single_low_byte_is_headerless() {
    let value = Bytes::from(&[0x42][..]);
    assert_eq!(encoded(&value), &[0x42]);
    assert_eq!(decode_exact::<Bytes>(&[0x42]).unwrap(), value);
}

#[test]
fn single_high_byte_takes_a_header() {
    let value = Bytes::from(&[0x80][..]);
    assert_eq!(encoded(&value), &[0x81, 0x80]);
    assert_eq!(decode_exact::<Bytes>(&[0x81, 0x80]).unwrap(), value);
}

#[test]
fn bytes_round_trips_across_the_55_byte_boundary() {
    for len in [0usize, 1, 2, 54, 55, 56, 57, 1024] {
        let value = Bytes::from_vec(vec![0xab; len]);
        let bytes = encoded(&value);

        assert_eq!(value.length(), bytes.len(), "length() disagrees at len={len}");
        assert_eq!(
            decode_exact::<Bytes>(&bytes).unwrap(),
            value,
            "round trip failed at len={len}"
        );
    }
}

#[test]
fn bytes_rejects_a_list() {
    assert_eq!(
        Bytes::decode(&mut &[0xc1, 0x01][..]).unwrap_err(),
        Error::UnexpectedList
    );
}

#[test]
fn bytes_rejects_a_truncated_payload() {
    assert_eq!(
        Bytes::decode(&mut &[0x85, 0x01, 0x02][..]).unwrap_err(),
        Error::InputTooShort
    );
}

#[test]
fn bytes_decode_advances_the_cursor() {
    let mut cursor = &[0x42, 0xff][..];
    assert_eq!(Bytes::decode(&mut cursor).unwrap(), Bytes::from(&[0x42][..]));
    assert_eq!(cursor, &[0xff]);
}
