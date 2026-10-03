mod sync {
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};

    #[cfg(not(loom))]
    pub(super) use core::cell::UnsafeCell;
    #[cfg(loom)]
    pub(super) use loom::cell::UnsafeCell;
    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
}

use sync::{AtomicBool, AtomicPtr, AtomicUsize, Ordering, UnsafeCell};