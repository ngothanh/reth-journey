use std::ops::{Deref, DerefMut};
use tracing::trace;

pub struct Page(Box<[u8; 4096]>);

impl Page {
    pub fn new() -> Self {
        let page = Self(Box::new([0u8; 4096]));
        trace!(ptr = ?page.0.as_ptr(), "Page allocated");
        page
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Page {
    type Target = [u8; 4096];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Page {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        trace!(ptr = ?self.0.as_ptr(), "Page dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_through_deref() {
        let mut p = Page::new();
        p[0] = 0xab;
        p[4095] = 0xcd;
        assert_eq!(p[0], 0xab);
        assert_eq!(p[4095], 0xcd);
    }

    #[test]
    fn page_size_is_4kib() {
        assert_eq!(Page::new().len(), 4096);
    }

    #[test]
    fn fresh_page_is_zeroed() {
        let p = Page::new();
        assert!(p.iter().all(|&b| b == 0));
    }
}
