use alloc::sync::Arc;
use core::fmt::{Display, Formatter};
use core::ops::{Bound, Deref, RangeBounds};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Bytes(pub Arc<[u8]>);

impl Bytes {
    pub fn new() -> Self {
        Self(Arc::from([]))
    }

    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self(bytes.into())
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
