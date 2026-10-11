use crate::Guard;
use crate::Retirable;

/// A scheme: the one that decides when a retired object can be freed.
///
/// Each scheme names its own guard type.
///
/// # Example
///
/// A small stack that works with any scheme. `head` is its only root. `pop`
/// goes through every step: take a guard, protect through the root, open the
/// object, move the root away, retire.
///
/// ```
/// use reclaim::{Domain, Guard, Leak, Reclaimer, Retirable, RetireLink, Root};
/// use std::ptr::null_mut;
/// use std::sync::atomic::{AtomicPtr, Ordering};
///
/// struct Node {
///     value: u64,
///     next: *mut Node,
///     link: RetireLink,
/// }
///
/// // SAFETY: a `Node` owns only its own memory. `next` is just the address of
/// // the node below; no safe code reads through it, and dropping a `Node` does
/// // not touch that node.
/// unsafe impl Send for Node {}
///
/// unsafe impl Retirable for Node {
///     fn retire_link(&self) -> &RetireLink {
///         &self.link
///     }
///
///     unsafe fn reclaim(ptr: *mut Self) {
///         // SAFETY: the node was made with `Box::new`, and the caller promises
///         // nobody holds it any more.
///         unsafe { drop(Box::from_raw(ptr)) };
///     }
/// }
///
/// // `push` takes no guard: it never opens the old top, it only copies and
/// // compares its address.
/// fn push(head: &AtomicPtr<Node>, value: u64) {
///     let new_node = Box::into_raw(Box::new(Node {
///         value,
///         next: null_mut(),
///         link: RetireLink::new(),
///     }));
///     let mut top = head.load(Ordering::Relaxed);
///     loop {
///         // SAFETY: `head` does not point to `new_node` yet, so only this
///         // thread can reach it.
///         unsafe { (*new_node).next = top };
///         match head.compare_exchange_weak(top, new_node, Ordering::Release, Ordering::Relaxed) {
///             Ok(_) => return,
///             Err(current) => top = current,
///         }
///     }
/// }
///
/// fn pop<R: Reclaimer>(
///     domain: &Domain<R>,
///     head: &AtomicPtr<Node>,
///     root: &Root<Node>,
/// ) -> Option<u64> {
///     // A reader takes a guard before it reads.
///     let mut guard = domain.guard();
///     loop {
///         // Read the root and protect what it holds.
///         let top = guard.protect(root);
///         if top.is_null() {
///             return None;
///         }
///
///         // SAFETY: `top` is not null and came from `guard.protect`, so the
///         // scheme does not free it while `guard` lives.
///         let next = unsafe { (*top).next };
///
///         // Move the root away from `top`. From here no new reader can get it.
///         if head
///             .compare_exchange_weak(top, next, Ordering::Release, Ordering::Relaxed)
///             .is_ok()
///         {
///             // SAFETY: still protected by `guard`.
///             let value = unsafe { (*top).value };
///
///             // SAFETY: the exchange succeeded, so:
///             // 1. `head` no longer holds `top`, and `head` is the only root,
///             //    so no new reader can get it;
///             // 2. only the thread whose exchange succeeded gets here, so
///             //    `top` is retired once;
///             // 3. `value` was read above, and `top` is not used after this.
///             unsafe { domain.retire(top) };
///             return Some(value);
///         }
///         // Another thread changed `head` first: protect again.
///     }
/// }
///
/// let domain = Domain::new(Leak::new());
/// let head: AtomicPtr<Node> = AtomicPtr::new(null_mut());
/// // SAFETY: `head` lives to the end of this example and is not moved.
/// let root = unsafe { domain.declare_root(&head) };
///
/// push(&head, 1);
/// push(&head, 2);
///
/// assert_eq!(pop(&domain, &head, &root), Some(2));
/// assert_eq!(pop(&domain, &head, &root), Some(1));
/// assert_eq!(pop(&domain, &head, &root), None);
///
/// domain.remove_root(root);
/// ```
///
/// # Safety
///
/// Whoever implements this trait promises:
///
/// - the scheme never frees an object while a guard still protects it;
/// - the scheme frees an object only by calling that object's own
///   `Retirable::reclaim`, and at most once.
///
/// Readers rely on this. A scheme that frees too early makes a reader open
/// freed memory, even though the reader and the structure did everything right.
pub unsafe trait Reclaimer {
    /// What a reader holds while it is reading.
    type Guard: Guard;

    /// Gives a reader something to read with.
    fn guard(&self) -> Self::Guard;

    /// Hands an object to the scheme: "I am done with it."
    ///
    /// The object is not freed here. The scheme frees it later, when no
    /// reader can still hold it.
    ///
    /// # Safety
    ///
    /// The caller promises:
    ///
    /// - no new reader can get this object's address from any root;
    /// - `obj` is handed to `retire` only once;
    /// - the caller does not use `obj` after this call.
    unsafe fn retire<T: Retirable>(&self, obj: *mut T);
}
