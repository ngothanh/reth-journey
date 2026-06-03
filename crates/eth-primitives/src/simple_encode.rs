pub trait SimpleEncode {
    fn encode(&self, out: &mut Vec<u8>);
}
