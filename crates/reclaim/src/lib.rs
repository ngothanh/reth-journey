mod domain;
mod guard;
mod leak;
mod reclaimer;
mod retire;
mod root;
#[allow(unused_imports)]
mod sync;

#[cfg(test)]
mod test_support;

pub use domain::*;
pub use guard::*;
pub use leak::*;
pub use reclaimer::*;
pub use retire::*;
pub use root::*;

/// Crate-private types, exposed only so the benches can measure them.
/// Not part of the public API: built only with the `bench-internals` feature.
#[cfg(feature = "bench-internals")]
#[doc(hidden)]
pub mod bench_internals {
    use crate::{Retirable, RetireLink};

    /// The pile, wrapped so its methods can be called from outside the crate.
    pub struct RetiredList(crate::retire::RetiredList);

    impl RetiredList {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            RetiredList(crate::retire::RetiredList::new())
        }

        /// # Safety
        /// The same promises as the crate's `RetiredList::push`.
        #[inline]
        pub unsafe fn push<T: Retirable>(&self, obj: *mut T) {
            // SAFETY: the caller keeps `push`'s promises.
            unsafe { self.0.push(obj) }
        }

        #[inline]
        pub fn take_all(&self) -> *mut RetireLink {
            self.0.take_all()
        }

        /// # Safety
        /// The same promises as the crate's `RetiredList::free_all`.
        pub unsafe fn free_all(link: *mut RetireLink) {
            // SAFETY: the caller keeps `free_all`'s promises.
            unsafe { crate::retire::RetiredList::free_all(link) }
        }
    }

    /// The pile split over 8 heads, wrapped the same way.
    pub struct ShardedList(crate::retire::ShardedList);

    impl ShardedList {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            ShardedList(crate::retire::ShardedList::new())
        }

        /// # Safety
        /// The same promises as the crate's `RetiredList::push`.
        #[inline]
        pub unsafe fn push<T: Retirable>(&self, obj: *mut T) {
            // SAFETY: the caller keeps `push`'s promises.
            unsafe { self.0.push(obj) }
        }

        pub fn take_all(&self) -> [*mut RetireLink; 8] {
            self.0.take_all()
        }
    }
}
