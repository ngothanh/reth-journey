pub use ruint::aliases::U256;

/// Extension methods on `U256` mirroring alloy-primitives' surface.
pub trait U256Ext: Sized {
    /// Construct from a big-endian byte slice. Length must be <= 32.
    fn from_be_slice(slice: &[u8]) -> Self;

    /// Big-endian byte representation with leading zero bytes stripped.
    /// `U256::ZERO` returns an empty `Vec` — matches RLP encoding convention.
    fn to_be_bytes_trimmed_vec(&self) -> Vec<u8>;

    /// Number of significant bits (position of the highest set bit + 1).
    /// `U256::ZERO::bit_len() == 0`.
    fn bit_len(&self) -> usize;
}

impl U256Ext for U256 {
    fn from_be_slice(slice: &[u8]) -> Self {
        U256::from_be_slice(slice)
    }

    fn to_be_bytes_trimmed_vec(&self) -> Vec<u8> {
        let bytes = self.to_be_bytes::<32>();
        let leading = bytes.iter().take_while(|&&b| b == 0).count();
        bytes[leading..].to_vec()
    }

    fn bit_len(&self) -> usize {
        U256::bit_len(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trimmed_vec_zero() {
        assert_eq!(U256::ZERO.to_be_bytes_trimmed_vec(), Vec::<u8>::new());
    }

    #[test]
    fn trimmed_vec_small() {
        let v = U256::from(0x42u64);
        assert_eq!(v.to_be_bytes_trimmed_vec(), vec![0x42]);
    }

    #[test]
    fn trimmed_vec_large() {
        let v = U256::from(0x1234_5678u64);
        assert_eq!(v.to_be_bytes_trimmed_vec(), vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn from_be_slice_round_trip() {
        let original = U256::from(0xdead_beefu64);
        let bytes = original.to_be_bytes_trimmed_vec();
        let mut padded = vec![0u8; 32 - bytes.len()];
        padded.extend(bytes);
        let recovered = <U256 as U256Ext>::from_be_slice(&padded);
        assert_eq!(original, recovered);
    }

    #[test]
    fn bit_len_basics() {
        assert_eq!(<U256 as U256Ext>::bit_len(&U256::ZERO), 0);
        assert_eq!(<U256 as U256Ext>::bit_len(&U256::from(1u64)), 1);
        assert_eq!(<U256 as U256Ext>::bit_len(&U256::from(0xffu64)), 8);
        assert_eq!(<U256 as U256Ext>::bit_len(&U256::from(0x100u64)), 9);
    }
}
