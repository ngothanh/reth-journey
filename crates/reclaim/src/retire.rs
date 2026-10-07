use crate::sync::AtomicPtr;
use core::ptr::null_mut;

/// The link stored inside every retirable object.
///
/// The field belongs to the object. The value in it belongs to the scheme.
pub struct RetireLink {
    next: AtomicPtr<()>,
}

/// An object that a scheme can take over and free later.
///
/// Implemented by the things a structure hands to `retire`: a queue segment, a
/// stack node, a page from a pool.
///
/// # Safety
///
/// Whoever implements this trait promises:
///
/// 1. `retire_link` returns a link that is a field of this same object. It is
///    not shared with any other object.
/// 2. `retire_link` returns the same link every time it is called.
/// 3. While the object is retired, the object does not read or write the link.
///    The scheme uses it to chain retired objects together.
/// 4. `reclaim` really frees the object, in the way it was allocated: `Box`,
///    a pool, an arena. It does not keep the pointer or make it reachable again.
///
/// The scheme relies on all four. If one is broken, the scheme can lose
/// objects or free one twice, even though the scheme itself is correct.
pub unsafe trait Retirable: Send {
    /// The link the scheme uses to chain this object into its list.
    fn retire_link(&self) -> &RetireLink;

    /// Frees the object. Called by the scheme, not by the structure.
    ///
    /// The scheme decides *when*; this function knows *how*.
    ///
    /// # Safety
    ///
    /// The caller promises:
    ///
    /// - `ptr` points to a live object of this type that was handed to `retire`;
    /// -  no reader holds the object anymore;
    /// - `reclaim` is called once for this object, and `ptr` is not used after.
    unsafe fn reclaim(ptr: *mut Self);
}

impl RetireLink {
    /// A link for an object that has not been retired.
    pub fn new() -> Self {
        RetireLink {
            next: AtomicPtr::new(null_mut()),
        }
    }
}

impl Default for RetireLink {
    fn default() -> Self {
        RetireLink::new()
    }
}
