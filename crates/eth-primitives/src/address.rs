use crate::aliases::B256;
use crate::keccak::keccak256;
use crate::{FixedBytes, PrimitivesError};

pub type Address = FixedBytes<20>;

pub fn parse_address(s: &str) -> Result<Address, PrimitivesError> {
    s.parse()
}

impl Address {
    pub const ZERO: Self = FixedBytes([0u8; 20]);
    pub fn from_word(word: B256) -> Self {
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&word.0[12..32]);
        FixedBytes(addr)
    }

    pub fn with_last_byte(b: u8) -> Self {
        let mut addr = [0u8; 20];
        addr[19] = b;
        FixedBytes(addr)
    }

    pub fn to_checksum(&self, chain_id: Option<u64>) -> String {
        use core::fmt::Write;

        let mut addr_hex = String::with_capacity(40);
        for byte in &self.0 {
            write!(addr_hex, "{:02x}", byte).unwrap();
        }

        let hash_input = match chain_id {
            Some(id) => format!("{id}0x{addr_hex}"),
            None => addr_hex.clone(),
        };

        let hash = keccak256(hash_input.as_bytes());
        let mut res = String::with_capacity(42);
        res.push_str("0x");
        for (i, ch) in addr_hex.chars().enumerate() {
            let nibble = (hash[i / 2] >> ((1 - (i % 2)) * 4)) & 0x0f;
            if ch.is_ascii_alphabetic() && nibble >= 8 {
                res.push(ch.to_ascii_uppercase());
            } else {
                res.push(ch);
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::{Address, FixedBytes};

    #[test]
    fn eip55_vectors() {
        let cases = [
            (
                "0x52908400098527886E0F7030069857D2E4169EE7",
                "52908400098527886e0f7030069857d2e4169ee7",
            ),
            (
                "0x8617E340B3D01FA5F11F306F4090FD50E238070D",
                "8617e340b3d01fa5f11f306f4090fd50e238070d",
            ),
            (
                "0xde709f2102306220921060314715629080e2fb77",
                "de709f2102306220921060314715629080e2fb77",
            ),
            (
                "0x27b1fdb04752bbc536007a920d24acb045561c26",
                "27b1fdb04752bbc536007a920d24acb045561c26",
            ),
            (
                "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
                "5aaeb6053f3e94c9b9a09f33669435e7ef1beaed",
            ),
            (
                "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
                "fb6916095ca1df60bb79ce92ce3ea74c37c5d359",
            ),
            (
                "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
                "dbf03b407c01e7cd3cbea99509d93f8dddc8c6fb",
            ),
            (
                "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
                "d1220a0cf47c7b9be7a2e6ba89f429762e7b9adb",
            ),
        ];
        for (expected, raw) in cases {
            let bytes = hex::decode(raw).unwrap();
            let addr: Address = FixedBytes(bytes.try_into().unwrap());
            assert_eq!(addr.to_checksum(None), expected);
        }
    }
}
