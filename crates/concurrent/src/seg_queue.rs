//! Unbounded lock-free MPMC queue built from linked, write-once segments.
//!
//! Each segment is a fixed array of slots plus two cursors: `claimed` hands out
//! slots to producers, `consumed` hands them out to consumers. A slot is written
//! exactly once and read exactly once — segments are never reused — so a slot
//! needs only a one-shot `state` flag to bridge the gap between "a producer
//! reserved this index" and "the value is actually there", not the wrapping
//! sequence counter a reusable ring buffer requires.
//!
//! The queue's single responsibility is **rate**: absorbing the backlog between
//! producers and a momentarily slower consumer, losslessly and without a ceiling.
//! It deliberately does NOT own ordering beyond enqueue order (application order
//! belongs to a sequence number the caller assigns), admission control (that is a
//! soft cap the caller enforces), or durability.
//!
//! # Stage R0: segments are LEAKED, on purpose
//!
//! There is no `Drop` impl and nothing ever calls `Box::from_raw` on a retired
//! segment. **This is deliberate, not an oversight.** It isolates the queue
//! mechanics from the safe-memory-reclamation problem, which is a much harder
//! question attacked separately in the R1→R3 ladder (naive refcount, which fails;
//! then hazard pointers; then epoch-based reclamation).
//!
//! Two consequences to keep in mind while this stage stands:
//!
//! - **Memory grows with throughput, forever.** A `Segment<usize>` is roughly
//!   536 B (32 slots × 16 B, plus `next` and the two cursors), so a segment is
//!   retired every 32 pushes: ~16.8 MB/s leaked at 1M push/s, ~84 MB/s at 5M.
//! - **Values still in the queue when it is dropped are never dropped either**,
//!   since the slots holding them are never visited again.
//!
//! Until reclamation lands, Miri must be run with `-Zmiri-ignore-leaks`:
//!
//! ```text
//! MIRIFLAGS="-Zmiri-preemption-rate=0.5 -Zmiri-ignore-leaks" \
//!   cargo +nightly miri test -p concurrent --lib seg_queue
//! ```
//!
//! That flag is also the acceptance test for the next steps: once R1/R2/R3 frees
//! segments, **dropping `-Zmiri-ignore-leaks` is the proof that it works**.

use crate::CachePadded;
use reclaim::{Domain, Guard, Reclaimer, Retirable, RetireLink, Root};
use std::mem::MaybeUninit;
use std::ptr::null_mut;

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

#[cfg(not(loom))]
const SEG_LEN: usize = 32;

#[cfg(loom)]
const SEG_LEN: usize = 2;

const EMPTY: usize = 0;
const WRITTEN: usize = 1;

struct Slot<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    state: AtomicUsize,
}
#[repr(C)]
struct Segment<T> {
    slots: [Slot<T>; SEG_LEN],
    next: AtomicPtr<Segment<T>>,
    consumed: CachePadded<AtomicUsize>,
    claimed: AtomicUsize,
    link: RetireLink,
}

unsafe impl<T: Send> Retirable for Segment<T> {
    fn retire_link(&self) -> &RetireLink {
        &self.link
    }

    unsafe fn reclaim(ptr: *mut Self) {
        // SAFETY: `Box::from_raw` needs an address that came from a `Box` and
        // that nobody else uses anymore.
        // - From a `Box`: every segment is made with `Box::new`, in
        //   `SegQueue::new` and in `advance_tail`.
        // - Nobody else: the caller of `reclaim` promises that no reader holds
        //   the segment, that `reclaim` is called once for it, and that `ptr`
        //   is not used after wards.
        unsafe { drop(Box::from_raw(ptr)) };
    }
}

#[repr(C)]
pub struct SegQueue<T, R: Reclaimer> {
    head: *mut CachePadded<AtomicPtr<Segment<T>>>,
    tail: *mut CachePadded<AtomicPtr<Segment<T>>>,
    reclaim: CachePadded<AtomicPtr<Segment<T>>>,
    reclaiming: CachePadded<AtomicBool>,
    domain: Domain<R>,
    head_root: Root<Segment<T>>,
    tail_root: Root<Segment<T>>,
}

unsafe impl<T: Send, R: Reclaimer + Send + Sync> Send for SegQueue<T, R> {}
unsafe impl<T: Send, R: Reclaimer + Send + Sync> Sync for SegQueue<T, R> {}

impl<T> Segment<T> {
    fn new() -> Self {
        Segment {
            slots: std::array::from_fn(|_| Slot::new()),
            next: AtomicPtr::new(null_mut()),
            consumed: CachePadded::new(AtomicUsize::new(0)),
            claimed: AtomicUsize::new(0),
            link: RetireLink::new(),
        }
    }
}

impl<T> Slot<T> {
    fn new() -> Self {
        Slot {
            value: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicUsize::new(EMPTY),
        }
    }

    fn write_value(&self, value: T) {
        // SAFETY (both builds): writing through the cell needs that no other
        // thread reads or writes this slot at the same time.
        // - No other writer: only `push` calls this, with an index it got from
        //   `claimed.fetch_add`, and that hands out each index once.
        // - No reader: a popper reads a slot only after it sees `WRITTEN`, and
        //   `push` stores `WRITTEN` after this write.
        #[cfg(not(loom))]
        unsafe {
            (*self.value.get()).write(value);
        }
        #[cfg(loom)]
        self.value.with_mut(|ptr| unsafe { (*ptr).write(value) });
    }

    fn read_existing_value(&self) -> T {
        // SAFETY (both builds): `assume_init_read` needs a slot that holds a
        // value, and it must be read out only once.
        // - Holds a value: only `pop` calls this, after it loaded `WRITTEN`
        //   from this slot's `state` with `Acquire`, and `push` stores
        //   `WRITTEN` with `Release` after writing the value.
        // - Read once: `pop` calls this only when its exchange on `consumed`
        //   succeeded for this index, and that succeeds for one thread.
        #[cfg(not(loom))]
        unsafe {
            (*self.value.get()).assume_init_read()
        }

        #[cfg(loom)]
        self.value.with(|p| unsafe { (*p).assume_init_read() })
    }
}

impl<T: Send, R: Reclaimer> SegQueue<T, R> {
    fn head(&self) -> &AtomicPtr<Segment<T>> {
        // SAFETY: opening `self.head` needs a live `AtomicPtr` behind it.
        // The address came from `Box::into_raw` in `new` and is never changed.
        // The place is freed only in `Drop`, which takes `&mut self`, so it
        // cannot run while a `&self` call like this one is in progress.
        unsafe { &(*self.head) }
    }

    fn tail(&self) -> &AtomicPtr<Segment<T>> {
        // SAFETY: same as `head()`: from `Box::into_raw` in `new`, never
        // changed, freed only in `Drop`.
        unsafe { &(*self.tail) }
    }
    pub fn new(domain: Domain<R>) -> SegQueue<T, R> {
        let segment = Box::into_raw(Box::new(Segment::new()));
        let head = Box::into_raw(Box::new(CachePadded::new(AtomicPtr::new(segment))));
        let tail = Box::into_raw(Box::new(CachePadded::new(AtomicPtr::new(segment))));
        let reclaim = CachePadded::new(AtomicPtr::new(segment));

        // SAFETY: two `unsafe` operations here.
        // Opening `head`: it came from `Box::into_raw` two lines above.
        // `declare_root` asks that the place keeps its address and stays alive
        // until `remove_root`:
        // - Same address: the place is on the heap. The queue holds only its
        //   address, so moving the queue copies that address and the place
        //   itself does not move. The address is raw, not a `Box`, so moving
        //   the queue also does not cancel the root's right to use it.
        // - Alive: the queue never frees or replaces the place while it
        //   lives. `Drop` frees it, and removes the root before that.
        let head_root = unsafe { domain.declare_root(&*head) };

        // SAFETY: same as for `head`: `tail` is its own heap place from
        // `Box::into_raw`, held by raw address, and `Drop` removes its root
        // before freeing it.
        let tail_root = unsafe { domain.declare_root(&*tail) };

        SegQueue {
            head,
            tail,
            reclaim,
            reclaiming: CachePadded::new(AtomicBool::new(false)),
            domain,
            head_root,
            tail_root,
        }
    }

    pub fn push(&self, value: T) {
        let mut guard = self.domain.guard();
        loop {
            let ptr = guard.protect(&self.tail_root);

            // SAFETY: opening `ptr` needs two things.
            // - Not null: `tail` never holds null. `new` stores the first
            //   segment in it, and `advance_tail` only ever stores a non-null
            //   `next`.
            // - Not freed: `ptr` came from `guard.protect`, so the scheme does
            //   not free the segment while `guard` lives, and `guard` lives to
            //   the end of this function.
            let seg = unsafe { &*ptr };
            let idx = seg.claimed.fetch_add(1, Ordering::Relaxed);
            if idx < SEG_LEN {
                seg.slots[idx].write_value(value);
                seg.slots[idx].state.store(WRITTEN, Ordering::Release);
                return;
            }

            // SAFETY: `ptr` came from `guard.protect` above and `guard` is
            // still alive, which is what `advance_tail` asks of its caller.
            unsafe { self.advance_tail(ptr) };
        }
    }

    /// Makes sure `tail` no longer holds the full segment `cur_tail_ptr`.
    ///
    /// It links a new segment after it if nobody has yet, then tries to move
    /// `tail` forward. When it returns, `tail` has moved, by this thread or by
    /// another one. The caller protects through `tail_root` again.
    ///
    /// # Safety
    ///
    /// The caller promises that `cur_tail_ptr` came from `Guard::protect` on
    /// `tail_root`, and that the guard lives until this call returns.
    unsafe fn advance_tail(&self, cur_tail_ptr: *mut Segment<T>) {
        // SAFETY: the caller promises the segment is protected for this whole
        // call. It is not null, because `tail` never holds null.
        let cur_tail = unsafe { &*cur_tail_ptr };
        let mut next = cur_tail.next.load(Ordering::Acquire);
        if next.is_null() {
            let raw = Box::into_raw(Box::new(Segment::new()));
            match cur_tail.next.compare_exchange(
                null_mut(),
                raw,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => next = raw,
                Err(winner) => {
                    // SAFETY: `raw` came from `Box::into_raw` a few lines
                    // above. The exchange failed, so `raw` was never stored
                    // anywhere and no other thread has seen it.
                    drop(unsafe { Box::from_raw(raw) });
                    next = winner;
                }
            }
        }
        let _ =
            self.tail()
                .compare_exchange(cur_tail_ptr, next, Ordering::Release, Ordering::Relaxed);
    }

    /// Free segments that can no longer be reached, oldest first.
    ///
    /// Reclamation cannot be a per-guard decision. A guard holds only its own
    /// pointer, and the chain is forward-only, so "has my predecessor been freed?"
    /// is unanswerable from inside `Drop`. Instead the queue owns a cursor that
    /// trails `head`, and freeing is a strictly ordered walk from the front:
    ///
    /// - the first segment ever allocated has **no predecessor**, so once `head` is
    ///   past it and nobody holds it, *nothing* anywhere points to it — safe to free;
    /// - freeing it destroys the only remaining pointer to its successor, which makes
    ///   the successor safe in turn. Induction up the chain.
    ///
    /// That ordering is what makes a count of zero trustworthy: once the predecessor
    /// is gone, no thread can obtain a *new* pointer to this segment, so its count can
    /// only fall. A count that can rise again makes "I was the last holder" a
    /// statement about the present that gets acted on in the future.
    ///
    /// Serialised by `reclaiming`, because the cursor itself would otherwise race:
    /// two threads load the same `reclaim`, one frees it, the other then reads its
    /// fields. Note the cost — this reintroduces a serialisation point, which is the
    /// same trade the `Mutex<VecDeque>` baseline makes.
    ///
    /// Cost of the ordering: one thread sitting on an old segment blocks reclamation
    /// of *everything after it*, so memory grows until it lets go. That is the same
    /// pathology epoch-based reclamation has with a thread parked inside a pin.
    fn try_reclaim(&self) {
        if self.reclaiming.swap(true, Ordering::Acquire) {
            return;
        }

        loop {
            let seg = self.reclaim.load(Ordering::Relaxed);
            if seg == self.head().load(Ordering::Relaxed) {
                break;
            }
            if seg == self.tail().load(Ordering::Relaxed) {
                break;
            }

            // SAFETY: opening `seg` needs two things.
            // - Not null: `reclaim` starts at the first segment and is only
            //   ever given a non-null `next` (checked below).
            // - Not freed: a segment is retired only by the line at the end of
            //   this loop, after `reclaim` has moved past it. Only the thread
            //   that set `reclaiming` runs this loop, so the segment `reclaim`
            //   holds has not been retired yet, and a scheme frees only what
            //   was retired.
            let s = unsafe { &*seg };
            if s.consumed.load(Ordering::Relaxed) != SEG_LEN {
                break;
            }

            let next = s.next.load(Ordering::Acquire);
            if next.is_null() {
                break;
            }

            self.reclaim.store(next, Ordering::Relaxed);
            // SAFETY: the three promises of `retire`.
            // 1. No root holds `seg`.
            //    - `head`: checked at the top of this loop, and `head` only
            //      moves forward, so it never comes back to `seg`.
            //    - `tail`: checked at the top of this loop too. `tail` only
            //      moves forward, so once it has left `seg` it never comes
            //      back. It cannot be behind `seg` either: every earlier
            //      segment was retired by this loop, so `tail` had already
            //      left it.
            // 2. Retired once: `reclaim` moved past `seg` on the line above
            //    and only moves forward, and `reclaiming` lets one thread at a
            //    time run this loop.
            // 3. Not used after: `s`, the reference into `seg`, is not used
            //    below this line.
            unsafe {
                self.domain.retire(seg);
            }
        }

        self.reclaiming.store(false, Ordering::Release);
    }

    pub fn pop(&self) -> Option<T> {
        let mut guard = self.domain.guard();
        loop {
            let ptr = guard.protect(&self.head_root);
            // SAFETY: opening `ptr` needs two things.
            // - Not null: `head` never holds null. `new` stores the first
            //   segment in it, and the exchange below stores `next` only
            //   after checking that `next` is not null.
            // - Not freed: `ptr` came from `guard.protect`, so the scheme does
            //   not free the segment while `guard` lives, and `guard` lives to
            //   the end of this function.
            let seg = unsafe { &*ptr };
            let consuming = seg.consumed.load(Ordering::Relaxed);

            if consuming >= SEG_LEN {
                let next = seg.next.load(Ordering::Acquire);
                if next.is_null() {
                    return None;
                }
                let _ =
                    self.head()
                        .compare_exchange(ptr, next, Ordering::Release, Ordering::Relaxed);
                self.try_reclaim();
                continue;
            }

            let claimed = seg.claimed.load(Ordering::Relaxed);
            if claimed <= consuming {
                return None;
            }
            if seg.slots[consuming].state.load(Ordering::Acquire) != WRITTEN {
                return None;
            }
            match seg.consumed.compare_exchange(
                consuming,
                consuming + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(idx) => {
                    return Some(seg.slots[idx].read_existing_value());
                }
                Err(_) => {
                    continue;
                }
            };
        }
    }
}

impl<T, R: Reclaimer> Drop for SegQueue<T, R> {
    fn drop(&mut self) {
        self.domain.remove_root(self.head_root);
        self.domain.remove_root(self.tail_root);

        // SAFETY: `Box::from_raw` needs three things.
        // 1. The address came from a `Box`: `Box::into_raw` in `new`, and the
        //    field is never changed.
        // 2. It is freed once: `drop` runs once, and nothing else frees it.
        // 3. Nobody uses the place afterwards. Two holders had its address:
        //    - the queue's own methods, through `head()`: `drop` takes
        //      `&mut self`, so no `push` or `pop` is running or can start;
        //    - the domain, through the root: the two `remove_root` calls
        //      above took the address out of its list. This is why the roots
        //      are removed before the places are freed.
        unsafe { drop(Box::from_raw(self.head)) };

        // SAFETY: same as for `head`, with `tail()` and `tail_root`.
        unsafe { drop(Box::from_raw(self.tail)) };
    }
}

#[cfg(test)]
mod tests {
    use crate::seg_queue::{SegQueue, SEG_LEN};
    use reclaim::{Domain, Leak};
    use std::hint::spin_loop;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Barrier;
    use std::thread;

    #[test]
    fn test_push_pop() {
        let queue = SegQueue::new(Domain::new(Leak::new()));
        queue.push(1);
        assert_eq!(queue.pop(), Some(1));
    }

    #[test]
    fn test_auto_create_new_segment() {
        let queue = SegQueue::new(Domain::new(Leak::new()));
        for i in 0..SEG_LEN {
            queue.push(i);
        }

        assert_eq!(
            queue.head().load(Ordering::Relaxed),
            queue.tail().load(Ordering::Relaxed)
        );

        queue.push(10);
        assert_ne!(
            queue.head().load(Ordering::Relaxed),
            queue.tail().load(Ordering::Relaxed)
        );
    }

    #[test]
    fn test_pop_empty_queue() {
        let queue = SegQueue::<i32, Leak>::new(Domain::new(Leak::new()));
        assert!(queue.pop().is_none());
    }

    #[test]
    fn test_push_pop_fifo() {
        let queue = SegQueue::new(Domain::new(Leak::new()));
        queue.push(1);
        queue.push(2);

        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
    }

    #[test]
    fn test_consumer_drain_index() {
        let queue = SegQueue::new(Domain::new(Leak::new()));
        queue.push(1);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), None);
        queue.push(2);
        assert_eq!(queue.pop(), Some(2));
    }

    #[test]
    fn fifo_through_edge() {
        let queue = SegQueue::new(Domain::new(Leak::new()));

        for i in 0..37 {
            queue.push(i);
        }

        let mut vec = Vec::new();
        loop {
            match queue.pop() {
                None => {
                    break;
                }
                Some(i) => {
                    vec.push(i);
                }
            }
        }

        assert_eq!(vec.len(), 37);
        assert!((0..37).eq(vec));

        assert!(queue.pop().is_none());
    }

    #[test]
    fn interleave_push_pop() {
        let queue = SegQueue::new(Domain::new(Leak::new()));

        for i in 0..100 {
            queue.push(i);
            assert_eq!(queue.pop(), Some(i));
        }

        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn consumer_index_drain() {
        let queue = SegQueue::new(Domain::new(Leak::new()));
        queue.push(1);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), None);

        queue.push(2);
        assert_eq!(queue.pop(), Some(2));
    }

    #[test]
    fn concurrent_mpmc() {
        const PRODUCERS: usize = 4;
        const CONSUMERS: usize = 4;
        #[cfg(not(miri))]
        const PER_PRODUCER: usize = 20_000;
        #[cfg(miri)]
        const PER_PRODUCER: usize = 50;
        const TOTAL: usize = PRODUCERS * PER_PRODUCER;

        let queue = SegQueue::<usize, Leak>::new(Domain::new(Leak::new()));
        let barrier = Barrier::new(PRODUCERS + CONSUMERS);
        let popped = AtomicUsize::new(0);

        let logs: Vec<Vec<usize>> = thread::scope(|s| {
            let q = &queue;
            let b = &barrier;
            let n = &popped;

            // Consumers first, so they are already spinning when producers start.
            let consumers: Vec<_> = (0..CONSUMERS)
                .map(|_| {
                    s.spawn(move || {
                        let mut seen = Vec::new();
                        b.wait();
                        while n.load(Ordering::Relaxed) < TOTAL {
                            if let Some(v) = q.pop() {
                                n.fetch_add(1, Ordering::Relaxed);
                                seen.push(v);
                            } else {
                                spin_loop();
                            }
                        }
                        seen
                    })
                })
                .collect();

            for p in 0..PRODUCERS {
                s.spawn(move || {
                    b.wait();
                    for seq in 0..PER_PRODUCER {
                        q.push(p * PER_PRODUCER + seq);
                    }
                });
            }

            consumers.into_iter().map(|h| h.join().unwrap()).collect()
        });

        let mut seen = vec![false; TOTAL];
        let mut count = 0usize;
        for log in &logs {
            for &v in log {
                assert!(v < TOTAL, "popped out-of-range value {v}");
                assert!(!seen[v], "value {v} popped more than once");
                seen[v] = true;
                count += 1;
            }
        }
        assert_eq!(count, TOTAL, "wrong number of items popped");
        if let Some(missing) = seen.iter().position(|&s| !s) {
            panic!("value {missing} was pushed but never popped");
        }

        for (c, log) in logs.iter().enumerate() {
            let mut last_seq = vec![None::<usize>; PRODUCERS];
            for &v in log {
                let (p, seq) = (v / PER_PRODUCER, v % PER_PRODUCER);
                if let Some(prev) = last_seq[p] {
                    assert!(
                        seq > prev,
                        "consumer {c} saw producer {p} out of order: seq {prev} before {seq}"
                    );
                }
                last_seq[p] = Some(seq);
            }
        }
    }

    /// Under `Leak` a retired segment is never freed, so the chain of
    /// segments only grows.
    ///
    /// The queue forgets a segment once it has retired it: `head`, `tail` and
    /// `reclaim` have all moved on. So the test saves the address of the first
    /// segment before anything is pushed, and walks the chain from there at
    /// the end.
    ///
    /// The walk opens segments that were retired. Run under Miri, that is
    /// what gives the test its teeth: if a scheme had freed one of them, Miri
    /// reports a use after free on that line.
    #[test]
    fn leak_keeps_every_retired_segment() {
        // One value more than three full segments: the queue has to link a
        // fourth segment and move past the first three.
        const VALUES: usize = 3 * SEG_LEN + 1;

        let queue = SegQueue::new(Domain::new(Leak::new()));
        let first = queue.reclaim.load(Ordering::Relaxed);

        for value in 0..VALUES {
            queue.push(value);
        }
        for expected in 0..VALUES {
            assert_eq!(queue.pop(), Some(expected));
        }
        assert_eq!(queue.pop(), None);

        // The queue really moved on. Without this, the test would also pass
        // for a queue that never retires anything.
        let oldest_not_retired = queue.reclaim.load(Ordering::Relaxed);
        assert_ne!(oldest_not_retired, first, "no segment was retired");

        // Follow `next` from the first segment to the end of the chain.
        let mut chain = Vec::new();
        let mut seg = first;
        while !seg.is_null() {
            chain.push(seg);
            // SAFETY: this is the claim under test. `Leak` never frees a
            // retired segment, and the queue is still alive, so every segment
            // from the first one on is still allocated.
            seg = unsafe { (*seg).next.load(Ordering::Relaxed) };
        }

        // Four segments were made, and all four are still there.
        assert_eq!(chain.len(), 4);
        // The first three were retired: `reclaim` now holds the fourth.
        assert_eq!(chain[3], oldest_not_retired);
    }
}

#[cfg(all(test, loom))]
mod loom_tests {
    use crate::SegQueue;
    use loom::sync::Arc;
    use reclaim::{Domain, Leak};

    #[test]
    fn one_producer_one_consumer() {
        loom::model(|| {
            let queue = Arc::new(SegQueue::<usize, Leak>::new(Domain::new(Leak::new())));

            let producer = queue.clone();
            let handler = loom::thread::spawn(move || {
                producer.push(1);
            });

            let got = queue.pop();
            handler.join().unwrap();
            let rest = queue.pop();

            match (got, rest) {
                (Some(1), None) => {}
                (None, Some(1)) => {}
                other => panic!("bad outcome: {other:?}"),
            }
        })
    }

    /// L2 — two producers racing on the same segment's `claimed` counter. Under
    /// loom SEG_LEN == 2, so both pushes fit in the first segment and no boundary
    /// is crossed: the only question is whether `claimed` hands out two distinct
    /// slots. Pops run after both joins because the axis is producer-vs-producer.
    #[test]
    fn two_producers_claim_distinct_slots() {
        loom::model(|| {
            let queue = Arc::new(SegQueue::<usize, Leak>::new(Domain::new(Leak::new())));

            let p1 = queue.clone();
            let h1 = loom::thread::spawn(move || p1.push(1));
            let p2 = queue.clone();
            let h2 = loom::thread::spawn(move || p2.push(2));

            h1.join().unwrap();
            h2.join().unwrap();

            let mut got: Vec<usize> = [queue.pop(), queue.pop()].into_iter().flatten().collect();
            got.sort_unstable();
            assert_eq!(got, vec![1, 2], "claimed duplicated or lost a slot");
            assert_eq!(queue.pop(), None);
        })
    }

    /// L3 — two consumers racing for the one available value. The push happens
    /// before the threads start, so the only question is whether `consumed` lets
    /// exactly one of them take the slot.
    #[test]
    fn two_consumers_take_one_value_once() {
        loom::model(|| {
            let queue = Arc::new(SegQueue::<usize, Leak>::new(Domain::new(Leak::new())));
            queue.push(1);

            let c1 = queue.clone();
            let h1 = loom::thread::spawn(move || c1.pop());
            let c2 = queue.clone();
            let h2 = loom::thread::spawn(move || c2.pop());

            let a = h1.join().unwrap();
            let b = h2.join().unwrap();

            match (a, b) {
                (Some(1), None) | (None, Some(1)) => {}
                other => panic!("value delivered twice or lost: {other:?}"),
            }
        })
    }

    /// L4 — two producers overflowing the same full segment, so both race to
    /// install the successor via `next.compare_exchange`: one wins, the loser must
    /// free its spare segment and use the winner's.
    ///
    /// NOTE: this handoff goes through a CAS, which loom 0.7 does NOT reliably
    /// model — a green run here is weak evidence. The real evidence for `next`'s
    /// ordering is Miri plus the happens-before argument.
    #[test]
    fn two_producers_race_to_install_next_segment() {
        loom::model(|| {
            let queue = Arc::new(SegQueue::<usize, Leak>::new(Domain::new(Leak::new())));
            // Fill the first segment exactly (SEG_LEN == 2 under loom) so the next
            // two pushes both overflow.
            queue.push(10);
            queue.push(20);

            let p1 = queue.clone();
            let h1 = loom::thread::spawn(move || p1.push(1));
            let p2 = queue.clone();
            let h2 = loom::thread::spawn(move || p2.push(2));

            h1.join().unwrap();
            h2.join().unwrap();

            // The first segment drains in order; the two racing values land in the
            // new segment in whichever order they claimed it.
            assert_eq!(queue.pop(), Some(10));
            assert_eq!(queue.pop(), Some(20));
            let mut tail: Vec<usize> = [queue.pop(), queue.pop()].into_iter().flatten().collect();
            tail.sort_unstable();
            assert_eq!(tail, vec![1, 2], "a value was lost crossing the boundary");
            assert_eq!(queue.pop(), None);
        })
    }

    /// L5 — a pop that observes an empty queue must not consume the slot the
    /// producer is about to fill. Two speculative pops run before the push can be
    /// guaranteed complete; the value must still come out exactly once.
    ///
    /// This is the regression test for the `consumed.fetch_add`-then-bail design,
    /// which burned a consume index on every empty pop and orphaned the value.
    #[test]
    fn empty_pop_does_not_burn_the_slot() {
        loom::model(|| {
            let queue = Arc::new(SegQueue::<usize, Leak>::new(Domain::new(Leak::new())));

            let p = queue.clone();
            let h = loom::thread::spawn(move || p.push(1));

            let a = queue.pop();
            let b = queue.pop();
            h.join().unwrap();
            let c = queue.pop();

            let got: Vec<usize> = [a, b, c].into_iter().flatten().collect();
            assert_eq!(got, vec![1], "an empty pop swallowed the pushed value");
        })
    }

    /// L6 — producers and consumers running together across a segment boundary.
    /// Three values at SEG_LEN == 2, so the third lands in a second segment while
    /// two consumers are popping. The question: is any pushed value lost, or
    /// delivered twice?
    ///
    /// This is the regression harness for the protect-source restructure (Q3 in
    /// `plan/reclaim_ladder.md`).
    ///
    /// The preemption bound: four threads plus main is too many orders for loom
    /// to try them all — unbounded, and with a bound of 3, the run does not
    /// finish in minutes. With a bound of 2 it takes about 25 s. So a green run
    /// means "no value is lost in any order with at most two forced thread
    /// switches", not "in any order".
    #[test]
    fn nothing_is_lost_across_a_boundary() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);
        builder.check(|| {
            let queue = Arc::new(SegQueue::<usize, Leak>::new(Domain::new(Leak::new())));
            let p1 = queue.clone();
            let p2 = queue.clone();

            let h1 = loom::thread::spawn(move || {
                p1.push(1);
                p1.push(2)
            });
            let h2 = loom::thread::spawn(move || p2.push(3));

            let c1 = queue.clone();
            let c2 = queue.clone();
            let h3 = loom::thread::spawn(move || c1.pop());
            let h4 = loom::thread::spawn(move || c2.pop());

            let mut got: Vec<usize> = [h3, h4]
                .into_iter()
                .map(|v| v.join().unwrap())
                .flatten()
                .collect();
            h1.join().unwrap();
            h2.join().unwrap();

            while let Some(v) = queue.pop() {
                got.push(v);
            }

            got.sort_unstable();
            assert_eq!(got, vec![1, 2, 3], "a value was lost crossing the boundary");
        })
    }
}
