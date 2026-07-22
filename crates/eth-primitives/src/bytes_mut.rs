use crate::Bytes;
use core::alloc::Layout;
use core::cmp::max;
use core::ptr::NonNull;
use std::alloc::{alloc, dealloc, handle_alloc_error};
use std::mem;

pub struct BytesMut {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
}

impl BytesMut {
    pub fn new(cap: usize) -> Self {
        let ptr = if cap == 0 {
            NonNull::dangling()
        } else {
            let layout = Layout::array::<u8>(cap).unwrap();
            let raw = unsafe { alloc(layout) };
            NonNull::new(raw).unwrap_or_else(|| handle_alloc_error(layout))
        };
        Self { ptr, cap, len: 0 }
    }

    pub fn freeze(self) -> Bytes {
        let ptr = self.ptr;
        let len = self.len;
        let cap = self.cap;

        mem::forget(self);
        unsafe { Bytes::from_raw_parts(ptr.as_ptr(), len, cap) }
    }

    pub fn reserve(&mut self, additional: usize) {
        if self.cap - self.len >= additional {
            return;
        }

        let old_cap = self.cap;
        let old_layout = Layout::array::<u8>(old_cap).unwrap();
        let old_ptr = self.ptr.as_ptr();

        let new_cap = max(
            self.cap.saturating_mul(2),
            old_cap.checked_add(additional).expect("capacity overflow"),
        );
        let new_layout = Layout::array::<u8>(new_cap).unwrap();
        let new_ptr = unsafe { alloc(new_layout) };
        if new_ptr.is_null() {
            handle_alloc_error(new_layout);
        }
        unsafe {
            new_ptr.copy_from_nonoverlapping(old_ptr, self.len);
        }

        if old_cap > 0 {
            unsafe {
                dealloc(old_ptr, old_layout);
            }
        }

        self.ptr = NonNull::new(new_ptr).unwrap();
        self.cap = new_cap;
    }

    pub fn extend_from_slice(&mut self, other: &[u8]) {
        self.reserve(other.len());
        debug_assert!(self.len + other.len() <= self.cap);
        unsafe {
            let start = self.ptr.add(self.len);
            start.copy_from_nonoverlapping(NonNull::from(other).cast(), other.len());
        }
        self.len += other.len();
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for BytesMut {
    fn drop(&mut self) {
        if self.cap > 0 {
            let layout = Layout::array::<u8>(self.cap).unwrap();
            unsafe {
                dealloc(self.ptr.as_ptr(), layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_buffer_with_requested_capacity_and_zero_len() {
        let b = BytesMut::new(8);
        assert_eq!(b.len, 0);
        assert_eq!(b.capacity(), 8);
    }

    #[test]
    fn reserve_is_noop_when_capacity_already_sufficient() {
        let mut b = BytesMut::new(16);
        b.extend_from_slice(&[1, 2, 3, 4]);
        b.reserve(4);
        assert_eq!(b.capacity(), 16);
    }

    #[test]
    fn reserve_grows_geometrically_not_linearly() {
        let mut b = BytesMut::new(4);
        b.extend_from_slice(&[1, 2, 3, 4]);
        b.reserve(1);
        assert!(b.capacity() >= 8);
    }

    #[test]
    fn reserve_preserves_existing_contents_across_realloc() {
        let mut b = BytesMut::new(4);
        let data = &[1, 2, 3, 4];
        b.extend_from_slice(data);
        b.reserve(100);
        assert_eq!(b.as_slice(), data)
    }

    #[test]
    fn extend_from_empty_slice_is_noop() {
        let mut b = BytesMut::new(0);
        b.extend_from_slice(&[]);
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn extend_from_slice_appends_within_capacity() {
        let mut b = BytesMut::new(8);
        b.extend_from_slice(&[1, 2, 3, 4]);
        assert_eq!(b.len(), 4);
        assert_eq!(b.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(b.capacity(), 8);
    }

    #[test]
    fn extend_triggers_reserve_when_capacity_exhausted() {
        let mut b = BytesMut::new(2);
        b.extend_from_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(b.len(), 5);
        assert!(b.capacity() >= 5);
        assert_eq!(b.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn extend_many_times_preserves_all_bytes() {
        let mut b = BytesMut::new(0);
        for i in 0u8..200 {
            b.extend_from_slice(&[i]);
        }

        assert_eq!(b.len(), 200);
        let x = b.as_slice();
        for i in 0u8..200 {
            assert_eq!(x[i as usize], i);
        }
    }

    #[test]
    fn freeze_preserves_contents() {
        let mut b = BytesMut::new(0);
        b.extend_from_slice(&[1, 2, 3, 4]);
        let bytes = b.freeze();
        assert_eq!(&bytes[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn freeze_then_drop_bytes_does_not_double_free() {
        let mut b = BytesMut::new(0);
        b.extend_from_slice(&[1, 2, 3, 4]);
        let bytes = b.freeze();
        drop(bytes);
    }

    // ===== Drop =====

    #[test]
    fn drop_after_multiple_grows_does_not_leak_or_double_free() {
        let mut b = BytesMut::new(4);
        for _ in 0..10 {
            b.extend_from_slice(&[0xAA; 17]);
        }
        assert_eq!(b.len(), 170);
        // b drops at end of scope; miri verifies BytesMut::drop freed the final
        // buffer (no leak) and that every intermediate buffer was dealloc'd by
        // reserve's old-layout dealloc (no leak, no double-free).
    }

    #[test]
    fn drop_of_empty_buffer_does_not_dealloc_dangling() {
        let b = BytesMut::new(0);
        drop(b);
        // miri verifies the `if self.cap > 0` guard in Drop prevents calling
        // dealloc on a dangling pointer. NOTE: currently `new(0)` itself is UB
        // (alloc on a zero-sized layout); once `new` is hardened to skip alloc
        // when cap == 0, this test exercises only the drop-side guard.
    }
}
