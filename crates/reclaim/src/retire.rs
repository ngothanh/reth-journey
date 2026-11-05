use crate::sync::AtomicPtr;
use crate::sync::Ordering;
use crate::sync::UnsafeCell;
use concurrent_utils::CachePadded;
use core::ptr::null_mut;

/// The link stored inside every retirable object.
///
/// The field belongs to the object. The value in it belongs to the scheme.
pub struct RetireLink {
    pub(crate) next: AtomicPtr<RetireLink>,
    obj: UnsafeCell<*mut ()>,
    reclaim: UnsafeCell<Option<unsafe fn(*mut ())>>,
}

/// The pile: retired objects waiting to be freed, chained through their links.
pub(crate) struct RetiredList {
    head: AtomicPtr<RetireLink>,
}

/// How many piles a `ShardedList` splits retired objects over. A power of two,
/// so picking one is a mask, not a division. Same number as folly's
/// `hazptr_domain` (`kNumShards`).
const SHARDS: usize = 8;

/// How many low bits of an object's address `shard_of` drops before picking a
/// shard. The allocator aligns objects, so the lowest bits are the same for
/// almost every object: `addr & 7` would send every 8-byte-aligned object to
/// shard 0. Dropping 8 bits skips them; objects within 256 bytes of each other
/// share a shard, objects further apart spread out. Same as folly's
/// `kIgnoredLowBits`.
const IGNORE_LOW_BITS: u32 = 8;

/// The pile split into `SHARDS` piles, so threads that retire at the same time
/// usually exchange on different `head`s instead of all on one.
///
/// Each shard has its own cache line (`CachePadded`). Without the padding all
/// 8 `head`s fit in one 128-byte line, and the line still travels between cores on almost every
/// push (L18 measured it: 8 threads were 4–5× better than one head, but still
/// ≈ 10× below one thread).
pub(crate) struct ShardedList {
    shards: [CachePadded<RetiredList>; SHARDS],
}

impl ShardedList {
    pub(crate) fn new() -> Self {
        ShardedList {
            shards: std::array::from_fn(|_| CachePadded::new(RetiredList::new())),
        }
    }

    /// Which shard an object goes to. Depends only on its address, so the
    /// same object always picks the same shard.
    fn shard_of(addr: *mut ()) -> usize {
        ((addr as usize) >> IGNORE_LOW_BITS) & (SHARDS - 1)
    }

    /// Puts one retired object on the shard its address picks.
    ///
    /// # Safety
    ///
    /// The same promises as `RetiredList::push`.
    pub(crate) unsafe fn push<T: Retirable>(&self, obj: *mut T) {
        let shard = &self.shards[Self::shard_of(obj.cast())];
        // SAFETY: the caller keeps `RetiredList::push`'s promises; picking a
        // shard changes none of them.
        unsafe { shard.push(obj) }
    }

    /// Takes every shard's chain in one call. Each entry is a chain for
    /// `RetiredList::free_all`, or null if that shard was empty.
    pub(crate) fn take_all(&self) -> [*mut RetireLink; SHARDS] {
        std::array::from_fn(|i| self.shards[i].take_all())
    }
}

/// An object that a scheme can take over and free later.
///
/// Implemented by the things a structure hands to `retire`: a queue segment, a
/// stack node, a page from a pool.
///
/// # Safety
///
/// Whoever implements this trait promises:
///
/// 1. `retire_link` returns a link that is a field of this same object. It is
///    not shared with any other object.
/// 2. `retire_link` returns the same link every time it is called.
/// 3. While the object is retired, the object does not read or write the link.
///    The scheme uses it to chain retired objects together.
/// 4. `reclaim` really frees the object, in the way it was allocated: `Box`,
///    a pool, an arena. It does not keep the pointer or make it reachable again.
///
/// The scheme relies on all four. If one is broken, the scheme can lose
/// objects or free one twice, even though the scheme itself is correct.
pub unsafe trait Retirable: Send {
    /// The link the scheme uses to chain this object into its list.
    fn retire_link(&self) -> &RetireLink;

    /// Frees the object. Called by the scheme, not by the structure.
    ///
    /// The scheme decides *when*; this function knows *how*.
    ///
    /// # Safety
    ///
    /// The caller promises:
    ///
    /// - `ptr` points to a live object of this type that was handed to `retire`;
    /// -  no reader holds the object anymore;
    /// - `reclaim` is called once for this object, and `ptr` is not used after.
    unsafe fn reclaim(ptr: *mut Self);
}

impl RetireLink {
    /// A link for an object that has not been retired.
    pub fn new() -> Self {
        RetireLink {
            next: AtomicPtr::new(null_mut()),
            obj: UnsafeCell::new(null_mut()),
            reclaim: UnsafeCell::new(None),
        }
    }
}

/// Turns the type-free address back into a `*mut T` and frees it the way `T` says.
///
/// # Safety
/// `obj` came from a `*mut T` handed to `push`, and the object can be freed now.
unsafe fn reclaim_one<T: Retirable>(obj: *mut ()) {
    // SAFETY: the caller promises `obj` came from a `*mut T` handed to
    // `push`, so the cast gives back the same object, and that it can be
    // freed now.
    unsafe { T::reclaim(obj.cast::<T>()) }
}

impl RetiredList {
    pub(crate) fn new() -> Self {
        RetiredList {
            head: AtomicPtr::new(null_mut()),
        }
    }

    /// Takes every link out of the pile in one step. The caller now owns the
    /// whole chain; the pile is empty.
    pub(crate) fn take_all(&self) -> *mut RetireLink {
        self.head.swap(null_mut(), Ordering::Acquire)
    }

    /// Puts one retired object on the pile.
    ///
    /// # Safety
    ///
    /// The caller promises:
    ///
    /// - `obj` points to a live object;
    /// - `obj` is pushed only once, so it is not in any pile already;
    /// - the caller does not use `obj` after this call.
    ///
    /// # Panics
    ///
    /// If the object is already in a pile. This catches the same object
    /// retired twice when the two retires are ordered (same thread, or a lock
    /// or `join` between them). Two retires of one object at the same moment
    /// are not caught here; under loom they show up as a causality violation.
    pub(crate) unsafe fn push<T: Retirable>(&self, obj: *mut T) {
        // SAFETY: the caller promises `obj` points to a live object.
        let link: &RetireLink = unsafe { (*obj).retire_link() };
        let link_ptr = link as *const RetireLink as *mut RetireLink;

        // A link in no pile has `reclaim == None`: `RetireLink::new` starts it
        // that way and `free_all` puts it back. `Some` means it is in a pile.
        //
        // `assert!`, not `debug_assert!`: the read is one plain load from the
        // cache line the next lines write anyway, and a second push would turn
        // the chain into a cycle, a double free far from the line at fault.
        //
        // SAFETY: if the caller keeps its promise, no other thread can reach
        // the link (it is in no pile), so nobody writes this cell now.
        let in_a_pile = link.reclaim.with(|p| unsafe { (*p).is_some() });
        assert!(
            !in_a_pile,
            "object retired twice: the structure called `retire` on the same \
             object more than once"
        );

        // SAFETY: the link is not in the pile yet (the caller pushes `obj`
        // only once), so no other thread can reach these two cells. They are
        // written before the exchange below makes the link visible.
        link.obj.with_mut(|p| unsafe { *p = obj.cast() });
        link.reclaim
            .with_mut(|p| unsafe { *p = Some(reclaim_one::<T>) });

        let mut top = self.head.load(Ordering::Relaxed);
        loop {
            // Point the new link at the current top *before* the exchange:
            // once `head` holds the link, another thread can take it.
            link.next.store(top, Ordering::Relaxed);
            match self.head.compare_exchange_weak(
                top,
                link_ptr,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(current) => top = current,
            }
        }
    }

    /// Frees every object in a chain that `take_all` handed out.
    ///
    /// # Safety
    ///
    /// The caller promises:
    ///
    /// - `link` is null, or the start of a chain that this thread got from
    ///   `take_all`. No other thread has the chain, and it is not used after
    ///   this call;
    /// - no reader holds any object in the chain anymore, so every one of
    ///   them can be freed now.
    pub(crate) unsafe fn free_all(mut link: *mut RetireLink) {
        while !link.is_null() {
            // SAFETY: `link` is not null, and it is in the chain the caller
            // owns. Its object is still alive: objects are freed only at the
            // last step of this loop, one link at a time, after that link's
            // `next` has been read.
            let next = unsafe { (*link).next.load(Ordering::Relaxed) };

            // SAFETY: the same link, still alive. Reading the two cells is
            // safe because:
            // 1. only this thread holds the chain, so nobody writes them now;
            // 2. `push` wrote them before its `Release` exchange, and the
            //    `Acquire` swap in `take_all` makes those writes visible here.
            // Every link in a chain from `take_all` went through `push`, which
            // set `reclaim` to `Some`, so `unwrap` cannot fail.
            let obj = unsafe { (*link).obj.with(|p| *p) };
            let reclaim = unsafe { (*link).reclaim.with(|p| *p).unwrap() };

            // Give the link back empty, as `RetireLink::new` made it, so an
            // object that `reclaim` returns to a pool can be retired again
            // without tripping the check in `push`. This must happen before
            // `reclaim`: after it, the link is no longer ours to touch.
            //
            // SAFETY: the same link, still alive, and only this thread holds it.
            unsafe { (*link).obj.with_mut(|p| *p = null_mut()) };
            unsafe { (*link).reclaim.with_mut(|p| *p = None) };

            // SAFETY:
            // 1. `obj` and `reclaim` were written together by `push::<T>`:
            //    `obj` is a `*mut T` with its type removed, and `reclaim` is
            //    `reclaim_one::<T>` for that same `T`, so the cast back is right;
            // 2. the caller promises no reader holds the object any more;
            // 3. each link is in the chain once, so each object is freed once;
            // 4. `next` was read above, and `link` is not used after this call
            //    (the link was inside the object, so it is gone now).
            unsafe { reclaim(obj) }

            link = next;
        }
    }
}

// SAFETY: handing the link to another thread hands over only the address in obj.
// The object it points to stays where it is.
// The other thread uses the address only to free that object,
// and that is allowed, because the object is Send (Retirable: Send).
unsafe impl Send for RetireLink {}

// SAFETY:
// 1. next is atomic, so sharing it is safe on its own.
// 2. obj and reclaim are written once, by the thread calling retire, and only before the exchange on head makes the link visible.
// No second writer is possible, because retire promises the object is retired only once.
// 3. They are read only by the thread that took the link out of the pile, and only after it took it out.
// 4. The exchange (Release) and the take-out (Acquire) make the write in 2 visible to the read in 3.
// 5. While the object is retired, the object itself does not touch the link (promise 3 of Retirable).
unsafe impl Sync for RetireLink {}

impl Default for RetireLink {
    fn default() -> Self {
        RetireLink::new()
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use crate::test_support::{Arc, Node, PoolSlot, Pooled};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    #[test]
    fn a_new_link_is_not_in_the_pile() {
        assert!(RetireLink::new().next.load(Ordering::Relaxed).is_null());
        assert!(RetireLink::default().next.load(Ordering::Relaxed).is_null());
    }

    #[test]
    fn retire_link_is_the_nodes_own_field() {
        let drops = Arc::new(AtomicUsize::new(0));
        let node = Node::boxed(16, &drops);

        assert!(core::ptr::eq(node.retire_link(), &node.link));
        assert_eq!(node.data, 16);
    }

    #[test]
    fn retire_link_is_the_same_every_call() {
        let drops = Arc::new(AtomicUsize::new(0));
        let node = Node::boxed(16, &drops);

        let first: *const RetireLink = node.retire_link();
        let second: *const RetireLink = node.retire_link();
        assert_eq!(first, second);
    }

    #[test]
    fn reclaim_frees_the_node_once() {
        let drops = Arc::new(AtomicUsize::new(0));
        let ptr: *mut Node = Box::into_raw(Node::boxed(16, &drops));
        assert_eq!(drops.load(Ordering::Relaxed), 0, "not freed yet");

        unsafe {
            Node::reclaim(ptr);
        }

        assert_eq!(drops.load(Ordering::Relaxed), 1, "freed exactly once");
    }

    #[test]
    fn retire_link_is_3_words() {
        assert_eq!(size_of::<RetireLink>(), 3 * size_of::<usize>());
    }

    #[test]
    fn a_new_pile_is_empty() {
        assert!(RetiredList::new().head.load(Ordering::Relaxed).is_null());
    }

    #[test]
    fn take_all_on_an_empty_pile_is_null() {
        assert!(RetiredList::new().take_all().is_null());
    }

    #[test]
    fn push_one_then_take_all_gives_its_link() {
        let drop_counter = Arc::new(AtomicUsize::new(0));
        let ptr = Box::into_raw(Node::boxed(16, &drop_counter));
        let list = RetiredList::new();
        unsafe { list.push(ptr) };
        let link = unsafe { &raw mut (*ptr).link };
        assert_eq!(list.take_all(), link);
        assert_eq!(list.take_all(), null_mut());
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }

    #[test]
    fn push_two_newest_is_on_top() {
        let drop_counter = Arc::new(AtomicUsize::new(0));
        let a = Box::into_raw(Node::boxed(1, &drop_counter));
        let b = Box::into_raw(Node::boxed(2, &drop_counter));

        let list = RetiredList::new();
        unsafe { list.push(a) };
        unsafe { list.push(b) };

        let a_link = unsafe { &raw mut (*a).link };
        let b_link = unsafe { &raw mut (*b).link };
        assert_eq!(list.take_all(), b_link);
        assert_eq!(unsafe { (*b_link).next.load(Ordering::Relaxed) }, a_link);
        assert_eq!(
            unsafe { (*a_link).next.load(Ordering::Relaxed) },
            null_mut()
        );

        unsafe {
            drop(Box::from_raw(a));
            drop(Box::from_raw(b));
        }
    }

    #[test]
    fn free_all_on_null_does_nothing() {
        let drops = Arc::new(AtomicUsize::new(0));
        let ptr = Box::into_raw(Node::boxed(1, &drops));
        let list = RetiredList::new();
        unsafe { list.push(ptr) };

        unsafe { RetiredList::free_all(null_mut()) };

        assert_eq!(drops.load(Ordering::Relaxed), 0, "nothing was freed");
        let chain = list.take_all();
        assert_eq!(
            chain,
            unsafe { &raw mut (*ptr).link },
            "the node is still in the pile"
        );

        unsafe { RetiredList::free_all(chain) };
        assert_eq!(drops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn free_all_frees_every_boxed_node() {
        let drops = Arc::new(AtomicUsize::new(0));
        let list = RetiredList::new();
        for data in 0..3 {
            let ptr = Box::into_raw(Node::boxed(data, &drops));
            unsafe { list.push(ptr) };
        }
        assert_eq!(drops.load(Ordering::Relaxed), 0, "push frees nothing");

        unsafe { RetiredList::free_all(list.take_all()) };

        assert_eq!(drops.load(Ordering::Relaxed), 3, "each node freed once");
        assert!(list.take_all().is_null(), "the pile is empty");
    }

    /// The second push panics before it writes anything, so the pile still
    /// holds the object exactly once and can free it normally. (Catching the
    /// panic instead of `#[should_panic]` lets the test free the node, so Miri
    /// sees no leak.)
    #[test]
    fn pushing_the_same_object_twice_panics() {
        let drops = Arc::new(AtomicUsize::new(0));
        let ptr = Box::into_raw(Node::boxed(1, &drops));
        let list = RetiredList::new();
        unsafe { list.push(ptr) };

        let second = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // SAFETY: none. This breaks `push`'s second promise on purpose.
            unsafe { list.push(ptr) };
        }));

        let message = *second.unwrap_err().downcast::<&str>().unwrap();
        assert!(message.starts_with("object retired twice"), "{message}");

        let chain = list.take_all();
        assert_eq!(chain, unsafe { &raw mut (*ptr).link });
        assert!(
            unsafe { (*chain).next.load(Ordering::Relaxed) }.is_null(),
            "the failed push did not touch the chain"
        );
        unsafe { RetiredList::free_all(chain) };
        assert_eq!(drops.load(Ordering::Relaxed), 1, "freed once");
    }

    /// A pooled object comes back from the pool and is retired again. That is
    /// correct use, so the double-retire check must stay quiet.
    #[test]
    fn a_pooled_object_can_be_retired_again() {
        let pool: Arc<Mutex<Vec<PoolSlot>>> = Arc::new(Mutex::new(Vec::new()));
        let ptr = Box::into_raw(Box::new(Pooled {
            link: RetireLink::new(),
            pool: Arc::clone(&pool),
        }));
        let list = RetiredList::new();

        for round in 0..2 {
            unsafe { list.push(ptr) };
            unsafe { RetiredList::free_all(list.take_all()) };
            let back = pool.lock().unwrap().pop();
            assert_eq!(back, Some(PoolSlot(ptr)), "round {round}: back in the pool");
        }

        unsafe { drop(Box::from_raw(ptr)) };
    }

    /// Real `Box` addresses, not made-up ones: the point of dropping the low
    /// bits is what the allocator really hands out.
    #[test]
    fn real_addresses_spread_over_every_shard() {
        const N: usize = 10_000;
        let drops = Arc::new(AtomicUsize::new(0));
        let nodes: Vec<*mut Node> = (0..N)
            .map(|i| Box::into_raw(Node::boxed(i as u64, &drops)))
            .collect();

        let mut per_shard = [0usize; SHARDS];
        for &node in &nodes {
            per_shard[ShardedList::shard_of(node.cast())] += 1;
        }

        for (shard, &count) in per_shard.iter().enumerate() {
            assert!(
                (N / 16..=N / 4).contains(&count),
                "shard {shard} got {count} of {N}: {per_shard:?}"
            );
        }

        for node in nodes {
            unsafe { drop(Box::from_raw(node)) };
        }
    }

    /// Without dropping the low bits, alignment alone decides the shard. The
    /// same addresses, picked with `addr & 7`, all land in one place.
    #[test]
    fn without_dropping_low_bits_every_object_lands_in_one_shard() {
        let drops = Arc::new(AtomicUsize::new(0));
        let nodes: Vec<*mut Node> = (0..1_000)
            .map(|i| Box::into_raw(Node::boxed(i, &drops)))
            .collect();

        assert!(nodes.iter().all(|&n| (n as usize) & (SHARDS - 1) == 0));

        for node in nodes {
            unsafe { drop(Box::from_raw(node)) };
        }
    }

    #[test]
    fn each_shard_has_its_own_cache_line() {
        let line = align_of::<CachePadded<()>>();
        let list = ShardedList::new();
        for pair in list.shards.windows(2) {
            let a = &pair[0].head as *const _ as usize;
            let b = &pair[1].head as *const _ as usize;
            assert_eq!(b - a, line, "neighbouring heads are one line apart");
        }
    }

    #[test]
    fn sharded_round_trip_frees_every_object_once() {
        const N: u64 = 100;
        let drops = Arc::new(AtomicUsize::new(0));
        let list = ShardedList::new();
        for i in 0..N {
            unsafe { list.push(Box::into_raw(Node::boxed(i, &drops))) };
        }

        let chains = list.take_all();
        assert!(
            chains.iter().filter(|c| !c.is_null()).count() > 1,
            "the objects went to more than one shard"
        );
        for chain in chains {
            unsafe { RetiredList::free_all(chain) };
        }

        assert_eq!(drops.load(Ordering::Relaxed), N as usize);
        assert!(list.take_all().iter().all(|c| c.is_null()), "every shard is empty");
    }

    #[test]
    fn each_kind_is_freed_its_own_way() {
        let drops = Arc::new(AtomicUsize::new(0));
        let pool: Arc<Mutex<Vec<PoolSlot>>> = Arc::new(Mutex::new(Vec::new()));
        let pooled = || {
            Box::into_raw(Box::new(Pooled {
                link: RetireLink::new(),
                pool: Arc::clone(&pool),
            }))
        };

        let list = RetiredList::new();
        let p1 = pooled();
        let p2 = pooled();
        // Mixed on purpose: Node, Pooled, Node, Pooled.
        unsafe {
            list.push(Box::into_raw(Node::boxed(1, &drops)));
            list.push(p1);
            list.push(Box::into_raw(Node::boxed(2, &drops)));
            list.push(p2);
        }

        unsafe { RetiredList::free_all(list.take_all()) };

        // Each `Node` went through `Box`: dropped once.
        assert_eq!(drops.load(Ordering::Relaxed), 2);
        // Each `Pooled` went back to the pool, not through `Box`.
        let mut back = pool.lock().unwrap().clone();
        back.sort();
        let mut expected = vec![PoolSlot(p1), PoolSlot(p2)];
        expected.sort();
        assert_eq!(back, expected);

        // The pool owns them now; empty it so Miri sees no leak.
        for PoolSlot(ptr) in pool.lock().unwrap().drain(..) {
            unsafe { drop(Box::from_raw(ptr)) };
        }
    }
}

#[cfg(all(test, loom))]
mod loom_tests {
    use crate::retire::RetiredList;
    use crate::sync::{AtomicUsize, Ordering};
    use crate::test_support::{Arc, Node};

    #[test]
    fn two_push_one_taker() {
        loom::model(|| {
            let drops = Arc::new(AtomicUsize::new(0));
            let list = Arc::new(RetiredList::new());

            let pushers = (0..2)
                .map(|i| {
                    let (list, drops) = (list.clone(), drops.clone());
                    loom::thread::spawn(move || unsafe {
                        list.push(Box::into_raw(Node::boxed(i, &drops)));
                    })
                })
                .collect::<Vec<_>>();

            unsafe {
                RetiredList::free_all(list.take_all());
            }

            for p in pushers {
                p.join().unwrap();
            }
            unsafe { RetiredList::free_all(list.take_all()) };
            assert_eq!(drops.load(Ordering::Relaxed), 2);
        })
    }
}
