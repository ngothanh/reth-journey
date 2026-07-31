use crate::Account;
use eth_primitives::{Address, Bytes, B256, U256};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StateCacheError {
    #[error("backend error: {0}")]
    Backend(String),

    #[error("account not found: {0}")]
    NotFound(Address),

    #[error("code not found: {0}")]
    CodeNotFound(B256),

    #[error("storage slot not found: {0} @ {1}")]
    StorageNotFound(Address, U256),

    #[error("block hash not found for block {0}")]
    BlockHashNotFound(u64),
}

pub trait StateCache {
    type Error;

    fn basic(&mut self, address: Address) -> Result<Option<Account>, Self::Error>;
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytes, Self::Error>;
    fn storage(&mut self, address: Address, key: U256) -> Result<U256, Self::Error>;
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error>;
}
