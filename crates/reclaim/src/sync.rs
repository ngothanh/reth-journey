//! The loom shim. Under `--cfg loom` every atomic and cell in this crate comes
//! from `loom` instead of `core`, so the models explore real interleavings.

#[cfg(not(loom))]
pub(crate) use core::cell::UnsafeCell;
#[cfg(not(loom))]
pub(crate) use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};

#[cfg(loom)]
pub(crate) use loom::cell::UnsafeCell;
#[cfg(loom)]
pub(crate) use loom::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
