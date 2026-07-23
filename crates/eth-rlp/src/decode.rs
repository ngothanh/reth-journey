use crate::Header;

pub trait Decodable {
    fn decode(buf: &mut &[u8]) -> Result<Self, Error>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Error {
    UnexpectedString,
    UnexpectedList,
    Overflow,
    InputTooShort,
    NonCanonical,
    /// A header's `payload_length` did not match the fixed width the type requires
    /// (e.g. a 31-byte string decoded as `B256`).
    UnexpectedLength,
    /// Bytes remained after a structural decode consumed everything it expected.
    TrailingBytes,
    Custom(&'static str),
}

/// Decode a `T` that must account for **every** byte of `buf`.
///
/// `Decodable::decode` deliberately consumes one item and leaves the rest — that is
/// what makes it composable inside a list. At the outermost layer nothing else is
/// coming, so leftover bytes are malformed input rather than the next item.
/// `alloy-rlp` splits the same two cases as `decode` / `decode_exact`.
pub fn decode_exact<T: Decodable>(mut buf: &[u8]) -> Result<T, Error> {
    let value = T::decode(&mut buf)?;
    if !buf.is_empty() {
        return Err(Error::TrailingBytes);
    }
    Ok(value)
}

impl Decodable for u64 {
    fn decode(buf: &mut &[u8]) -> Result<u64, Error> {
        let header = Header::decode(buf)?;
        if header.list {
            return Err(Error::UnexpectedList);
        }
        let n = header.payload_length;
        if n > 8 {
            return Err(Error::Overflow); // more bytes than a u64 can hold
        }
        if buf.len() < n {
            return Err(Error::InputTooShort);
        }
        let payload = &buf[..n];
        // canonicity: an integer's minimal big-endian bytes never start with 0.
        // This one check rejects BOTH `[0x82, 0x00, 0x42]` and `[0x00]`.
        if n > 0 && payload[0] == 0 {
            return Err(Error::NonCanonical);
        }
        let mut value = 0u64;
        for &b in payload {
            value = (value << 8) | b as u64;
        }
        *buf = &buf[n..];
        Ok(value)
    }
}

impl Decodable for bool {
    fn decode(buf: &mut &[u8]) -> Result<bool, Error> {
        match u64::decode(buf)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Custom("bool must be 0 or 1")),
        }
    }
}

impl<T: Decodable> Decodable for Vec<T> {
    fn decode(buf: &mut &[u8]) -> Result<Vec<T>, Error> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(Error::UnexpectedString);
        }
        let n = header.payload_length;
        if buf.len() < n {
            return Err(Error::InputTooShort);
        }
        // Decode items out of the list's own payload window until it's exhausted.
        let mut payload = &buf[..n];
        let mut out = Vec::new();
        while !payload.is_empty() {
            out.push(T::decode(&mut payload)?);
        }
        *buf = &buf[n..];
        Ok(out)
    }
}
