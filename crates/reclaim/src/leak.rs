use crate::{Guard, Reclaimer, Retirable, Root};

pub struct Leak {}

pub struct LeakGuard {}

impl Leak {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Leak {
    fn default() -> Self {
        Self::new()
    }
}

impl Guard for LeakGuard {
    fn try_protect<T>(&mut self, addr: *mut T, _root: &Root<T>) -> Result<*mut T, *mut T> {
        Ok(addr)
    }
}

unsafe impl Reclaimer for Leak {
    type Guard = LeakGuard;

    fn guard(&self) -> Self::Guard {
        LeakGuard {}
    }

    unsafe fn retire<T: Retirable>(&self, obj: *mut T) {
        let _ = obj;
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use crate::RetireLink;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Plays the role of a segment. `drops` counts how many times a node was
    /// really dropped, so a test can see whether the scheme freed it.
    struct Node {
        link: RetireLink,
        drops: Arc<AtomicUsize>,
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
            // SAFETY: the node was made with `Box::new`, and the caller promises
            // nobody holds it any more.
            unsafe { drop(Box::from_raw(ptr)) };
        }
    }

    #[test]
    fn a_retired_object_is_never_reclaimed() {
        let drops = Arc::new(AtomicUsize::new(0));

        {
            let leak = Leak::new();
            for _ in 0..3 {
                let node = Box::into_raw(Box::new(Node {
                    link: RetireLink::new(),
                    drops: Arc::clone(&drops),
                }));
                // SAFETY: the node is in no root, it is retired once, and
                // this test does not use it again.
                unsafe { leak.retire(node) };
            }
        } // the scheme goes away here

        // Not while the scheme lived, and not when it went away either.
        assert_eq!(drops.load(Ordering::Relaxed), 0);
    }
}
