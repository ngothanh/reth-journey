use crate::root::RootRegistry;
use crate::sync::AtomicPtr;
use crate::Root;
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

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;

    struct NotClone; // a scheme-like type that cannot be cloned

    #[test]
    fn domain_is_clone_for_any_scheme() {
        fn needs_clone<X: Clone>() {}
        needs_clone::<Domain<NotClone>>();
    }

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

    #[test]
    fn a_clone_of_a_domain_sees_the_same_roots() {
        let domain = Domain::new(NotClone);
        let copy = domain.clone();
        let head: AtomicPtr<u64> = AtomicPtr::new(std::ptr::null_mut());

        // SAFETY: `head` lives to the end of this test and is not moved.
        let root = unsafe { domain.declare_root(&head) };

        assert!(copy.inner.roots.contains(root.erase()));
    }
}
