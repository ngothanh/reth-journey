//! `PageBox<T>` — a `Box`-like smart pointer over a 4 KiB [`Page`].
//!
//! # Lesson recap
//!
//! A smart pointer is **a struct with a `Deref` impl**. `Box`, `Rc`, `Arc`,
//! `MutexGuard` — all the same shape, different storage strategies. `PageBox`
//! replaces "arbitrary heap allocation" (what `Box<T>` uses) with "one 4 KiB
//! [`Page`]" — the same primitive `storage-trie` Phase 3 will lay over mmap.
//!
//! Single-allocation deserialize-in-place: the bytes of the [`Page`] **are**
//! the in-memory representation of `T`. No second allocation, no copy on read.
//! This is the shape MDBX cursors return — `&[u8]` pointing at mmap'd memory,
//! interpreted as `T` without copying.
//!
//! # Today's scope: `T: Sized`
//!
//! The README spec calls for `PageBox<T: ?Sized>` ultimately, but Rust requires
//! `Drop` bounds to match the struct's bounds exactly — and a sound `?Sized`
//! `Drop` impl needs DST-metadata machinery (specialized impls for `[u8]`,
//! `dyn Trait`, etc.). That redesign lands in W4 (Rustonomicon-driven unsafe +
//! `Layout`) and W27 (`MmapPageProvider` returning borrowed page views).
//! Today's MVP is `T: Sized` — the smart-pointer pattern and the in-place
//! deserialization both land without the DST complexity.
//!
//! # Safety contract today
//!
//! - `T` must fit: `size_of::<T>() <= 4096`. Enforced via runtime `assert!`.
//! - The `Page`'s heap allocation must be suitably aligned for `T`. Enforced
//!   at runtime via `align_offset`. Holds on modern allocators for any `T`
//!   with `align_of <= 16`, which covers all Ethereum primitive types.
//!   W4 will replace the runtime check with `Layout`-based static guarantees.

use core::marker::PhantomData;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::page::Page;

pub struct PageBox<T> {
    page: Page,
    _marker: PhantomData<T>,
}

impl<T> PageBox<T> {
    pub fn new(value: T) -> Self {
        assert!(
            size_of::<T>() <= 4096,
            "T ({} bytes) does not fit in a 4 KiB Page",
            size_of::<T>()
        );

        let mut page = Page::new();
        let dst = page.as_mut_ptr().cast::<T>();
        assert_eq!(
            dst.align_offset(align_of::<T>()),
            0,
            "Page allocation address {:p} is not aligned for T (align = {})",
            dst,
            align_of::<T>()
        );

        // SAFETY: size + alignment verified above; we move `value` into the
        // page bytes; `_marker: PhantomData<T>` declares logical ownership of T.
        unsafe {
            dst.write(value);
        }

        Self {
            page,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for PageBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: `new` guarantees a valid T is initialized at the start of `page`.
        unsafe { &*self.page.as_ptr().cast::<T>() }
    }
}

impl<T> DerefMut for PageBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: same as `deref`, plus exclusive access via `&mut self`.
        unsafe { &mut *self.page.as_mut_ptr().cast::<T>() }
    }
}

impl<T> Drop for PageBox<T> {
    fn drop(&mut self) {
        // SAFETY: drop the T in place so its destructor runs (matters when T
        // owns heap resources, e.g. T = String). The Page itself drops next,
        // releasing the 4 KiB allocation — its tracing::trace! fires after.
        unsafe {
            ptr::drop_in_place(self.page.as_mut_ptr().cast::<T>());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_primitive() {
        let pb = PageBox::new(0xdead_beef_u64);
        assert_eq!(*pb, 0xdead_beef_u64);
    }

    #[test]
    fn mutation_through_deref_mut() {
        let mut pb = PageBox::new(0u32);
        *pb = 42;
        assert_eq!(*pb, 42);
    }

    #[test]
    fn round_trip_struct() {
        #[derive(Debug, PartialEq)]
        struct Account {
            nonce: u64,
            balance: u128,
        }

        let pb = PageBox::new(Account {
            nonce: 7,
            balance: 1_000_000_000_000_000_000,
        });
        assert_eq!(pb.nonce, 7);
        assert_eq!(pb.balance, 1_000_000_000_000_000_000);
    }

    #[test]
    #[should_panic(expected = "does not fit in a 4 KiB Page")]
    fn rejects_oversized_t() {
        struct TooBig {
            _data: [u8; 8192],
        }
        let _ = PageBox::new(TooBig { _data: [0u8; 8192] });
    }

    #[test]
    fn drops_t_in_place() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        struct Tracked {
            counter: Arc<AtomicUsize>,
        }
        impl Drop for Tracked {
            fn drop(&mut self) {
                self.counter.fetch_add(1, Ordering::SeqCst);
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));
        {
            let _pb = PageBox::new(Tracked {
                counter: Arc::clone(&counter),
            });
        }
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
