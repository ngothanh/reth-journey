//! Cache-line padding to prevent false sharing.
//!
//! [`CachePadded<T>`] aligns `T` to the target's cache-line size and pads its
//! total size up to a multiple of that line. Two adjacent `CachePadded<T>`
//! therefore never share a cache line, eliminating MESI ping-pong on
//! cross-thread writes to neighboring atomics (Vyukov MPMC head/tail,
//! Disruptor cursor pairs, matching-engine best-bid/ask).
//!
//! ## Cache-line policy
//!
//! - `aarch64` (incl. Apple M-series): **128 B**. The L2 prefetcher pulls
//!   128-byte pairs, so 64 B alignment still false-shares adjacent counters.
//! - `x86_64`: **64 B**. Standard cache-line size.
//! - other targets: **64 B** (conservative default).
//!
//! `repr(align(N))` requires a literal `N`, so the per-target value is
//! selected with stacked `#[cfg_attr]` arms rather than a `const`.
//!
//! ## Pitfall — `repr(C)` on the outer struct
//!
//! When a `CachePadded<T>` lives inside another struct, that **outer struct
//! must be `#[repr(C)]`**. `repr(Rust)` is free to reorder fields, and may
//! shuffle the alignment-induced padding away from where you intended it —
//! false sharing returns silently, caught only by a contended benchmark.
//!
//! ```ignore
//! #[repr(C)]                                  // <-- required
//! struct MpmcRing<T> {
//!     head: CachePadded<AtomicUsize>,         // hot for producers
//!     tail: CachePadded<AtomicUsize>,         // hot for consumers
//!     ring: *mut [T],
//! }
//! ```

use std::fmt;
use std::ops::{Deref, DerefMut};

#[cfg_attr(
    any(target_arch = "aarch64", target_arch = "powerpc64"),
    repr(align(128))
)]
#[cfg_attr(target_arch = "x86_64", repr(align(64)))]
#[cfg_attr(
    not(any(
        target_arch = "aarch64",
        target_arch = "powerpc64",
        target_arch = "x86_64",
    )),
    repr(align(64))
)]
#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CachePadded<T> {
    value: T,
}

impl<T> CachePadded<T> {
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Deref for CachePadded<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for CachePadded<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> From<T> for CachePadded<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: fmt::Debug> fmt::Debug for CachePadded<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CachePadded")
            .field("value", &self.value)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};
    use std::sync::atomic::AtomicU64;

    #[cfg(any(target_arch = "aarch64", target_arch = "powerpc64"))]
    const LINE: usize = 128;
    #[cfg(not(any(target_arch = "aarch64", target_arch = "powerpc64")))]
    const LINE: usize = 64;

    #[test]
    fn align_matches_cache_line() {
        assert_eq!(align_of::<CachePadded<u8>>(), LINE);
        assert_eq!(align_of::<CachePadded<AtomicU64>>(), LINE);
    }

    #[test]
    fn size_is_padded_to_line_multiple() {
        // A single-byte payload still rounds up to a full cache line.
        assert_eq!(size_of::<CachePadded<u8>>(), LINE);
        // An 8 B atomic also rounds up.
        assert_eq!(size_of::<CachePadded<AtomicU64>>(), LINE);
    }

    #[test]
    fn adjacent_paddeds_are_on_separate_lines() {
        let pair: [CachePadded<u64>; 2] = [CachePadded::new(0), CachePadded::new(0)];
        let a = (&*pair[0]) as *const u64 as usize;
        let b = (&*pair[1]) as *const u64 as usize;
        assert!(
            b - a >= LINE,
            "adjacent payloads share a cache line: a={a:#x} b={b:#x} stride={}",
            b - a
        );
    }

    #[test]
    fn deref_gives_transparent_access() {
        let cell = CachePadded::new(42u32);
        assert_eq!(*cell, 42);

        let mut cell = CachePadded::new(0u32);
        *cell = 7;
        assert_eq!(*cell, 7);
    }
}