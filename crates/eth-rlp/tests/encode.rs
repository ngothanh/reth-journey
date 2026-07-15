//! RLP encoder acceptance tests — canonical vectors from the Ethereum spec.
//! Every test maps to one rule; turn them green in order (Step 1 → Step 4).
//!
//! Requires `pub use encode::Encodable;` in `src/lib.rs`.
//! Run: `cargo test -p eth-rlp`

use eth_rlp::Encodable;

fn enc<T: Encodable + ?Sized>(v: &T) -> Vec<u8> {
    let mut out = Vec::new();
    v.encode(&mut out);
    out
}

// ------------------------------------------------------------------ Step 2: byte strings

#[test]
fn empty_string() {
    // "" -> 0x80
    assert_eq!(enc(&b""[..]), vec![0x80]);
}

#[test]
fn single_byte_below_0x80_is_itself() {
    // a lone byte < 0x80 is its own encoding (no header)
    assert_eq!(enc(&[0x00u8][..]), vec![0x00]);
    assert_eq!(enc(&[0x0fu8][..]), vec![0x0f]);
    assert_eq!(enc(&[0x7fu8][..]), vec![0x7f]);
}

#[test]
fn single_byte_at_or_above_0x80_gets_a_header() {
    // 0x80 is NOT < 0x80, so it's a 1-byte string: header 0x81, then the byte.
    assert_eq!(enc(&[0x80u8][..]), vec![0x81, 0x80]);
    assert_eq!(enc(&[0xffu8][..]), vec![0x81, 0xff]);
}

#[test]
fn short_string_dog() {
    // "dog" -> 0x83 'd' 'o' 'g'
    assert_eq!(enc(&b"dog"[..]), vec![0x83, b'd', b'o', b'g']);
}

#[test]
fn short_string_55_bytes_is_still_short_form() {
    // exactly 55 bytes -> short form, header 0x80 + 55 = 0xb7
    let s = vec![b'a'; 55];
    let out = enc(&s[..]);
    assert_eq!(out[0], 0xb7);
    assert_eq!(&out[1..], &s[..]);
}

#[test]
fn long_string_56_bytes_is_long_form() {
    // 56 > 55 -> long form: 0xb7 + 1 (len-of-len) = 0xb8, then length byte 56, then bytes
    let s = vec![b'a'; 56];
    let out = enc(&s[..]);
    assert_eq!(&out[..2], &[0xb8, 56]);
    assert_eq!(&out[2..], &s[..]);
}

// ------------------------------------------------------------------ Step 3: integers (u64)

#[test]
fn u64_zero_is_empty_string() {
    // the integer 0 encodes as the empty string 0x80 (NOT 0x00)
    assert_eq!(enc(&0u64), vec![0x80]);
}

#[test]
fn u64_small_is_single_byte() {
    assert_eq!(enc(&15u64), vec![0x0f]);
    assert_eq!(enc(&127u64), vec![0x7f]);
}

#[test]
fn u64_128_gets_a_header() {
    // 128 = 0x80, one byte, not < 0x80 -> header 0x81
    assert_eq!(enc(&128u64), vec![0x81, 0x80]);
}

#[test]
fn u64_1024_is_two_bytes_no_leading_zero() {
    // 1024 = 0x0400 -> minimal big-endian [0x04, 0x00] -> header 0x82
    assert_eq!(enc(&1024u64), vec![0x82, 0x04, 0x00]);
}

#[test]
fn u64_max_strips_no_bytes() {
    // u64::MAX = 8 bytes of 0xff -> header 0x88, then 8 x 0xff
    let mut expected = vec![0x88];
    expected.extend_from_slice(&[0xff; 8]);
    assert_eq!(enc(&u64::MAX), expected);
}

// ------------------------------------------------------------------ Step 4: lists

#[test]
fn empty_list() {
    // [] -> 0xc0
    let list: &[u64] = &[];
    assert_eq!(enc(list), vec![0xc0]);
}

#[test]
fn list_cat_dog() {
    // ["cat","dog"] -> payload is the two RLP strings (8 bytes) -> header 0xc0 + 8 = 0xc8
    let list: &[&[u8]] = &[b"cat", b"dog"];
    assert_eq!(
        enc(list),
        vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']
    );
}

#[test]
fn list_of_ints() {
    // [1, 2, 3] -> each is a single byte < 0x80 -> payload [1,2,3] len 3 -> header 0xc3
    let list: &[u64] = &[1, 2, 3];
    assert_eq!(enc(list), vec![0xc3, 0x01, 0x02, 0x03]);
}

#[test]
fn nested_list_of_lists() {
    // [ [], [] ] -> each inner [] is 0xc0 -> payload [0xc0, 0xc0] len 2 -> header 0xc2
    let inner: &[u64] = &[];
    let list: &[&[u64]] = &[inner, inner];
    assert_eq!(enc(list), vec![0xc2, 0xc0, 0xc0]);
}

#[test]
fn long_list_over_55_bytes() {
    // a list of 56 single-byte ints -> payload is 56 bytes -> long form list:
    // 0xf7 + 1 (len-of-len) = 0xf8, then length byte 56, then the 56 payload bytes
    let list: Vec<u64> = (0..56).map(|i| (i % 128) as u64).collect(); // all < 0x80 -> 1 byte each
    let out = enc(&list[..]);
    assert_eq!(&out[..2], &[0xf8, 56]);
    assert_eq!(out.len(), 2 + 56);
}
