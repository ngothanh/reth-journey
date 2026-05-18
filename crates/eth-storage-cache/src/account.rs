use eth_primitives::{Bytes, FixedBytes, B256, U256};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
    pub code_hash: B256,
    pub code: Option<Bytes>,
}

pub const EMPTY_CODE_HASH: B256 = FixedBytes([
    0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
    0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
]);

impl Account {
    pub fn is_empty(&self) -> bool {
        self.nonce == 0 && self.balance == U256::ZERO && self.code_hash == EMPTY_CODE_HASH
    }

    pub fn has_code(&self) -> bool {
        self.code_hash != EMPTY_CODE_HASH
    }

    pub fn code_size(&self) -> usize {
        self.code.as_ref().map_or(0, |code| code.len())
    }
}

impl Default for Account {
    fn default() -> Self {
        Self {
            nonce: 0,
            balance: U256::ZERO,
            code_hash: EMPTY_CODE_HASH,
            code: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let acct = Account::default();
        assert!(acct.is_empty());
        assert!(!acct.has_code());
        assert_eq!(acct.code_size(), 0);
    }

    #[test]
    fn empty_code_hash_is_keccak_of_empty() {
        assert_eq!(EMPTY_CODE_HASH, eth_primitives::keccak256(&[][..]));
    }

    #[test]
    fn account_with_balance_is_not_empty() {
        let acct = Account {
            balance: U256::from(1u64),
            ..Default::default()
        };
        assert!(!acct.is_empty());
        assert!(!acct.has_code()); // still no code
    }

    #[test]
    fn contract_has_code() {
        let acct = Account {
            code_hash: eth_primitives::keccak256(b"deadbeef"),
            code: Some(Bytes::from_static(b"deadbeef")),
            ..Default::default()
        };
        assert!(!acct.is_empty());
        assert!(acct.has_code());
        assert_eq!(acct.code_size(), 8);
    }
}
