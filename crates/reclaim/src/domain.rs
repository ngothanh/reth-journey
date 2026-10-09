use crate::root::RootRegistry;
use crate::sync::AtomicPtr;
use crate::{Reclaimer, Retirable, Root};
use std::sync::Arc;

pub struct Domain<R> {
    inner: Arc<Inner<R>>,
}

struct Inner<R> {
    scheme: R,
    roots: RootRegistry,
}

impl<R> Domain<R> {
    pub fn new(scheme: R) -> Self {
        Self {
            inner: Arc::new(Inner {
                scheme,
                roots: RootRegistry::new(),
            }),
        }
    }

    /// Declares `place` as a root of this domain and returns the root for it.
    ///
    /// A structure calls this once for each of its roots when it is built, for
    /// example a queue for its `head` and its `tail`. It keeps the returned
    /// `Root` and passes it to `Guard::protect`.
    ///
    /// # Safety
    ///
    /// The caller promises that `place` stays alive, and stays at the same
    /// address, until `remove_root` is called with the returned root.
    ///
    /// The domain keeps the address of `place`, not a borrow of it. If the
    /// place is freed or moved before `remove_root`, the domain and every
    /// guard that uses this root read memory that is no longer the place.
    pub unsafe fn declare_root<T>(&self, place: &AtomicPtr<T>) -> Root<T> {
        let root = Root::new(place);
        self.inner.roots.declare(root.erase());
        root
    }

    /// Removes a root that `declare_root` returned.
    ///
    /// A structure calls this from its `Drop`, before the place is freed.
    pub fn remove_root<T>(&self, root: Root<T>) {
        self.inner.roots.remove(root.erase())
    }
}

impl<R> Clone for Domain<R> {
    fn clone(&self) -> Self {
        Domain {
            inner: self.inner.clone(),
        }
    }
}

impl<R: Reclaimer> Domain<R> {
    /// Gives a reader a guard from this domain's scheme.
    ///
    /// A reader takes one before it reads, and protects through a root with it.
    pub fn guard(&self) -> R::Guard {
        self.inner.scheme.guard()
    }

    /// Hands an object to this domain's scheme: "I am done with it."
    ///
    /// The object is not freed here. The scheme frees it later, when no
    /// reader can still hold it.
    ///
    /// # Safety
    ///
    /// The caller promises:
    ///
    /// 1. No new reader can get the address of `obj` from any root.
    /// 2. `obj` is handed to `retire` only once.
    /// 3. The caller does not use `obj` after this call.
    pub unsafe fn retire<T: Retirable>(&self, obj: *mut T) {
        #[cfg(debug_assertions)]
        assert!(
            !self.inner.roots.any_root_holds(obj.cast()),
            "retire: a root still holds this object (Rule 1 is broken)"
        );
        // SAFETY: `Reclaimer::retire` asks for the same three promises that
        // this function asks of its own caller, and we pass `obj` on unchanged.
        unsafe { self.inner.scheme.retire(obj) }
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;

    struct NotClone; // a scheme-like type that cannot be cloned

    #[test]
    fn domain_is_clone_for_any_scheme() {
        fn needs_clone<X: Clone>() {}
        needs_clone::<Domain<NotClone>>();
    }

    #[cfg(debug_assertions)]
    #[test]
    fn a_declared_root_is_in_the_domain_until_removed() {
        let domain = Domain::new(NotClone);
        let head: AtomicPtr<u64> = AtomicPtr::new(std::ptr::null_mut());

        // SAFETY: `head` lives to the end of this test and is not moved.
        let root = unsafe { domain.declare_root(&head) };
        assert!(domain.inner.roots.contains(root.erase()));

        domain.remove_root(root);
        assert!(!domain.inner.roots.contains(root.erase()));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn two_domains_do_not_see_each_others_roots() {
        let first = Domain::new(NotClone);
        let second = Domain::new(NotClone);
        let head: AtomicPtr<u64> = AtomicPtr::new(std::ptr::null_mut());

        // SAFETY: `head` lives to the end of this test and is not moved.
        let root = unsafe { first.declare_root(&head) };

        assert!(first.inner.roots.contains(root.erase()));
        assert!(!second.inner.roots.contains(root.erase()));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn a_clone_of_a_domain_sees_the_same_roots() {
        let domain = Domain::new(NotClone);
        let copy = domain.clone();
        let head: AtomicPtr<u64> = AtomicPtr::new(std::ptr::null_mut());

        // SAFETY: `head` lives to the end of this test and is not moved.
        let root = unsafe { domain.declare_root(&head) };

        assert!(copy.inner.roots.contains(root.erase()));
    }

    #[cfg(debug_assertions)]
    mod retire_check {
        use super::*;
        use crate::{Leak, RetireLink};
        use std::ptr::null_mut;

        /// Plays the role of a segment.
        struct Node {
            link: RetireLink,
        }

        impl Node {
            fn new_raw() -> *mut Node {
                Box::into_raw(Box::new(Node {
                    link: RetireLink::new(),
                }))
            }
        }

        unsafe impl Retirable for Node {
            fn retire_link(&self) -> &RetireLink {
                &self.link
            }

            unsafe fn reclaim(ptr: *mut Self) {
                // SAFETY: the node was made with `Box::new`, and the caller
                // promises nobody holds it any more.
                unsafe { drop(Box::from_raw(ptr)) };
            }
        }

        #[test]
        fn retiring_an_object_no_root_holds_is_fine() {
            let domain = Domain::new(Leak::new());
            let node = Node::new_raw();
            let head = AtomicPtr::new(node);
            // SAFETY: `head` lives to the end of this test and is not moved.
            let _root = unsafe { domain.declare_root(&head) };

            // What a correct structure does: first move the root away...
            head.store(null_mut(), crate::sync::Ordering::Release);
            // ...then retire.
            // SAFETY: no root holds the node now, it is retired once, and
            // this test does not use it again.
            unsafe { domain.retire(node) };
        }

        #[test]
        #[should_panic(expected = "Rule 1 is broken")]
        fn retiring_an_object_a_root_still_holds_panics() {
            let domain = Domain::new(Leak::new());
            let node = Node::new_raw();
            let head = AtomicPtr::new(node);
            // SAFETY: `head` lives to the end of this test and is not moved.
            let _root = unsafe { domain.declare_root(&head) };

            // The bug this check is for: `head` still holds the node.
            // SAFETY: none. This call breaks the first promise on purpose;
            // the check must stop it before the scheme sees the node.
            unsafe { domain.retire(node) };
        }

        #[test]
        fn a_removed_root_is_not_looked_at() {
            let domain = Domain::new(Leak::new());
            let node = Node::new_raw();
            let head = AtomicPtr::new(node);
            // SAFETY: `head` lives to the end of this test and is not moved.
            let root = unsafe { domain.declare_root(&head) };

            // The structure is being dropped: it removes its root first.
            domain.remove_root(root);
            // SAFETY: the root is gone, so no reader can reach the node
            // through it. Retired once, not used again.
            unsafe { domain.retire(node) };
        }
    }
}
