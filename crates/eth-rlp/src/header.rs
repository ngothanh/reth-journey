use crate::Error;

pub struct Header {
    list: bool,
    payload_length: usize,
}

impl Header {
    pub fn decode(buf: &mut &[u8]) -> Result<Header, Error> {
        let &first = buf.first().ok_or(Error::InputTooShort)?;

        let header = match first {
            0x00..=0x7 => Header {
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
        todo!()
    }
}
