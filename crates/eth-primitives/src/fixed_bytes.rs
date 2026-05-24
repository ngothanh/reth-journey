use core::str::FromStr;
use std::fmt::Formatter;

use crate::PrimitivesError;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct FixedBytes<const N: usize>(pub [u8; N]);

impl<const N: usize> Default for FixedBytes<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}

impl<const N: usize> From<[u8; N]> for FixedBytes<N> {
    fn from(value: [u8; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> AsRef<[u8]> for FixedBytes<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> AsMut<[u8]> for FixedBytes<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl<const N: usize> core::ops::Deref for FixedBytes<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> core::fmt::Debug for FixedBytes<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl<const N: usize> core::fmt::Display for FixedBytes<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl<const N: usize> FixedBytes<N> {
    pub fn split(&mut self) -> (&mut [u8], &mut [u8]) {
        self.0.split_at_mut(N / 2)
    }
}

impl<const N: usize> FromStr for FixedBytes<N> {
    type Err = PrimitivesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = [0u8; N];
        crate::hex::decode_into(s, &mut buf)?;
        Ok(FixedBytes(buf))
    }
}

#[cfg(test)]
mod tests {
    use crate::FixedBytes;

    #[test]
    fn zero_init() {
        assert_eq!(FixedBytes::<32>::default().as_ref(), [0u8; 32]);
    }

    #[test]
    fn equality() {
        let mut a = FixedBytes([1u8, 2, 3, 4]);
        let b = FixedBytes([1u8, 2, 3, 4]);
        assert_eq!(a, b);

        a.as_mut()[0] = 2;
        assert_ne!(a, b);
    }

    #[test]
    fn slice_access() {
        let mut a = FixedBytes([1u8, 2, 3, 4]);

        assert_eq!(a.as_ref(), [1u8, 2, 3, 4]);

        a.as_mut()[0] = 2;
        assert_eq!(a.as_ref(), [2, 2, 3, 4]);

        assert_eq!(a.len(), 4);
    }

    #[test]
    fn hash_stability() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn h<T: Hash>(v: &T) -> u64 {
            let mut s = DefaultHasher::new();
            v.hash(&mut s);
            s.finish()
        }

        let a = FixedBytes([1u8; 16]);
        let b = FixedBytes([1u8; 16]);
        assert_eq!(h(&a), h(&b));
    }
}
