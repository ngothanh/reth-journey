use crate::sync::AtomicPtr;
#[cfg(debug_assertions)]
use std::ops::Not;
#[cfg(debug_assertions)]
use std::ptr;
#[cfg(debug_assertions)]
use std::sync::Mutex;

/// A place that readers start from, declared as a root.
///
/// It holds the address of one `AtomicPtr`, for example a queue's `head`.
/// A structure stores one `Root` for each of its roots.
pub struct Root<T> {
    src: *const AtomicPtr<T>,
}

/// The list of roots declared in one domain.
///
/// Roots of every type are kept together, so each is stored with its type
/// erased. The debug check walks this list to see what each root holds.
///
/// Only that check reads the list, and the check runs only in a debug build.
/// So in a release build the list is not there: this struct has no fields,
/// takes no memory, and `declare` and `remove` do nothing.
pub(crate) struct RootRegistry {
    #[cfg(debug_assertions)]
    inner: Mutex<Vec<Root<()>>>,
}

// A raw pointer is never Send or Sync by default, so we say it ourselves.
// It is safe for two reasons:
// - the place behind the address is an `AtomicPtr`, which is made to be used
//   by many threads at once;
// - the place stays alive, because whoever declared the root promised that.
// No bound on `T`: like `AtomicPtr<T>`, a `Root<T>` shares only an address,
// never the `T` itself.
unsafe impl<T> Sync for Root<T> {}

unsafe impl<T> Send for Root<T> {}

impl<T> Clone for Root<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Root<T> {}

impl<T> Root<T> {
    pub(crate) fn new(src: &AtomicPtr<T>) -> Self {
        Self { src }
    }

    pub(crate) fn ptr(&self) -> &AtomicPtr<T> {
        // SAFETY: `src` came from a real reference in `Root::new`, and whoever
        // declared this root promised the place stays alive until it is removed.
        unsafe { &*self.src }
    }

    pub(crate) fn erase(&self) -> Root<()> {
        Root {
            src: self.src.cast(),
        }
    }
}

impl RootRegistry {
    pub(crate) fn new() -> Self {
        Self {
            #[cfg(debug_assertions)]
            inner: Mutex::new(Vec::new()),
        }
    }

    // `declare` and `remove` exist in both builds, so `Domain` calls them the
    // same way in both. In a release build their body is empty.

    pub(crate) fn declare(&self, root: Root<()>) {
        #[cfg(debug_assertions)]
        self.inner.lock().unwrap().push(root);
        #[cfg(not(debug_assertions))]
        let _ = root;
    }

    pub(crate) fn remove(&self, root: Root<()>) {
        #[cfg(debug_assertions)]
        self.inner
            .lock()
            .unwrap()
            .retain(|x| ptr::eq(x.src, root.src).not());
        #[cfg(not(debug_assertions))]
        let _ = root;
    }

    /// Is this root in the list? Exists only in a debug build, because only
    /// the debug check asks.
    #[cfg(debug_assertions)]
    pub(crate) fn contains(&self, root: Root<()>) -> bool {
        self.inner
            .lock()
            .unwrap()
            .iter()
            .any(|r| ptr::eq(r.src, root.src))
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use crate::root::{Root, RootRegistry};
    use std::cell::Cell;
    use std::ptr;
    use std::sync::atomic::AtomicPtr;

    fn need_sync_send<T: Send + Sync>() {}

    #[test]
    fn root_must_be_send_sync() {
        need_sync_send::<Root<u64>>();
    }

    #[test]
    fn root_is_as_sharable_as_the_atomic_it_names() {
        need_sync_send::<AtomicPtr<Cell<u64>>>();
        need_sync_send::<Root<Cell<u64>>>();
    }

    #[test]
    fn root_is_copy_for_any_t() {
        fn need_copy<X: Copy>() {}

        need_copy::<Root<u64>>();
        need_copy::<Root<String>>();
    }

    #[test]
    fn root_give_back_the_place_it_was_created_from() {
        let head: AtomicPtr<u64> = AtomicPtr::new(ptr::null_mut());
        let root = Root::new(&head);

        assert!(ptr::eq(root.ptr(), &head));
    }

    #[test]
    fn root_is_one_word() {
        assert_eq!(size_of::<Root<String>>(), size_of::<usize>());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn a_declared_root_is_found() {
        let head: AtomicPtr<u64> = AtomicPtr::new(ptr::null_mut());
        let root = Root::new(&head).erase();
        let registry = RootRegistry::new();

        registry.declare(root);

        assert!(registry.contains(root));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn a_removed_root_is_not_found() {
        let head: AtomicPtr<u64> = AtomicPtr::new(ptr::null_mut());
        let root = Root::new(&head).erase();
        let registry = RootRegistry::new();

        registry.declare(root);
        registry.remove(root);

        assert!(!registry.contains(root));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn a_root_that_was_never_declared_is_not_found() {
        let head: AtomicPtr<u64> = AtomicPtr::new(ptr::null_mut());
        let tail: AtomicPtr<u64> = AtomicPtr::new(ptr::null_mut());
        let head_root = Root::new(&head).erase();
        let tail_root = Root::new(&tail).erase();
        let registry = RootRegistry::new();

        registry.declare(head_root);

        assert!(!registry.contains(tail_root));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn removing_one_root_keeps_the_others() {
        let head: AtomicPtr<u64> = AtomicPtr::new(ptr::null_mut());
        let tail: AtomicPtr<u64> = AtomicPtr::new(ptr::null_mut());
        let head_root = Root::new(&head).erase();
        let tail_root = Root::new(&tail).erase();
        let registry = RootRegistry::new();

        registry.declare(head_root);
        registry.declare(tail_root);
        registry.remove(head_root);

        assert!(!registry.contains(head_root));
        assert!(registry.contains(tail_root));
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn the_registry_takes_no_memory_in_a_release_build() {
        assert_eq!(size_of::<RootRegistry>(), 0);
    }
}
