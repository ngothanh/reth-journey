use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PrimitivesError {
    #[error("invalid length: expected {expected} bytes, got {got}")]
    InvalidLength { expected: usize, got: usize },

    #[error("invalid hex: {0}")]
    InvalidHex(String),

    #[error("invalid checksum")]
    InvalidChecksum,

    #[error("integer overflow")]
    Overflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages() {
        let e = PrimitivesError::InvalidLength {
            expected: 32,
            got: 28,
        };
        assert_eq!(e.to_string(), "invalid length: expected 32 bytes, got 28");

        let e = PrimitivesError::InvalidHex("bad char 'g'".into());
        assert_eq!(e.to_string(), "invalid hex: bad char 'g'");

        assert_eq!(
            PrimitivesError::InvalidChecksum.to_string(),
            "invalid checksum"
        );
        assert_eq!(PrimitivesError::Overflow.to_string(), "integer overflow");
    }
}
