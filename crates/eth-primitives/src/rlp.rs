//! RLP support for the primitive types, behind the `rlp` feature.
//!
//! Mirrors `alloy-primitives`' optional `rlp` feature: the codec crate stays free of any
//! Ethereum-type knowledge, and the batteries crate opts into implementing the codec's
//! traits for its own types.
//!
//! Every type here is an RLP **string** (a byte sequence), never a list — and crucially
//! never zero-trimmed. `B256::ZERO` is 32 zero bytes on the wire, not the empty string.
//! `U256` is the opposite (minimal big-endian); keeping them in separate impls is what
//! stops that rule from leaking across.

use crate::{Bytes, FixedBytes};
use eth_rlp::{BufMut, Decodable, Encodable, Error, Header};

impl<const N: usize> Encodable for FixedBytes<N> {
    fn encode(&self, out: &mut dyn BufMut) {
        // Delegates to the `[u8]` impl, which owns the single-byte-below-0x80 special
        // case. That only ever fires for `FixedBytes<1>`; B256/B64/Address always take
        // the header path.
        self.0[..].encode(out);
    }

    fn length(&self) -> usize {
        self.0[..].length()
    }
}

impl<const N: usize> Decodable for FixedBytes<N> {
    fn decode(buf: &mut &[u8]) -> Result<Self, Error> {
        let header = Header::decode(buf)?;
        if header.list {
            return Err(Error::UnexpectedList);
        }
        // Fixed-width: a 31- or 33-byte string is not a short/long B256, it is malformed.
        if header.payload_length != N {
            return Err(Error::UnexpectedLength);
        }
        if buf.len() < N {
            return Err(Error::InputTooShort);
        }
        let mut out = [0u8; N];
        out.copy_from_slice(&buf[..N]);
        *buf = &buf[N..];
        Ok(FixedBytes(out))
    }
}

impl Encodable for Bytes {
    fn encode(&self, out: &mut dyn BufMut) {
        self[..].encode(out);
    }

    fn length(&self) -> usize {
        self[..].length()
    }
}

impl Decodable for Bytes {
    fn decode(buf: &mut &[u8]) -> Result<Self, Error> {
        let header = Header::decode(buf)?;
        if header.list {
            return Err(Error::UnexpectedList);
        }
        let n = header.payload_length;
        if buf.len() < n {
            return Err(Error::InputTooShort);
        }
        // Owned, so this copies out of the input rather than borrowing it — the same
        // trade `alloy` makes. Decode allocates; encode does not.
        let value = Bytes::from_vec(buf[..n].to_vec());
        *buf = &buf[n..];
        Ok(value)
    }
}
