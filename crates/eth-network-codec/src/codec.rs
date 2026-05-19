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
        if (src.len() < 5) {
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

        match tag {
            TAG_STATUS => Ok(Some(decode_status(&mut payload)?)),
            TAG_GET_BLOCK_HEADERS => Ok(Some(decode_get_block_headers(&mut payload)?)),
            TAG_BLOCK_HEADERS => Ok(Some(decode_block_headers(&mut payload)?)),
            TAG_BLOCK_BODIES => Ok(Some(decode_block_bodies(&mut payload)?)),
            TAG_NEW_BLOCK => Ok(Some(decode_new_block(&mut payload)?)),
            other => Err(CodecError::UnknownTag(other)),
        }
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

fn encode_new_block(bytes: &EthMessage) -> Bytes {
    todo!()
}

fn encode_block_bodies(bytes: &EthMessage) -> Bytes {
    todo!()
}

fn encode_block_headers(message: &EthMessage) -> Bytes {
    todo!()
}

fn encode_get_block_header(message: &EthMessage) -> Bytes {
    todo!()
}

fn encode_status(message: &EthMessage) -> Bytes {
    todo!()
}

fn decode_new_block(p0: &mut BytesMut) -> Result<EthMessage, CodecError> {
    todo!()
}

fn decode_block_bodies(p0: &mut BytesMut) -> Result<EthMessage, CodecError> {
    todo!()
}

fn decode_block_headers(p0: &mut BytesMut) -> Result<EthMessage, CodecError> {
    todo!()
}

fn decode_get_block_headers(payload: &mut BytesMut) -> Result<EthMessage, CodecError> {
    todo!()
}

fn decode_status(payload: &mut BytesMut) -> Result<EthMessage, CodecError> {
    todo!()
}
