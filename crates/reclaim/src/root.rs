use crate::sync::AtomicPtr;
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
pub(crate) struct RootRegistry {
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
}

impl RootRegistry {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
        }
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use crate::root::Root;
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
}
