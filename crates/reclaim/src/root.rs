//! Declared roots.
//!
//! A **root** is an atomic a reader starts from: the place it loads a pointer
//! out of before protecting it. The set of atomics ever passed to `try_protect`
//! is the root set, and `retire(p)` is sound once `p` is unreachable from it.
//!
//! The set used to be implicit — whatever a client happened to pass to
//! `protect` — which made requirement (A) impossible to check. Declaring roots
//! makes the set enumerable, so a debug build can walk it.
//!
//! # The two constructors live in two places, on purpose
//!
//! ```text
//! Domain::declare_root(&self, src)  -> Root<T>   domain.rs, edit C1.3
//!     takes &self, so it REGISTERS with that domain.
//!     retire() walks the registered roots in debug and checks (A) for you.
//!
//! Root::assume_root(src)            -> Root<T>   here, edit C1.2
//!     no self, so it registers NOWHERE.
//!     the caller owes requirement (A) unaided, with an unsafe block and a
//!     comment at the call site.
//! ```
//!
//! That `&self` is the entire difference between them. `declare_root` is not
//! in this file because it is a method on `Domain`, which C1.3 introduces —
//! see the C1 piece table in `plan/reclaim_ladder.md`.

use crate::sync::AtomicPtr;

/// An atomic that has been declared a root.
///
/// One word, `Copy`, no `Drop` — a data structure stores one per root next to
/// the field it points at (`SegQueue` holds two, for `head` and `tail`), and
/// that has to cost nothing in a `#[repr(C)]` struct whose layout is tuned.
///
/// The "`src` outlives the domain" obligation lives in the `# Safety` block of
/// whichever constructor produced this value, not in a lifetime parameter:
/// a lifetime would make the holder self-referential, since the root borrows
/// from a sibling field of the same struct.
pub struct Root<T> {
    // Read by `as_atomic` / `addr`, which have no caller in the library until
    // C1.3's `declare_root` and C1.4's `try_protect`. Drop the `allow` there.
    src: *const AtomicPtr<T>,
}

// Root carries an address and nothing else; the thing it addresses is an
// AtomicPtr, which is already Send + Sync. Keeping the raw pointer would
// otherwise make Root !Send + !Sync and infect every structure that stores one.
unsafe impl<T> Send for Root<T> {}
unsafe impl<T> Sync for Root<T> {}

// Hand-written rather than derived: `derive` would add a `T: Clone` / `T: Copy`
// bound that is not needed, since `*const _` is `Copy` for every `T`.
impl<T> Clone for Root<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Root<T> {}

#[allow(dead_code)] // until C1.3 / C1.4 call these
impl<T> Root<T> {
    /// Treat `src` as a root without registering it with any domain.
    ///
    /// The escape hatch for a structure whose root set is not statically
    /// enumerable — a traversal that protects through a different atomic per
    /// node, say. This is the documented-roots regime, available per call site
    /// instead of crate-wide.
    ///
    /// # Safety
    ///
    /// - `src` must stay live for as long as any domain holds this root, and
    /// - the caller takes on **requirement (A)** unaided: this root is
    ///   registered nowhere, so the debug reachability walk cannot see it and
    ///   will not check (A) on the caller's behalf. Prefer
    ///   `Domain::declare_root` wherever the root set *can* be enumerated.
    #[must_use]
    pub unsafe fn assume_root(src: &AtomicPtr<T>) -> Self {
        Root { src }
    }

    /// The atomic this root names.
    ///
    /// Safe because `Root` can only be built by one of the two `unsafe`
    /// constructors, both of which require `src` to outlive the domain. The
    /// type carries that invariant, so every root-dangling bug traces back to
    /// a constructor rather than to this dereference.
    pub(crate) fn as_atomic(&self) -> &AtomicPtr<T> {
        unsafe { &*self.src }
    }

    /// The address of the atomic — what the debug reachability walk compares.
    pub(crate) fn addr(&self) -> usize {
        self.src as usize
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    // TODO(you): two things worth pinning here, and the second is the one that
    // protects a tuned layout.
    //
    //   - assume_root names the atomic it was given (as_atomic / addr agree)
    //   - size_of::<Root<T>>() == size_of::<usize>()
    //
    // The second is a real regression test, not a tautology: it is what breaks
    // if Root ever grows a Drop, an Arc or a domain id.
}
