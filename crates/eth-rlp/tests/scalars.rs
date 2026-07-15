//! W5 Wed Build — scalar + byte-container + list impls (Phase 2). Turn green in Phase 3.
//!
//! Implement (in the crate, on the real `Encodable`/`Decodable` traits — migrate your spike):
//!   - integers: `u64` (minimal big-endian; 0 -> empty string)  [+ u8..u32, U256 later]
//!   - `bool`
//!   - byte containers as STRINGS: `[u8]`, `Vec<u8>`  [+ &[u8], Bytes, [u8;N] later]
//!   - typed containers as LISTS: `[T]`, `Vec<T>` for non-byte T
//! Each `encode` calls `Header::encode` + writes payload; each `decode` calls `Header::decode`
//! + reads payload + per-type canonicity; each `length` is ARITHMETIC (never scratch-encode).
//! Run: cargo test -p eth-rlp --test scalars

use eth_rlp::{Decodable, Encodable, Error};

fn enc<T: Encodable + ?Sized>(v: &T) -> Vec<u8> {
    let mut out = Vec::new();
    v.encode(&mut out);
    out
}
fn dec<T: Decodable>(mut bytes: &[u8]) -> Result<T, Error> {
    T::decode(&mut bytes)
}

// ============================================================ integers: minimal BE, 0 -> empty

#[test]
fn u64_one_is_single_byte() {
    assert_eq!(enc(&1u64), vec![0x01]); // NOT [0x88, 0,0,0,0,0,0,0,1] — integers aren't fixed-width
}
#[test]
fn u64_zero_is_empty_string() {
    assert_eq!(enc(&0u64), vec![0x80]); // the integer 0 is the EMPTY string, NOT [0x00]
}
#[test]
fn u64_127_is_single_byte() {
    assert_eq!(enc(&127u64), vec![0x7f]);
}
#[test]
fn u64_128_gets_a_header() {
    assert_eq!(enc(&128u64), vec![0x81, 0x80]); // 0x80 is not < 0x80, so it needs a 1-byte-string header
}
#[test]
fn u64_1024_is_minimal_be() {
    assert_eq!(enc(&1024u64), vec![0x82, 0x04, 0x00]); // 0x0400, no leading zero byte
}
#[test]
fn u64_max() {
    let mut expected = vec![0x88];
    expected.extend_from_slice(&[0xff; 8]);
    assert_eq!(enc(&u64::MAX), expected);
}

// ============================================================ bool == integer 0/1

#[test]
fn bool_true_is_one() {
    assert_eq!(enc(&true), vec![0x01]);
}
#[test]
fn bool_false_is_empty_string() {
    assert_eq!(enc(&false), vec![0x80]); // false == integer 0 == empty string
}

// ============================================================ THE specialization: bytes vs typed

#[test]
fn vec_u8_is_a_string() {
    // Vec<u8> -> one string header + raw bytes
    assert_eq!(enc(&vec![1u8, 2, 3]), vec![0x83, 1, 2, 3]);
}
#[test]
fn slice_u8_is_a_string() {
    assert_eq!(enc(&[1u8, 2, 3][..]), vec![0x83, 1, 2, 3]);
}
#[test]
fn vec_u64_is_a_list() {
    // Vec<u64> -> list header + each element RLP-encoded (each is a 1-byte int here)
    assert_eq!(enc(&vec![1u64, 2, 3]), vec![0xc3, 1, 2, 3]);
}
#[test]
fn empty_vec_u8_is_empty_string() {
    assert_eq!(enc(&Vec::<u8>::new()), vec![0x80]);
}
#[test]
fn empty_vec_u64_is_empty_list() {
    assert_eq!(enc(&Vec::<u64>::new()), vec![0xc0]);
}

// ============================================================ length() is arithmetic + matches encode

#[test]
fn u64_length_matches_encode_len() {
    for n in [0u64, 1, 127, 128, 255, 256, 65535, 65536, u64::MAX] {
        assert_eq!(n.length(), enc(&n).len(), "length() != encode().len() for {n}");
    }
}
#[test]
fn vec_u64_length_matches_encode_len() {
    let v = vec![1u64, 256, 65536];
    assert_eq!(v.length(), enc(&v).len());
}

// ============================================================ decode round-trips

#[test]
fn u64_round_trips() {
    for n in [0u64, 1, 127, 128, 255, 256, 1024, 65535, 65536, u64::MAX] {
        assert_eq!(dec::<u64>(&enc(&n)), Ok(n), "round-trip failed for {n}");
    }
}
#[test]
fn bool_round_trips() {
    assert_eq!(dec::<bool>(&enc(&true)), Ok(true));
    assert_eq!(dec::<bool>(&enc(&false)), Ok(false));
}
#[test]
fn vec_u64_round_trips() {
    let v = vec![1u64, 2, 300, 70000];
    assert_eq!(dec::<Vec<u64>>(&enc(&v)), Ok(v));
}

// ============================================================ per-type decode canonicity

#[test]
fn decode_rejects_leading_zero_integer() {
    // [0x82, 0x00, 0x42]: a 2-byte string whose integer bytes start with 0x00 -> non-minimal
    assert_eq!(dec::<u64>(&[0x82, 0x00, 0x42]), Err(Error::NonCanonical));
}
#[test]
fn decode_rejects_byte_zero_as_integer() {
    // [0x00] decodes to integer 0, but the canonical encoding of 0 is [0x80]. Reject the [0x00] form.
    assert_eq!(dec::<u64>(&[0x00]), Err(Error::NonCanonical));
}
