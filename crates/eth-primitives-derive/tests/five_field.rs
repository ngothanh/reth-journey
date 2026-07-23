//! W005 Friday — the 5-field spread, exercising every scalar and byte-container leaf
//! from Wednesday through the derive at once.
//!
//! Run: `cargo test -p eth-primitives-derive --test five_field`
//!
//! The payload here is 61 bytes, which pushes the list header past the 55-byte cutoff
//! into long form — the three-field test in `rlp_derive.rs` only covers short form.

use eth_primitives::{Address, Bytes, FixedBytes, B256};
use eth_primitives_derive::{RlpDecodable, RlpEncodable};
use eth_rlp::{decode_exact, Encodable};

#[derive(RlpEncodable, RlpDecodable, Debug, PartialEq)]
struct Five {
    a: u64,
    b: bool,
    c: Bytes,
    d: Address,
    e: B256,
}

fn sample() -> Five {
    Five {
        a: 1,
        b: true,
        c: Bytes::from(&[0xde, 0xad, 0xbe, 0xef][..]),
        d: Address::with_last_byte(0x11),
        e: FixedBytes([0xab; 32]),
    }
}

/// Hand-derived from the RLP spec and pasted as a literal — never regenerated.
///
/// ```text
/// f8 3d                 list header, payload = 61 bytes (long form: 0xf7+1, then 61)
///   01                  a = 1              -> single byte < 0x80, no header
///   01                  b = true           -> the integer 1, same encoding as a
///   84 de ad be ef      c = 4-byte string  -> 0x80 + 4
///   94 00*19 11         d = 20-byte string -> 0x80 + 20
///   a0 ab*32            e = 32-byte string -> 0x80 + 32
/// ```
#[rustfmt::skip]
const FIVE_FIXTURE: &[u8] = &[
    0xf8, 0x3d,
    0x01,
    0x01,
    0x84, 0xde, 0xad, 0xbe, 0xef,
    0x94,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x11,
    0xa0,
    0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab,
    0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab,
    0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab,
    0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab,
];

fn encoded(v: &impl Encodable) -> Vec<u8> {
    let mut out = Vec::new();
    v.encode(&mut out);
    out
}

#[test]
fn five_field_struct_matches_alloy_byte_for_byte() {
    #[derive(alloy_rlp::RlpEncodable)]
    struct AlloyFive {
        a: u64,
        b: bool,
        c: alloy_rlp::Bytes,
        d: [u8; 20],
        e: [u8; 32],
    }

    let mut d = [0u8; 20];
    d[19] = 0x11;

    let alloy = alloy_rlp::encode(&AlloyFive {
        a: 1,
        b: true,
        c: alloy_rlp::Bytes::from_static(&[0xde, 0xad, 0xbe, 0xef]),
        d,
        e: [0xab; 32],
    });

    assert_eq!(encoded(&sample()), alloy, "diverged from alloy-rlp-derive");
    assert_eq!(alloy, FIVE_FIXTURE, "the pinned fixture disagrees with alloy");
}

#[test]
fn five_field_struct_matches_pinned_fixture() {
    assert_eq!(encoded(&sample()), FIVE_FIXTURE);
}

#[test]
fn five_field_struct_round_trips() {
    let decoded: Five = decode_exact(&encoded(&sample())).expect("round trip");
    assert_eq!(decoded, sample());
}

#[test]
fn five_field_length_is_arithmetic_and_agrees_with_encode() {
    assert_eq!(sample().length(), FIVE_FIXTURE.len());
    assert_eq!(sample().length(), 63);
}

#[test]
fn long_form_list_header_is_used() {
    // 61 > 55, so the header must be two bytes, not one. A one-byte header here would
    // mean the payload silently wrapped.
    assert_eq!(&FIVE_FIXTURE[..2], &[0xf8, 0x3d]);
    assert_eq!(FIVE_FIXTURE.len(), 2 + 61);
}
