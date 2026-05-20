use crate::RequestId;
use bytes::Bytes;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EthMessage {
    Status {
        protocol_version: u8,
        chain_id: u64,
        total_difficulty: u128,
        best_hash: [u8; 32],
        genesis_hash: [u8; 32],
    },
    GetBlockHeaders {
        request_id: RequestId,
        start_block: u64,
        limit: u64,
        skip: u64,
        reverse: bool,
    },
    BlockHeaders {
        request_id: RequestId,
        headers: Vec<Bytes>,
    },
    BlockBodies {
        request_id: RequestId,
        bodies: Vec<Bytes>,
    },
    NewBlock {
        block: Bytes,
        total_difficulty: u128,
    },
}
