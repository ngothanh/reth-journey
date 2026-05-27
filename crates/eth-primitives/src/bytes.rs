use alloc::sync::Arc;
use core::fmt::{Display, Formatter};
use core::ops::{Bound, Deref, RangeBounds};
use core::str::FromStr;

use crate::PrimitivesError;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Bytes(pub Arc<[u8]>);

impl Bytes {
    pub fn new() -> Self {
        Self(Arc::from([]))
    }

    pub unsafe fn from_raw_parts(ptr: *const u8, len: usize, cap: usize) -> Self {
        let vec = unsafe { Vec::from_raw_parts(ptr as *mut u8, len, cap) };
        Self::from_vec(vec)
    }

    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self(bytes.into())
    }

    pub fn concat(parts: impl IntoIterator<Item = impl AsRef<[u8]>>) -> Self {
        let mut out = Vec::new();
        for part in parts {
            out.extend_from_slice(part.as_ref());
        }
        Self::from_vec(out)
    }

    pub fn from_static(bytes: &'static [u8]) -> Self {
        Self(Arc::from(bytes))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        let len = self.len();
        let start = match range.start_bound() {
            Bound::Included(&x) => x,
            Bound::Excluded(&x) => x + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&x) => x + 1,
            Bound::Excluded(&x) => x,
            Bound::Unbounded => len,
        };
        assert!(start <= end, "slice: start ({start}) > end ({end})");
        assert!(end <= len, "slice: end ({end}) > len ({len})");
        Self(Arc::from(&self.0[start..end]))
    }

    pub fn view(&self) -> BytesView<'_> {
        BytesView(&self.0)
    }

    pub fn map_chunks<F>(&self, chunk_size: usize, f: F) -> Self
    where
        F: FnMut(&[u8]) -> Bytes,
    {
        Self::concat(self.0.chunks(chunk_size).map(f))
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Bytes::from_vec(bytes)
    }
}

impl From<&'static [u8]> for Bytes {
    fn from(bytes: &'static [u8]) -> Self {
        Bytes::from_static(bytes)
    }
}

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x")?;
        for b in self.0.iter() {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BytesView<'a>(pub &'a [u8]);

impl<'a> BytesView<'a> {
    pub const fn new(slice: &'a [u8]) -> Self {
        Self(slice)
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> BytesView<'a> {
        let len = self.len();
        let start = match range.start_bound() {
            Bound::Included(&x) => x,
            Bound::Excluded(&x) => x + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&x) => x + 1,
            Bound::Excluded(&x) => x,
            Bound::Unbounded => len,
        };
        assert!(start <= end, "slice: start ({start}) > end ({end})");
        assert!(end <= len, "slice: end ({end}) > len ({len})");
        BytesView(&self.0[start..end])
    }

    pub fn split_at(self, mid: usize) -> (BytesView<'a>, BytesView<'a>) {
        assert!(mid <= self.len());

        let (left, right) = self.0.split_at(mid);
        (BytesView::new(left), BytesView::new(right))
    }
}

impl<'a> From<&'a [u8]> for BytesView<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        BytesView::new(bytes)
    }
}

impl Display for BytesView<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x")?;
        for b in self.0.iter() {
            write!(f, "{:02x}", b)?;
        }

        Ok(())
    }
}

impl AsRef<[u8]> for BytesView<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl Deref for BytesView<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl FromStr for Bytes {
    type Err = PrimitivesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::hex::decode_to_vec(s).map(Bytes::from_vec)
    }
}
