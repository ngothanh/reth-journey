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
        todo!()
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
        todo!()
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
        todo!()
    }
}
