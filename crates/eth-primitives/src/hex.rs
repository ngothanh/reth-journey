use crate::PrimitivesError;

pub(crate) fn strip_prefix(s: &str) -> &str {
    s.strip_prefix("0x").unwrap_or(s)
}

pub(crate) fn nibble(byte: u8) -> Result<u8, PrimitivesError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(PrimitivesError::InvalidHex(format!(
            "invalid hex char: {:?}",
            byte as char
        ))),
    }
}

pub(crate) fn decode_into(s: &str, out: &mut [u8]) -> Result<(), PrimitivesError> {
    let s = strip_prefix(s);
    if s.len() != out.len() * 2 {
        return Err(PrimitivesError::InvalidLength {
            expected: out.len() * 2,
            got: s.len(),
        });
    }
    for (i, pair) in s.as_bytes().chunks_exact(2).enumerate() {
        out[i] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Ok(())
}

pub(crate) fn decode_to_vec(s: &str) -> Result<Vec<u8>, PrimitivesError> {
    let s = strip_prefix(s);
    if s.len() % 2 != 0 {
        return Err(PrimitivesError::InvalidHex(format!(
            "odd-length hex string (len={})",
            s.len()
        )));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for pair in s.as_bytes().chunks_exact(2) {
        out.push((nibble(pair[0])? << 4) | nibble(pair[1])?);
    }
    Ok(out)
}
