use crate::Error;
use bytes::BufMut;

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub list: bool,
    pub payload_length: usize,
}

impl Header {
    pub fn encode(&self, out: &mut dyn BufMut) {
        let (short, long): (u8, u8) = if self.list {
            (0xc0, 0xf7)
        } else {
            (0x80, 0xb7)
        };
        if self.payload_length <= 55 {
            out.put_u8(short + self.payload_length as u8);
        } else {
            let be = self.payload_length.to_be_bytes();
            let start = be.iter().position(|&b| b != 0).unwrap_or(be.len());
            let len_bytes = &be[start..];
            out.put_u8(long + len_bytes.len() as u8);
            out.put_slice(len_bytes);
        }
    }
    pub fn decode(buf: &mut &[u8]) -> Result<Header, Error> {
        let &first = buf.first().ok_or(Error::InputTooShort)?;

        let header = match first {
            0x00..=0x7f => Header {
                list: false,
                payload_length: 1,
            },
            0x80..=0xb7 => {
                *buf = &buf[1..];
                Header {
                    list: false,
                    payload_length: (first - 0x80) as usize,
                }
            }
            0xb8..=0xbf => Self::decode_long(buf, (first - 0xb7) as usize, false)?,
            0xc0..=0xf7 => {
                *buf = &buf[1..];
                Header {
                    list: true,
                    payload_length: (first - 0xc0) as usize,
                }
            }
            0xf8..=0xff => Self::decode_long(buf, (first - 0xf7) as usize, true)?,
        };

        Ok(header)
    }

    fn decode_long(buf: &mut &[u8], len_of_len: usize, list: bool) -> Result<Header, Error> {
        if buf.len() < 1 + len_of_len {
            return Err(Error::InputTooShort);
        }
        let len_bytes = &buf[1..1 + len_of_len];
        if len_bytes[0] == 0 {
            return Err(Error::NonCanonical);
        }
        let mut payload_length: usize = 0;
        for &b in len_bytes {
            payload_length = (payload_length << 8) | b as usize;
        }
        if payload_length < 56 {
            return Err(Error::NonCanonical);
        }

        *buf = &buf[1 + len_of_len..];
        Ok(Header {
            list,
            payload_length,
        })
    }
}
