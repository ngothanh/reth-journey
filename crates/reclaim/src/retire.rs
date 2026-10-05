//! What a retirable object owes the reclamation scheme.
//!
//! An object that can be handed to a scheme carries two things: an intrusive
//! list link, so the scheme can thread retired objects together without
//! allocating, and a way to free *itself*, because not every object came from a
//! `Box`.
//!
//! # Why these signatures
//!
//! **`Retire: Send`.** `retire(p)` transfers ownership to the scheme, and the
//! thread that eventually runs [`Retire::reclaim`] is not the thread that called
//! `retire` — it is whichever thread crosses the reclamation threshold, or a
//! dedicated reclaimer thread that outlives the producer entirely. The object
//! crosses a thread boundary between retirement and drop, and `Send` is the
//! license to do that. Without the bound, an object holding an `Rc` would have
//! its non-atomic refcount decremented on the reclaimer while being cloned on
//! the producer.
//!
//! **`reclaim(ptr: *mut Self)`, not `&mut self` or `Box<Self>`.** Each
//! alternative states a guarantee this function has to break. `&mut self`
//! promises the object stays alive for the reference's lifetime, while
//! `reclaim`'s whole job is to deallocate it — freeing memory a live `&mut`
//! points into is what Miri's Stacked Borrows flags. `Box<Self>` promises the
//! allocation came from the global allocator, which is exactly the promise this
//! trait exists to deny. A raw pointer promises nothing, which is the only
//! honest option for a function whose purpose is to end the object; every
//! obligation is therefore stated below rather than implied by a type.
//!
//! Taking no `self` also makes it an associated function, which is what lets a
//! scheme hold a plain `(ptr, reclaim_fn)` pair and never need `dyn Retire` —
//! monomorphisation turns `T::reclaim` into a function pointer at the retire
//! site.
//!
//! **`RetireLink::next` is `AtomicPtr<()>`, not `AtomicPtr<Self>`.** A domain's
//! retired list is a *single* heterogeneous chain: a queue segment, a stack
//! node, a pool page and a price level all hang off it in arrival order. Typed,
//! a segment's `next` could only point at another segment, forcing one list,
//! one threshold and one scan per type — defeating the point of a domain. By
//! the time an object is retired its static type is gone; what survives is an
//! address plus a function that knows how to free it. Erase the type from the
//! data, keep it in the code. A thin `*mut ()` also fits in an atomic word,
//! which a fat pointer would not.
//!
//! **`reclaim` has no default body.** A `Box::from_raw` default would be the
//! common case and silently wrong for the two consumers this trait was designed
//! for: a pool-allocated page whose implementor forgets to override it gets
//! `Box::from_raw` on memory the global allocator never handed out — undefined
//! behaviour, on a reclaimer thread, minutes later. Same argument that rejected
//! `SegQueue<T, R = Leak>`: a bad default is worse than no default, because it
//! is silently wrong rather than loudly absent.

use crate::sync::AtomicPtr;
use core::ptr::null_mut;

/// The intrusive link a scheme threads retired objects through.
///
/// One per retirable object, owned by that object. The storage is the object's;
/// the *contents* belong to the scheme for as long as the object is retired —
/// see [`Retire::retire_link`].
pub struct RetireLink {
    // Written at construction, read by the scheme's drain. C2 is the first
    // reader; drop the `allow` there.
    #[allow(dead_code)]
    next: AtomicPtr<()>,
}

impl RetireLink {
    /// A link that is not currently on any retired list.
    //
    // Not `const fn`: loom's atomics have no const constructor, and this type
    // is built from both the real and the loom shim.
    #[must_use]
    pub fn new() -> Self {
        RetireLink { next: AtomicPtr::new(null_mut()) }
    }
}

impl Default for RetireLink {
    fn default() -> Self {
        Self::new()
    }
}

/// An object a reclamation scheme can take ownership of and later free.
///
/// # Safety
///
/// This trait is `unsafe` to implement because a scheme calls both methods from
/// its own **safe** code and relies on what they do for memory safety. A
/// well-meaning safe implementation can corrupt the retired list, so the
/// obligations below cannot be checked by the compiler and are not optional.
///
/// Throughout, *the retired window* means the interval from the moment
/// `retire(p)` is called until [`reclaim`](Retire::reclaim) returns.
///
/// ## [`retire_link`](Retire::retire_link) promises four things
///
/// 1. **Ownership** — the returned link's storage belongs to this object and to
///    nothing else. Two objects sharing one link overwrite each other's `next`,
///    which both loses the tail of the chain and makes the drain revisit a
///    freed object: use-after-free, then double free.
/// 2. **Determinism** — the *same* link on every call. An implementation that
///    picks between two fields on some condition will be written through one
///    and read through the other, so the chain behind this object is lost.
/// 3. **Stability** — the link's address does not move for the whole retired
///    window. The scheme stores a pointer *into* the link; relocating it — by
///    replacing a `Box` field that contains it, say — leaves the scheme holding
///    a dangling pointer into a live object.
/// 4. **Non-interference** — the link's *contents* are the scheme's for that
///    same window. Do not read or write `next` while the object is retired;
///    overwriting it drops the rest of the chain on the floor.
///
/// Note that 3 and 4 share one window, and that interior mutability is the only
/// way to violate either through `&self`.
///
/// ## [`reclaim`](Retire::reclaim) promises three more
///
/// 5. **It actually frees.** Returning without releasing the object is a leak,
///    and a scheme's bounded-garbage guarantee is a claim about the scheme, not
///    a promise it can keep on an implementor's behalf.
/// 6. **It does not resurrect.** The pointer must not be published anywhere a
///    reader could reach it. Resurrection reintroduces from the inside the
///    double-free that retiring twice was forbidden to cause.
/// 7. **It does not unwind, and does not re-enter the scheme.** Reclamation runs
///    inside a drain loop, often on a shared reclaimer thread; a panic there
///    takes down work that has nothing to do with this object.
pub unsafe trait Retire: Send {
    /// This object's intrusive link.
    ///
    /// Must satisfy properties 1–4 of the trait's safety contract: the same
    /// link, at a stable address, owned solely by `self`, whose contents are
    /// left alone for the retired window.
    fn retire_link(&self) -> &RetireLink;

    /// Free this object, once the grace period has passed.
    ///
    /// Implement it to match however the object was allocated — `Box`, a pool,
    /// an arena. There is deliberately no default; see the module docs.
    ///
    /// # Safety
    ///
    /// The caller — in practice the scheme, never user code — must ensure:
    ///
    /// - `ptr` is non-null, well aligned, and points to a live `Self` that was
    ///   allocated the way this implementation expects to free;
    /// - the grace period has elapsed, so no thread can still be reading it.
    ///   Nothing is racing this call by construction, which is the entire thing
    ///   the scheme buys;
    /// - `ptr` is reclaimed exactly once, and the caller does not touch `*ptr`
    ///   afterwards.
    unsafe fn reclaim(ptr: *mut Self);
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// A retirable object whose `reclaim` is *not* a plain `Box` free — it
    /// reports back first, which is the whole reason `reclaim` is overridable.
    struct Node {
        link: RetireLink,
        reclaimed: Arc<AtomicUsize>,
    }

    unsafe impl Retire for Node {
        fn retire_link(&self) -> &RetireLink {
            &self.link
        }

        unsafe fn reclaim(ptr: *mut Self) {
            let node = unsafe { Box::from_raw(ptr) };
            node.reclaimed.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn node(counter: &Arc<AtomicUsize>) -> Box<Node> {
        Box::new(Node { link: RetireLink::new(), reclaimed: Arc::clone(counter) })
    }

    #[test]
    fn retire_link_is_deterministic_and_belongs_to_self() {
        let counter = Arc::new(AtomicUsize::new(0));
        let n = node(&counter);

        let first: *const RetireLink = n.retire_link();
        let second: *const RetireLink = n.retire_link();
        assert_eq!(first, second, "property 2: the same link on every call");
        assert!(
            core::ptr::eq(n.retire_link(), &n.link),
            "property 1: the link is this object's own field"
        );

        unsafe { Node::reclaim(Box::into_raw(n)) };
    }

    #[test]
    fn reclaim_runs_the_implementors_deallocator_exactly_once() {
        let counter = Arc::new(AtomicUsize::new(0));
        let ptr = Box::into_raw(node(&counter));

        unsafe { Node::reclaim(ptr) };

        assert_eq!(
            counter.load(Ordering::Relaxed),
            1,
            "property 5: the object's own reclaim ran, exactly once"
        );
    }
}
