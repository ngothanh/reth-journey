//! W5 Tue Build #2 — RLP `Header` prefix codec (Phase 2). Turn green in Phase 3.
//!
//! Define in the crate (e.g. `src/header.rs`, `pub use`'d from lib.rs):
//!   #[derive(Debug, PartialEq, Eq)]
//!   pub struct Header { pub list: bool, pub payload_length: usize }
//!   impl Header {
//!       pub fn encode(&self, out: &mut dyn bytes::BufMut);            // writes ONLY the prefix
//!       pub fn decode(buf: &mut &[u8]) -> Result<Header, Error>;      // prefix + canonicity + cursor
//!   }
//! Run: cargo test -p eth-rlp --test header

use eth_rlp::{Error, Header};

fn enc(h: &Header) -> Vec<u8> {
    let mut out = Vec::new();
    h.encode(&mut out);
    out
}

// ------------------------------------------------ encode: the prefix bytes only

#[test]
fn encode_short_string_prefix() {
    // string, payload 0..=55 -> single prefix byte 0x80 + len
    assert_eq!(enc(&Header { list: false, payload_length: 0 }), vec![0x80]);
    assert_eq!(enc(&Header { list: false, payload_length: 1 }), vec![0x81]);
    assert_eq!(enc(&Header { list: false, payload_length: 55 }), vec![0xb7]);
}

#[test]
fn encode_long_string_prefix() {
    // 56 -> [0xb8, 56];  257 -> [0xb9, 0x01, 0x01]  (0x0101 = 257, minimal 2-byte BE)
    assert_eq!(enc(&Header { list: false, payload_length: 56 }), vec![0xb8, 56]);
    assert_eq!(enc(&Header { list: false, payload_length: 257 }), vec![0xb9, 0x01, 0x01]);
}

#[test]
fn encode_short_list_prefix() {
    // list, payload 0..=55 -> single prefix byte 0xc0 + len
    assert_eq!(enc(&Header { list: true, payload_length: 0 }), vec![0xc0]);
    assert_eq!(enc(&Header { list: true, payload_length: 8 }), vec![0xc8]);
    assert_eq!(enc(&Header { list: true, payload_length: 55 }), vec![0xf7]);
}

#[test]
fn encode_long_list_prefix() {
    assert_eq!(enc(&Header { list: true, payload_length: 56 }), vec![0xf8, 56]);
}

// ------------------------------------------------ decode: prefix + cursor + single-byte

#[test]
fn decode_short_string_advances_past_prefix() {
    let mut buf: &[u8] = &[0x83, b'd', b'o', b'g'];
    let h = Header::decode(&mut buf).unwrap();
    assert_eq!(h, Header { list: false, payload_length: 3 });
    assert_eq!(buf, b"dog", "cursor must point at the payload start (past the 1 prefix byte)");
}

#[test]
fn decode_advances_cursor_to_payload() {
    // header announces payload_length = 5, then 5 payload bytes + a trailing byte
    let mut buf: &[u8] = &[0x85, 10, 11, 12, 13, 14, 0xff];
    let h = Header::decode(&mut buf).unwrap();
    assert_eq!(h.payload_length, 5);
    assert_eq!(buf, &[10, 11, 12, 13, 14, 0xff], "cursor must land on the first payload byte");
}

#[test]
fn decode_single_byte_below_0x80_is_headerless() {
    // 0x42 < 0x80 IS its own value: Header{false, 1}, cursor NOT advanced (0x42 is the payload).
    let mut buf: &[u8] = &[0x42];
    let h = Header::decode(&mut buf).unwrap();
    assert_eq!(h, Header { list: false, payload_length: 1 });
    assert_eq!(buf, &[0x42], "single-byte value is header-less: cursor must NOT advance");
}

#[test]
fn decode_long_string_reads_length() {
    // [0xb8, 56, ...56 bytes...] -> Header{false, 56}, cursor past the 2 prefix bytes
    let mut bytes = vec![0xb8u8, 56];
    bytes.extend(std::iter::repeat(b'a').take(56));
    let mut buf: &[u8] = &bytes;
    let h = Header::decode(&mut buf).unwrap();
    assert_eq!(h, Header { list: false, payload_length: 56 });
    assert_eq!(buf.len(), 56, "cursor at payload start");
}

// ------------------------------------------------ decode: CANONICITY (consensus security)

#[test]
fn decode_rejects_non_canonical_long_form_for_len_under_56() {
    // [0xb8, 0x01, 0x42]: long-form announcing a length-1 string. Length 1 MUST use short form;
    // accepting this = two byte strings decode to one value = a consensus split.
    let mut buf: &[u8] = &[0xb8, 0x01, 0x42];
    assert_eq!(Header::decode(&mut buf), Err(Error::NonCanonical));
}

#[test]
fn decode_rejects_leading_zero_length() {
    // [0xb9, 0x00, 0x38, ...]: 2-byte length field [0x00, 0x38] has a leading zero.
    // The canonical length is [0x38] (prefix 0xb8). Leading-zero length = NonCanonical.
    let mut bytes = vec![0xb9u8, 0x00, 0x38];
    bytes.extend(std::iter::repeat(b'a').take(0x38));
    let mut buf: &[u8] = &bytes;
    assert_eq!(Header::decode(&mut buf), Err(Error::NonCanonical));
}

#[test]
fn decode_rejects_truncated_input() {
    let mut empty: &[u8] = &[];
    assert_eq!(Header::decode(&mut empty), Err(Error::InputTooShort));

    // long-form header claims 1 length byte, but the buffer ends after the prefix
    let mut short: &[u8] = &[0xb8];
    assert_eq!(Header::decode(&mut short), Err(Error::InputTooShort));
}

// ------------------------------------------------ the 55/56 boundary round-trip

#[test]
fn boundary_55_and_56_round_trip() {
    for &len in &[55usize, 56] {
        for &list in &[false, true] {
            let h = Header { list, payload_length: len };
            let mut bytes = enc(&h);
            bytes.extend(std::iter::repeat(0u8).take(len)); // dummy payload for the cursor to land on
            let mut buf: &[u8] = &bytes;
            let decoded = Header::decode(&mut buf).unwrap();
            assert_eq!(decoded, h, "round-trip failed at len={len}, list={list}");
        }
    }
}
