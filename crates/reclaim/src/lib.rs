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
mod registry;

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

    /// A chain of retired objects, wrapped like the piles below.
    pub struct Chain(crate::retire::Chain);

    impl Chain {
        #[allow(clippy::new_without_default)]
        #[inline]
        pub fn new() -> Self {
            Chain(crate::retire::Chain::new())
        }

        /// # Safety
        /// The same promises as the crate's `Chain::add`.
        #[inline]
        pub unsafe fn add<T: Retirable>(&mut self, obj: *mut T) {
            // SAFETY: the caller keeps `add`'s promises.
            unsafe { self.0.add(obj) }
        }
    }

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
        pub fn push_chain(&self, chain: Chain) {
            self.0.push_chain(chain.0)
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

        #[inline]
        pub fn push_chain(&self, chain: Chain) {
            self.0.push_chain(chain.0)
        }

        pub fn take_all(&self) -> [*mut RetireLink; 8] {
            self.0.take_all()
        }
    }
}
