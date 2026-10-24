use crate::sync::AtomicPtr;
use core::ptr::null_mut;
use crate::sync::UnsafeCell;

/// The link stored inside every retirable object.
///
/// The field belongs to the object. The value in it belongs to the scheme.
pub struct RetireLink {
    pub(crate) next: AtomicPtr<RetireLink>,
    obj: UnsafeCell<*mut ()>,
    reclaim: UnsafeCell<Option<unsafe fn(*mut ())>>,
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
            obj: UnsafeCell::new(null_mut()),
            reclaim: UnsafeCell::new(None),
        }
    }
}

//SAFETY: handing the link to another thread hands over only the number in obj.
// The object it points to stays where it is.
// The other thread uses the number only to free that object,
// and that is allowed, because the object is Send (Retirable: Send).
unsafe impl Send for RetireLink {}

//SAFETY:
// 1. next is atomic, so sharing it is safe on its own.
// 2. obj and reclaim are written once, by the thread calling retire, and only before the exchange on head makes the link visible.
// No second writer is possible, because retire promises the object is retired only once.
// 3. They are read only by the thread that took the link out of the pile, and only after it took it out.
// 4. The exchange (Release) and the take-out (Acquire) make the write in 2 visible to the read in 3.
// 5. While the object is retired, the object itself does not touch the link (promise 3 of Retirable).
unsafe impl Sync for RetireLink {}

impl Default for RetireLink {
    fn default() -> Self {
        RetireLink::new()
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Plays the role of a segment. `drops` counts how many times a node was
    /// really dropped, so a test can see whether `reclaim` freed it.
    struct Node {
        link: RetireLink,
        data: u64,
        drops: Arc<AtomicUsize>,
    }

    impl Node {
        fn boxed(data: u64, drops: &Arc<AtomicUsize>) -> Box<Node> {
            Box::new(Node {
                link: RetireLink::new(),
                data,
                drops: Arc::clone(drops),
            })
        }
    }

    impl Drop for Node {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::Relaxed);
        }
    }

    unsafe impl Retirable for Node {
        fn retire_link(&self) -> &RetireLink {
            &self.link
        }

        unsafe fn reclaim(ptr: *mut Self) {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        }
    }

    #[test]
    fn a_new_link_is_not_in_the_pile() {
        assert!(RetireLink::new().next.load(Ordering::Relaxed).is_null());
        assert!(RetireLink::default().next.load(Ordering::Relaxed).is_null());
    }

    #[test]
    fn retire_link_is_the_nodes_own_field() {
        let drops = Arc::new(AtomicUsize::new(0));
        let node = Node::boxed(16, &drops);

        assert!(core::ptr::eq(node.retire_link(), &node.link));
        assert_eq!(node.data, 16);
    }

    #[test]
    fn retire_link_is_the_same_every_call() {
        let drops = Arc::new(AtomicUsize::new(0));
        let node = Node::boxed(16, &drops);

        let first: *const RetireLink = node.retire_link();
        let second: *const RetireLink = node.retire_link();
        assert_eq!(first, second);
    }

    #[test]
    fn reclaim_frees_the_node_once() {
        let drops = Arc::new(AtomicUsize::new(0));
        let ptr: *mut Node = Box::into_raw(Node::boxed(16, &drops));
        assert_eq!(drops.load(Ordering::Relaxed), 0, "not freed yet");

        unsafe {
            Node::reclaim(ptr);
        }

        assert_eq!(drops.load(Ordering::Relaxed), 1, "freed exactly once");
    }
}
