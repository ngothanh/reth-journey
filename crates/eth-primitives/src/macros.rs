macro_rules! b256 {
    ($s: literal) => {
        $crate::B256($crate::hex::decode_hex::<32>($s))
    };
}
