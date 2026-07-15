use bytes::BufMut;

pub trait Encodable {
    fn encode(&self, out: &mut dyn BufMut);

    fn length(&self) -> usize;
}

impl<T: Encodable + ?Sized> Encodable for &T {
    fn encode(&self, out: &mut dyn BufMut) {
        (**self).encode(out)
    }

    fn length(&self) -> usize {
        (**self).length()
    }
}

fn encode_header(payload_len: usize, is_list: bool, out: &mut dyn BufMut) {
    let (short, long): (u8, u8) = if is_list { (0xc0, 0xf7) } else { (0x80, 0xb7) };
    if payload_len <= 55 {
        out.put_u8(short + payload_len as u8);
    } else {
        let be = payload_len.to_be_bytes();
        let minimal = match be.iter().position(|&x| x != 0) {
            Some(pos) => &be[pos..],
            None => &[],
        };
        out.put_u8(long + minimal.len() as u8);
        minimal.iter().for_each(|x| out.put_u8(*x));
    }
}

/// Bytes needed to write `l` in minimal big-endian (the length-of-length for long form).
/// Only reached for `l >= 56`, but defined for all `l` for reuse.
fn length_of_length(l: usize) -> usize {
    if l == 0 {
        1
    } else {
        ((usize::BITS - l.leading_zeros() + 7) / 8) as usize
    }
}

/// Arithmetic length of a byte-string encoding (header bytes + payload), no scratch encode.
fn string_length(payload_len: usize) -> usize {
    let header = if payload_len <= 55 {
        1
    } else {
        1 + length_of_length(payload_len)
    };
    header + payload_len
}

impl Encodable for [u8] {
    fn encode(&self, out: &mut dyn BufMut) {
        if self.len() == 1 && self[0] < 0x80 {
            out.put_u8(self[0]);
            return;
        }

        encode_header(self.len(), false, out);
        out.put_slice(self);
    }

    fn length(&self) -> usize {
        if self.len() == 1 && self[0] < 0x80 {
            return 1; // header-less single byte
        }
        string_length(self.len())
    }
}

impl Encodable for u64 {
    fn encode(&self, out: &mut dyn BufMut) {
        let be = self.to_be_bytes();
        let minimal = match be.iter().position(|&x| x != 0) {
            Some(pos) => &be[pos..],
            None => &[],
        };
        minimal.encode(out)
    }

    fn length(&self) -> usize {
        let be = self.to_be_bytes();
        let minimal = match be.iter().position(|&x| x != 0) {
            Some(pos) => &be[pos..],
            None => &[],
        };
        minimal.length()
    }
}

impl Encodable for bool {
    // A bool is the integer 0 or 1: false -> [0x80] (empty string), true -> [0x01].
    fn encode(&self, out: &mut dyn BufMut) {
        (*self as u64).encode(out);
    }

    fn length(&self) -> usize {
        (*self as u64).length()
    }
}

impl<T: Encodable> Encodable for [T] {
    fn encode(&self, out: &mut dyn BufMut) {
        let mut tmp: Vec<u8> = Vec::new();
        self.iter().for_each(|x| x.encode(&mut tmp));
        encode_header(tmp.len(), true, out);
        out.put_slice(&tmp);
    }

    fn length(&self) -> usize {
        let payload: usize = self.iter().map(|x| x.length()).sum();
        let header = if payload <= 55 {
            1
        } else {
            1 + length_of_length(payload)
        };
        header + payload
    }
}

// --- The `Vec<u8>` (string) vs `Vec<T>` (list) specialization ---
// These coexist for the SAME reason `[u8]` and `[T]` do: `u8` does NOT implement `Encodable`,
// so `Vec<T: Encodable>` never overlaps the concrete `Vec<u8>`. Both just delegate to the slice.
impl Encodable for Vec<u8> {
    fn encode(&self, out: &mut dyn BufMut) {
        self[..].encode(out);
    }

    fn length(&self) -> usize {
        self[..].length()
    }
}

impl<T: Encodable> Encodable for Vec<T> {
    fn encode(&self, out: &mut dyn BufMut) {
        self[..].encode(out);
    }

    fn length(&self) -> usize {
        self[..].length()
    }
}
