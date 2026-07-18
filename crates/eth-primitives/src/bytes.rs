use crate::PrimitivesError;
use alloc::sync::Arc;
use core::fmt::{Display, Formatter};
use core::ops::{Bound, Deref, RangeBounds};
use core::ptr;
use core::ptr::NonNull;
use core::str::FromStr;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct Bytes {
    ptr: NonNull<u8>,
    len: usize,
    ctx: AtomicPtr<()>,
    vtable: &'static Vtable,
}

unsafe impl Send for Bytes {}
unsafe impl Sync for Bytes {}

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

// Two vtables for the not-yet-promoted (KIND_VEC) state. Which one a Bytes gets is
// chosen in `from_vec` by the real low bit of the buffer pointer:
//   EVEN  → buffer ptr had low bit 0. We set the bit to mark KIND_VEC, so recovering
//           the real ptr means MASKING the low bit off:  (ctx & !KIND_MASK).
//   ODD   → buffer ptr had low bit 1 already (== KIND_VEC coincidentally). We store it
//           verbatim, so recovering the real ptr is just `ctx` as-is (NO masking).
// Both treat `ctx & KIND_MASK == KIND_ARC` as "already promoted → act like SHARE".
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

// Reusable: given a pointer to an EXISTING Shared, bump its refcount and hand back a
// SHARE_VTABLE Bytes viewing (ptr, len). Both share_clone and the promotable "already
// promoted" branch call this. Caller guarantees `shared` is a live `*mut Shared`.
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
    if tagged as usize & KIND_MASK == KIND_VEC {
        unsafe { shallow_clone_arc(tagged as *mut Shared, ptr, len) }
    } else {
        let buf = tagged as *mut u8;
        unsafe { promote_vec(ctx, tagged, buf, ptr, len) }
    }
}

fn promotable_even_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {
    let tagged = *ctx.get_mut();
    if tagged as usize & KIND_MASK == KIND_VEC {
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
    if tagged as usize & KIND_MASK == KIND_VEC {
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
                shallow_clone_arc(shared, ptr, len)
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
        if buf as usize & KIND_MASK == KIND_ARC {
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
        BytesView(&self)
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
        &self
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
