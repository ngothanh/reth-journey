use crate::message::EthMessage;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use io::Error;
use tokio::io;
use tokio_util::codec::{Decoder, Encoder};

pub trait Codec:
    Decoder<Item = Self::Message, Error = Self::CodecError> + Encoder<Self::Message>
{
    type Message;
    type CodecError: From<Error>;
}

impl<T, M, E> Codec for T
where
    T: Decoder<Item = M, Error = E> + Encoder<M, Error = E>,
    E: From<Error>,
{
    type Message = M;
    type CodecError = E;
}

const TAG_STATUS: u8 = 0x00;
const TAG_GET_BLOCK_HEADERS: u8 = 0x01;
const TAG_BLOCK_HEADERS: u8 = 0x02;
const TAG_BLOCK_BODIES: u8 = 0x03;
const TAG_NEW_BLOCK: u8 = 0x04;

pub struct EthMessageCodec;

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error(transparent)]
    Io(#[from] Error),

    #[error("Cannot find tag {0:#x}")]
    UnknownTag(u8),

    #[error("truncated {message} payload: need {need} bytes, have {have}")]
    Truncated {
        message: &'static str,
        need: usize,
        have: usize,
    },

    #[error("{0} trailing byte(s) after message payload")]
    TrailingBytes(usize),
}

impl EthMessageCodec {
    pub fn new() -> Self {
        Self {}
    }
}

impl Decoder for EthMessageCodec {
    type Item = EthMessage;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 5 {
            return Ok(None);
        }

        let len = u32::from_be_bytes(src[1..5].try_into().unwrap()) as usize;
        if src.len() < 5 + len {
            src.reserve(5 + len - src.len());
            return Ok(None);
        }
        let tag = src.get_u8();
        let _payload_len = src.get_u32();
        let mut payload = src.split_to(len);

        let message = match tag {
            TAG_STATUS => decode_status(&mut payload)?,
            TAG_GET_BLOCK_HEADERS => decode_get_block_headers(&mut payload)?,
            TAG_BLOCK_HEADERS => decode_block_headers(&mut payload)?,
            TAG_BLOCK_BODIES => decode_block_bodies(&mut payload)?,
            TAG_NEW_BLOCK => decode_new_block(&mut payload)?,
            other => return Err(CodecError::UnknownTag(other)),
        };
        if !payload.is_empty() {
            return Err(CodecError::TrailingBytes(payload.len()));
        }
        Ok(Some(message))
    }
}

impl Encoder<EthMessage> for EthMessageCodec {
    type Error = CodecError;

    fn encode(&mut self, item: EthMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let (tag, payload) = match item {
            EthMessage::Status { .. } => (TAG_STATUS, encode_status(&item)),
            EthMessage::GetBlockHeaders { .. } => {
                (TAG_GET_BLOCK_HEADERS, encode_get_block_header(&item))
            }
            EthMessage::BlockHeaders { .. } => (TAG_BLOCK_HEADERS, encode_block_headers(&item)),
            EthMessage::BlockBodies { .. } => (TAG_BLOCK_BODIES, encode_block_bodies(&item)),
            EthMessage::NewBlock { .. } => (TAG_NEW_BLOCK, encode_new_block(&item)),
        };
        dst.reserve(5 + payload.len());
        dst.put_u8(tag);
        dst.put_u32(payload.len() as u32);
        dst.put_slice(&payload);
        Ok(())
    }
}

fn encode_new_block(message: &EthMessage) -> Bytes {
    let EthMessage::NewBlock {
        block,
        total_difficulty,
    } = message
    else {
        unreachable!("encode_new_block called with non-NewBlock variant");
    };
    let mut buf = BytesMut::with_capacity(4 + block.len() + 16);
    buf.put_u32(block.len() as u32);
    buf.put_slice(block);
    buf.put_u128(*total_difficulty);
    buf.freeze()
}

fn encode_block_bodies(message: &EthMessage) -> Bytes {
    let EthMessage::BlockBodies { request_id, bodies } = message else {
        unreachable!("encode_block_bodies called with non-BlockBodies variant");
    };
    let mut buf = BytesMut::with_capacity(8 + byte_list_len(bodies));
    buf.put_u64(*request_id);
    encode_byte_list(&mut buf, bodies);
    buf.freeze()
}

fn encode_block_headers(message: &EthMessage) -> Bytes {
    let EthMessage::BlockHeaders { request_id, headers } = message else {
        unreachable!("encode_block_headers called with non-BlockHeaders variant");
    };
    let mut buf = BytesMut::with_capacity(8 + byte_list_len(headers));
    buf.put_u64(*request_id);
    encode_byte_list(&mut buf, headers);
    buf.freeze()
}

fn encode_get_block_header(message: &EthMessage) -> Bytes {
    let EthMessage::GetBlockHeaders {
        request_id,
        start_block,
        limit,
        skip,
        reverse,
    } = message
    else {
        unreachable!("encode_get_block_header called with non-GetBlockHeaders variant");
    };
    let mut buf = BytesMut::with_capacity(8 * 4 + 1);
    buf.put_u64(*request_id);
    buf.put_u64(*start_block);
    buf.put_u64(*limit);
    buf.put_u64(*skip);
    buf.put_u8(*reverse as u8);
    buf.freeze()
}

fn encode_status(message: &EthMessage) -> Bytes {
    let EthMessage::Status {
        protocol_version,
        chain_id,
        total_difficulty,
        best_hash,
        genesis_hash,
    } = message
    else {
        unreachable!("encode_status called with non-Status variant");
    };
    let mut buf = BytesMut::with_capacity(1 + 8 + 16 + 32 + 32);
    buf.put_u8(*protocol_version);
    buf.put_u64(*chain_id);
    buf.put_u128(*total_difficulty);
    buf.put_slice(best_hash);
    buf.put_slice(genesis_hash);
    buf.freeze()
}

fn decode_new_block(payload: &mut BytesMut) -> Result<EthMessage, CodecError> {
    ensure(payload, 4, "NewBlock")?;
    let block_len = payload.get_u32() as usize;
    ensure(payload, block_len, "NewBlock")?;
    let block = payload.split_to(block_len).freeze();
    ensure(payload, 16, "NewBlock")?;
    let total_difficulty = payload.get_u128();
    Ok(EthMessage::NewBlock {
        block,
        total_difficulty,
    })
}

fn decode_block_bodies(payload: &mut BytesMut) -> Result<EthMessage, CodecError> {
    ensure(payload, 8, "BlockBodies")?;
    let request_id = payload.get_u64();
    let bodies = decode_byte_list(payload, "BlockBodies")?;
    Ok(EthMessage::BlockBodies { request_id, bodies })
}

fn decode_block_headers(payload: &mut BytesMut) -> Result<EthMessage, CodecError> {
    ensure(payload, 8, "BlockHeaders")?;
    let request_id = payload.get_u64();
    let headers = decode_byte_list(payload, "BlockHeaders")?;
    Ok(EthMessage::BlockHeaders {
        request_id,
        headers,
    })
}

fn decode_get_block_headers(payload: &mut BytesMut) -> Result<EthMessage, CodecError> {
    ensure(payload, 8 + 8 + 8 + 8 + 1, "GetBlockHeaders")?;
    let request_id = payload.get_u64();
    let start_block = payload.get_u64();
    let limit = payload.get_u64();
    let skip = payload.get_u64();
    let reverse = payload.get_u8() != 0;
    Ok(EthMessage::GetBlockHeaders {
        request_id,
        start_block,
        limit,
        skip,
        reverse,
    })
}

fn decode_status(payload: &mut BytesMut) -> Result<EthMessage, CodecError> {
    ensure(payload, 1 + 8 + 16 + 32 + 32, "Status")?;
    let protocol_version = payload.get_u8();
    let chain_id = payload.get_u64();
    let total_difficulty = payload.get_u128();
    let mut best_hash = [0u8; 32];
    payload.copy_to_slice(&mut best_hash);
    let mut genesis_hash = [0u8; 32];
    payload.copy_to_slice(&mut genesis_hash);
    Ok(EthMessage::Status {
        protocol_version,
        chain_id,
        total_difficulty,
        best_hash,
        genesis_hash,
    })
}

/// Serialized size of a `[count][len, bytes]...` byte-list section.
fn byte_list_len(items: &[Bytes]) -> usize {
    4 + items.iter().map(|item| 4 + item.len()).sum::<usize>()
}

fn encode_byte_list(buf: &mut BytesMut, items: &[Bytes]) {
    buf.put_u32(items.len() as u32);
    for item in items {
        buf.put_u32(item.len() as u32);
        buf.put_slice(item);
    }
}

fn decode_byte_list(
    payload: &mut BytesMut,
    message: &'static str,
) -> Result<Vec<Bytes>, CodecError> {
    ensure(payload, 4, message)?;
    let count = payload.get_u32() as usize;
    // Cap the pre-allocation by what the payload can actually hold (each item
    // costs at least its 4-byte length prefix), so a peer claiming a huge count
    // cannot trigger a giant allocation.
    let mut items = Vec::with_capacity(count.min(payload.len() / 4));
    for _ in 0..count {
        ensure(payload, 4, message)?;
        let item_len = payload.get_u32() as usize;
        ensure(payload, item_len, message)?;
        items.push(payload.split_to(item_len).freeze());
    }
    Ok(items)
}

fn ensure(payload: &BytesMut, need: usize, message: &'static str) -> Result<(), CodecError> {
    if payload.len() < need {
        Err(CodecError::Truncated {
            message,
            need,
            have: payload.len(),
        })
    } else {
        Ok(())
    }
}
