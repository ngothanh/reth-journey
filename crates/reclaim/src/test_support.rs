//! Objects shared by the unit tests and the loom models.

use crate::sync::{AtomicUsize, Ordering};
use crate::{Retirable, RetireLink};

#[cfg(loom)]
pub(crate) use loom::sync::Arc;
#[cfg(not(loom))]
pub(crate) use std::sync::Arc;
use std::sync::Mutex;

/// Plays the role of a segment. `drops` counts how many times a node was
/// really dropped, so a test can see whether `reclaim` freed it.
pub(crate) struct Node {
    pub(crate) link: RetireLink,
    pub(crate) data: u64,
    pub(crate) drops: Arc<AtomicUsize>,
}

/// An object that belongs to a pool: `reclaim` gives it back to the pool
/// instead of freeing it.
pub(crate) struct Pooled {
    pub(crate) link: RetireLink,
    pub(crate) pool: Arc<Mutex<Vec<PoolSlot>>>,
}

/// The address of one `Pooled` object sitting in the pool.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) struct PoolSlot(pub(crate) *mut Pooled);

// SAFETY: a slot is only an address. Moving it to another thread moves the
// address, not the object behind it, and the pool hands each address out
// through its `Mutex`.
unsafe impl Send for PoolSlot {}

unsafe impl Retirable for Pooled {
    fn retire_link(&self) -> &RetireLink {
        &self.link
    }

    unsafe fn reclaim(ptr: *mut Self) {
        // SAFETY: the caller promises `ptr` points to a live `Pooled`.
        // It is not dropped here: it goes back to the pool, which owns it
        // from now on.
        unsafe {
            (*ptr).pool.lock().unwrap().push(PoolSlot(ptr));
        }
    }
}

impl Node {
    pub(crate) fn boxed(data: u64, drops: &Arc<AtomicUsize>) -> Box<Node> {
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
