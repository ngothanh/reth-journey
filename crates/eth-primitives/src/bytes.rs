use crate::PrimitivesError;
use core::fmt::{Debug, Display, Formatter};
use core::ops::{Bound, Deref, RangeBounds};
use core::ptr;
use core::ptr::NonNull;
use core::str::FromStr;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::hash::{Hash, Hasher};

pub struct Bytes {
    ptr: NonNull<u8>,
    len: usize,
    ctx: AtomicPtr<()>,
    vtable: &'static Vtable,
}

unsafe impl Send for Bytes {}
unsafe impl Sync for Bytes {}

// Equality/hashing are BY CONTENT (via the deref'd slice), never by pointer identity —
// two Bytes over equal bytes must compare equal and hash the same, even if they back
// different allocations. This is what lets Bytes be a HashMap key.
impl PartialEq for Bytes {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for Bytes {}

impl PartialEq<[u8]> for Bytes {
    fn eq(&self, other: &[u8]) -> bool {
        **self == *other
    }
}

impl Hash for Bytes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl Debug for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Reuse the hex Display form: Bytes(0x0aff)
        write!(f, "Bytes(")?;
        Display::fmt(self, f)?;
        write!(f, ")")
    }
}

impl Default for Bytes {
    fn default() -> Self {
        Bytes::from_static(&[])
    }
}

struct Vtable {
    clone: unsafe fn(&AtomicPtr<()>, *const u8, usize) -> Bytes,
    drop: unsafe fn(&mut AtomicPtr<()>, *const u8, usize),
}

struct Shared {
    buf: *mut u8,
    cap: usize,
    ref_count: AtomicUsize,
}

const KIND_ARC: usize = 0b0;
const KIND_VEC: usize = 0b1;
const KIND_MASK: usize = 0b1;

impl Clone for Bytes {
    fn clone(&self) -> Self {
        unsafe { (self.vtable.clone)(&self.ctx, self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for Bytes {
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(&mut self.ctx, self.ptr.as_ptr(), self.len) };
    }
}

static STATIC_VTABLE: Vtable = Vtable {
    clone: static_clone,
    drop: static_drop,
};

static PROMOTABLE_EVEN_VTABLE: Vtable = Vtable {
    clone: promotable_even_clone,
    drop: promotable_even_drop,
};

static PROMOTABLE_ODD_VTABLE: Vtable = Vtable {
    clone: promotable_odd_clone,
    drop: promotable_odd_drop,
};

static SHARE_VTABLE: Vtable = Vtable {
    clone: share_clone,
    drop: share_drop,
};

fn static_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    unsafe {
        Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8),
            len,
            ctx: AtomicPtr::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }
}

fn static_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {}

fn share_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = ctx.load(Ordering::Relaxed) as *mut Shared;
    unsafe { shallow_clone_arc(shared, ptr, len) }
}

unsafe fn shallow_clone_arc(shared: *mut Shared, ptr: *const u8, len: usize) -> Bytes {
    unsafe {
        let old = (*shared).ref_count.fetch_add(1, Ordering::Relaxed);
        if old > isize::MAX as usize / 2 {
            std::process::abort();
        }
        Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8),
            len,
            ctx: AtomicPtr::new(shared as *mut ()),
            vtable: &SHARE_VTABLE,
        }
    }
}

fn share_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {
    let shared = ctx.load(Ordering::Relaxed) as *mut Shared;
    unsafe { release_shared(shared) }
}

unsafe fn release_shared(shared: *mut Shared) {
    unsafe {
        if (*shared).ref_count.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }
        core::sync::atomic::fence(Ordering::Acquire);
        let cap = (*shared).cap;
        drop(Vec::from_raw_parts((*shared).buf, cap, cap));
        drop(Box::from_raw(shared));
    }
}

fn promotable_even_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let tagged = ctx.load(Ordering::Acquire);
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe { shallow_clone_arc(tagged as *mut Shared, ptr, len) }
    } else {
        let buf = (tagged as usize & !KIND_MASK) as *mut u8;
        unsafe { promote_vec(ctx, tagged, buf, ptr, len) }
    }
}

fn promotable_odd_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let tagged = ctx.load(Ordering::Acquire);
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe { shallow_clone_arc(tagged as *mut Shared, ptr, len) }
    } else {
        let buf = tagged as *mut u8;
        unsafe { promote_vec(ctx, tagged, buf, ptr, len) }
    }
}

fn promotable_even_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {
    let tagged = *ctx.get_mut();
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe {
            release_shared(tagged as *mut Shared);
        }
    } else {
        let buf = (tagged as usize & !KIND_MASK) as *mut u8;
        unsafe {
            free_boxed_slice(buf, ptr, len);
        }
    }
}

fn promotable_odd_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {
    let tagged = *ctx.get_mut();
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe {
            release_shared(tagged as *mut Shared);
        }
    } else {
        let buf = tagged as *mut u8;
        unsafe {
            free_boxed_slice(buf, ptr, len);
        }
    }
}

unsafe fn promote_vec(
    ctx: &AtomicPtr<()>,
    tagged: *mut (),
    buf: *mut u8,
    ptr: *const u8,
    len: usize,
) -> Bytes {
    let cap = (ptr as usize - buf as usize) + len;
    let shared = Box::into_raw(Box::new(Shared {
        buf,
        cap,
        ref_count: AtomicUsize::new(2),
    }));
    unsafe {
        match ctx.compare_exchange(
            tagged,
            shared as *mut (),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => Bytes {
                ptr: NonNull::new_unchecked(ptr as *mut u8),
                len,
                ctx: AtomicPtr::new(shared as *mut ()),
                vtable: &SHARE_VTABLE,
            },
            Err(actual) => {
                drop(Box::from_raw(shared));
                shallow_clone_arc(actual as *mut Shared, ptr, len)
            }
        }
    }
}

unsafe fn free_boxed_slice(buf: *mut u8, ptr: *const u8, len: usize) {
    let cap = (ptr as usize - buf as usize) + len;
    unsafe {
        drop(Vec::from_raw_parts(buf, cap, cap));
    }
}

impl Bytes {
    pub unsafe fn from_raw_parts(ptr: *const u8, len: usize, cap: usize) -> Self {
        let vec = unsafe { Vec::from_raw_parts(ptr as *mut u8, len, cap) };
        Self::from_vec(vec)
    }

    pub fn from_vec(bytes: Vec<u8>) -> Self {
        if bytes.is_empty() {
            return Self::from_static(&[]);
        }
        let boxed: Box<[u8]> = bytes.into_boxed_slice();
        let len = boxed.len();
        let buf = Box::into_raw(boxed) as *mut u8;
        if buf as usize & KIND_MASK == 0 {
            let ctx = (buf as usize | KIND_VEC) as *mut ();
            unsafe {
                Bytes {
                    ptr: NonNull::new_unchecked(buf),
                    len,
                    ctx: AtomicPtr::new(ctx),
                    vtable: &PROMOTABLE_EVEN_VTABLE,
                }
            }
        } else {
            unsafe {
                Bytes {
                    ptr: NonNull::new_unchecked(buf),
                    len,
                    ctx: AtomicPtr::new(buf as *mut ()),
                    vtable: &PROMOTABLE_ODD_VTABLE,
                }
            }
        }
    }

    pub fn concat(parts: impl IntoIterator<Item = impl AsRef<[u8]>>) -> Self {
        let mut out = Vec::new();
        for part in parts {
            out.extend_from_slice(part.as_ref());
        }
        Self::from_vec(out)
    }

    pub fn from_static(bytes: &'static [u8]) -> Self {
        unsafe {
            Bytes {
                ptr: NonNull::new_unchecked(bytes.as_ptr() as *mut u8),
                len: bytes.len(),
                ctx: AtomicPtr::default(),
                vtable: &STATIC_VTABLE,
            }
        }
    }

    pub fn len(&self) -> usize {
        (**self).len()
    }

    pub fn is_empty(&self) -> bool {
        (**self).is_empty()
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
        if start == end {
            return Bytes::from_static(&[]);
        }
        let mut tmp = self.clone();
        tmp.ptr = unsafe { NonNull::new_unchecked(tmp.ptr.as_ptr().add(start)) };
        tmp.len = end - start;
        tmp
    }

    pub fn view(&self) -> BytesView<'_> {
        BytesView(self.as_ref())
    }

    pub fn map_chunks<F>(&self, chunk_size: usize, f: F) -> Self
    where
        F: FnMut(&[u8]) -> Bytes,
    {
        Self::concat(self.chunks(chunk_size).map(f))
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
        for b in self.iter() {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &**self
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_static_exposes_the_underlying_bytes() {
        let b = Bytes::from_static(b"hello");
        assert_eq!(&*b, b"hello");
        assert_eq!(b.len(), 5);
        assert!(!b.is_empty());
    }

    #[test]
    fn from_static_empty_is_empty() {
        let b = Bytes::from_static(&[]);
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
        assert_eq!(&*b, &[] as &[u8]);
    }

    #[test]
    fn clone_of_static_is_cheap_and_equal() {
        let b = Bytes::from_static(b"static");
        let c = b.clone();
        assert_eq!(&*b, &*c);
    }

    #[test]
    fn slice_of_static_yields_subrange() {
        let b = Bytes::from_static(b"0123456789");
        assert_eq!(&*b.slice(2..5), b"234");
        assert_eq!(&*b.slice(..), b"0123456789");
        assert_eq!(&*b.slice(7..), b"789");
    }

    // ── owned (from_vec) representation ────────────────────────────────────
    #[test]
    fn from_vec_roundtrips_contents() {
        let b = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
        assert_eq!(&*b, &[1, 2, 3, 4, 5]);
        assert_eq!(b.len(), 5);
    }

    #[test]
    fn from_vec_empty_routes_to_empty_repr() {
        let b = Bytes::from_vec(Vec::new());
        assert!(b.is_empty());
        assert_eq!(&*b, &[] as &[u8]);
    }

    #[test]
    fn clone_of_owned_shares_contents() {
        let b = Bytes::from_vec(vec![9, 8, 7]);
        let c = b.clone(); // first clone promotes b to shared
        assert_eq!(&*b, &*c);
        assert_eq!(&*c, &[9, 8, 7]);
    }

    #[test]
    fn owned_clone_outlives_the_original() {
        let c = {
            let b = Bytes::from_vec(vec![10, 20, 30]);
            let c = b.clone();
            drop(b); // original gone, backing must survive via refcount
            c
        };
        assert_eq!(&*c, &[10, 20, 30]);
    }

    // ── slice sharing ──────────────────────────────────────────────────────
    #[test]
    fn slice_of_owned_returns_subrange() {
        let b = Bytes::from_vec((0u8..10).collect());
        assert_eq!(&*b.slice(3..7), &[3, 4, 5, 6]);
    }

    #[test]
    fn slice_empty_range_is_empty() {
        let b = Bytes::from_vec(vec![1, 2, 3]);
        assert!(b.slice(1..1).is_empty());
    }

    #[test]
    fn nested_slices_stay_correct() {
        let b = Bytes::from_vec((0u8..20).collect());
        let s = b.slice(5..15); // [5..15]
        assert_eq!(&*s.slice(2..4), &[7, 8]); // [7..9] of the original
    }

    #[test]
    #[should_panic]
    fn slice_end_past_len_panics() {
        let b = Bytes::from_static(b"abc");
        let _ = b.slice(0..99);
    }

    #[test]
    #[should_panic]
    fn slice_start_after_end_panics() {
        let b = Bytes::from_static(b"abcdef");
        #[allow(clippy::reversed_empty_ranges)]
        let _ = b.slice(4..2);
    }

    // ── constructors / helpers ─────────────────────────────────────────────
    #[test]
    fn concat_joins_all_parts() {
        let b = Bytes::concat([&b"ab"[..], &b"cd"[..], &b"ef"[..]]);
        assert_eq!(&*b, b"abcdef");
    }

    #[test]
    fn map_chunks_transforms_each_chunk() {
        let b = Bytes::from_vec((0u8..6).collect());
        let out = b.map_chunks(2, |c| Bytes::from_vec(c.to_vec()));
        assert_eq!(&*out, &[0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn display_formats_as_0x_hex() {
        let b = Bytes::from_static(&[0x0a, 0xff, 0x00]);
        assert_eq!(format!("{b}"), "0x0aff00");
    }

    #[test]
    fn from_str_parses_hex() {
        let b: Bytes = "0x0aff".parse().unwrap();
        assert_eq!(&*b, &[0x0a, 0xff]);
    }

    #[test]
    fn view_borrows_the_same_bytes() {
        let b = Bytes::from_vec(vec![1, 2, 3]);
        assert_eq!(b.view().as_ref(), &[1, 2, 3]);
    }

    // ── content-based traits ───────────────────────────────────────────────
    #[test]
    fn equality_is_by_content_across_representations() {
        // same bytes, different backing allocations / representations
        let owned = Bytes::from_vec(vec![1, 2, 3]);
        let stat = Bytes::from_static(&[1, 2, 3]);
        assert_eq!(owned, stat);
        assert_ne!(owned, Bytes::from_static(&[1, 2, 4]));
    }

    #[test]
    fn hash_agrees_with_equality() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Bytes::from_vec(vec![7, 7]));
        assert!(set.contains(&Bytes::from_static(&[7, 7])));
    }

    #[test]
    fn default_is_empty() {
        assert!(Bytes::default().is_empty());
    }

    #[test]
    fn debug_shows_hex() {
        let b = Bytes::from_static(&[0x0a, 0xff]);
        assert_eq!(format!("{b:?}"), "Bytes(0x0aff)");
    }

    // ── concurrency: exercises the promotion CAS race + refcount ordering ──
    // Run repeatedly and under Miri/loom to shake out ordering bugs:
    //   cargo +nightly miri test concurrent
    #[test]
    fn concurrent_first_clones_race_promotion_safely() {
        let original = Bytes::from_vec((0u8..64).collect());
        std::thread::scope(|s| {
            let handles: Vec<_> = (0..16)
                .map(|_| {
                    s.spawn(|| {
                        // all threads clone the SAME handle → concurrent promote_vec
                        let c = original.clone();
                        assert_eq!(c.len(), 64);
                        c
                    })
                })
                .collect();
            for h in handles {
                let c = h.join().unwrap();
                assert_eq!(&*c, &(0u8..64).collect::<Vec<_>>()[..]);
            }
        });
        // original still valid after every clone dropped
        assert_eq!(&*original, &(0u8..64).collect::<Vec<_>>()[..]);
    }

    #[test]
    fn clones_dropped_across_threads_free_exactly_once() {
        let original = Bytes::from_vec(vec![1, 2, 3, 4]);
        std::thread::scope(|s| {
            for _ in 0..8 {
                let c = original.clone();
                s.spawn(move || {
                    assert_eq!(&*c, &[1, 2, 3, 4]);
                    drop(c);
                });
            }
        });
        assert_eq!(&*original, &[1, 2, 3, 4]);
    }
}
