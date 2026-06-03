pub trait SimpleEncode {
    fn encode(&self, out: &mut Vec<u8>);
}


impl SimpleEncode for u8 {
    fn encode(&self, out: &mut Vec<u8>) {
        out.push(*self);
    }
}

impl SimpleEncode for u16 {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_be_bytes());
    }
}

impl SimpleEncode for u32 {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_be_bytes());
    }
}