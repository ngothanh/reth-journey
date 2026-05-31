macro_rules! b256 {
    ($s: literal) => {
        $crate::FixedBytes($crate::hex::decode_hex($s))
    };
}

macro_rules! address {
    ($s: literal) => {
        todo!()
    };
}

#[cfg(test)]
mod tests {
    use crate::{Address, B256};

    // The two B256 round-trip tests decode this same 32-byte value through
    // different literal spellings (with / without the `0x` prefix) and must
    // land on these exact bytes. Pattern: 00 11 .. ff, repeated twice.
    const EXPECTED_B256: [u8; 32] = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, //
        0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, //
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, //
        0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, //
    ];

    // `static` (not `const`) so we read it through a real memory location in a
    // function — exercises R6 (the macro output is `Copy`-friendly bytes usable
    // in a `static`).
    static G: B256 = b256!("0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff");

    /// R1 + R2: `b256!` with the `0x` prefix expands to a `const` B256 whose
    /// bytes equal a hand-written array.
    #[test]
    fn b256_round_trip_with_prefix() {
        const X: B256 =
            b256!("0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff");
        assert_eq!(X.0, EXPECTED_B256);
    }

    /// R2: the `0x` prefix is optional — the bare-hex spelling decodes to the
    /// same bytes as the prefixed one above.
    #[test]
    fn b256_round_trip_without_prefix() {
        const X: B256 =
            b256!("00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff");
        assert_eq!(X.0, EXPECTED_B256);
    }

    /// R1: `address!` expands to a `const` 20-byte Address matching a
    /// hand-written array.
    #[test]
    fn address_round_trip() {
        const A: Address = address!("0x000102030405060708090a0b0c0d0e0f10111213");
        let expected: [u8; 20] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, //
            0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, //
        ];
        assert_eq!(A.0, expected);
    }

    /// R6: a `static` initialized by `b256!` can be read from a function and
    /// still holds the right bytes (proves the expansion is a usable const
    /// value, not a runtime parse).
    #[test]
    fn b256_in_static_context() {
        fn read_global() -> B256 {
            G
        }
        assert_eq!(read_global().0, EXPECTED_B256);
    }
}
