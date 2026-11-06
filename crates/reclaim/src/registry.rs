use crate::sync::{AtomicBool, AtomicPtr, Ordering, UnsafeCell};
use concurrent_utils::CachePadded;
use std::marker::PhantomData;
use std::ptr::{null, null_mut};

pub(crate) trait Payload: Default + Send + Sync {
    /// After clear, the payload means using nothing,
    /// so the scan that reads a free record protects nothing
    fn clear(&self);
}

impl Payload for AtomicPtr<()> {
    fn clear(&self) {
        self.store(null_mut(), Ordering::Relaxed);
    }
}

pub(crate) struct Registry<P: Payload> {
    head: AtomicPtr<CachePadded<Record<P>>>,
    _owns: PhantomData<P>,
}

impl<P: Payload> Registry<P> {
    pub(crate) fn new() -> Self {
        Self {
            head: AtomicPtr::new(null_mut()),
            _owns: PhantomData,
        }
    }

    pub(crate) fn push_new(&self) -> &Record<P> {
        let new_record = Box::into_raw(Box::new(CachePadded::new(Record::<P>::new())));
        let mut top = self.head.load(Ordering::Relaxed);
        loop {
            // SAFETY: `new_record` came from `Box::into_raw` above and is not in the list
            // yet, so no other thread can reach it: this write is the only access.
            unsafe { (&*new_record).next.with_mut(|p| *p = top) };
            // `Release` publishes the record: its fields (`next`, `in_use`, `payload`) are
            // written before this, and a scan that loads `head` with `Acquire` sees them.
            match self.head.compare_exchange_weak(
                top,
                new_record,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                // SAFETY: records are never freed while the registry lives (Rule 1), and
                // the returned borrow cannot outlive `&self`.
                Ok(_) => return unsafe { &*new_record },
                Err(current) => top = current,
            }
        }
    }

    /// A record nobody else holds: a free one if there is one, else a new one.
    pub(crate) fn claim(&self) -> &Record<P> {
        for rec in self.iter() {
            // Taking a free record is one compare-exchange: of two claimers that both
            // see it free, exactly one wins. The loser moves on; this record is now taken.
            // `Acquire` on success pairs with the `Release` in `release`: the old owner's
            // clear is visible before this thread writes the payload.
            if rec
                .in_use
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return rec;
            }
        }
        self.push_new()
    }
}

impl<P: Payload> Registry<P> {
    /// Every record from `head` onwards, in use or free. A free record's payload is
    /// cleared (Rule 2), so reading it protects nothing. Reads `head` once: a record
    /// added after that is not seen by this walk.
    pub(crate) fn iter(&self) -> Iter<'_, P> {
        Iter {
            // `Acquire` pairs with the `Release` in `push_new`: a record found here is seen
            // fully built, and so is every record after it (they were published earlier).
            current: self.head.load(Ordering::Acquire),
            _registry: PhantomData,
        }
    }
}

/// A walk over a registry's records. Each `&'a Record` borrows the registry, so no
/// record reference can outlive it.
pub(crate) struct Iter<'a, P: Payload> {
    current: *const CachePadded<Record<P>>,
    _registry: PhantomData<&'a Registry<P>>,
}

impl<'a, P: Payload> Iterator for Iter<'a, P> {
    type Item = &'a Record<P>;

    fn next(&mut self) -> Option<&'a Record<P>> {
        if self.current.is_null() {
            return None;
        }
        // SAFETY: `current` is a record of the registry borrowed for `'a`. Records are
        // freed only in the registry's `Drop` (Rule 1), which needs `&mut` and so cannot
        // run while `'a` lasts.
        let rec: &'a Record<P> = unsafe { &*self.current };
        // SAFETY: `next` is written only in `push_new`, before the record is published,
        // and `iter`'s `Acquire` load of `head` ordered that write before this read.
        self.current = rec.next.with(|p| unsafe { *p });
        Some(rec)
    }
}

impl<P: Payload> Drop for Registry<P> {
    fn drop(&mut self) {
        // `Relaxed` is enough: `&mut self` means no other thread can reach the registry,
        // and the hand-over that gave this thread `&mut` (the last `Arc` drop, a `join`)
        // already ordered every earlier `push_new` before this point.
        let mut top = self.head.load(Ordering::Relaxed);
        while !top.is_null() {
            // SAFETY: `top` is a record in this list. Records are freed only here (Rule 1),
            // and this one is not freed yet: `next` is read before the `from_raw` below.
            // No other thread can write `next` (`&mut self`), and its last write, in
            // `push_new`, happened before this thread got `&mut` (see above).
            let next = unsafe { (&*top).next.with(|p| *p) };

            // SAFETY: every record in the list came from `Box::into_raw` in `push_new`,
            // and is in the list once, so this frees each record exactly once. No `&Record`
            // from `push_new` is still alive: it borrows `&self`, and we hold `&mut self`.
            drop(unsafe { Box::from_raw(top) });
            top = next as *mut CachePadded<Record<P>>;
        }
    }
}

pub(crate) struct Record<P: Payload> {
    payload: P,
    in_use: AtomicBool,
    /// Written only while `push_new` still owns the record, then only read. An
    /// `UnsafeCell` (from `sync`) so that loom checks every read against that write.
    next: UnsafeCell<*const CachePadded<Record<P>>>,
}

impl<P: Payload> Record<P> {
    fn new() -> Self {
        Record {
            payload: P::default(),
            in_use: AtomicBool::new(true),
            next: UnsafeCell::new(null()),
        }
    }

    pub(crate) fn payload(&self) -> &P {
        &self.payload
    }

    /// Gives the record back for reuse: clears the payload, then marks it free.
    ///
    /// # Safety
    /// The caller holds the claim on this record (it came from `claim` or
    /// `push_new` and has not been released since), and does not use the
    /// record again until it claims it again.
    pub(crate) unsafe fn release(&self) {
        self.payload.clear();
        // `Release`: the clear above is visible to whoever claims this record next (their
        // compare-exchange is `Acquire`). With `Relaxed`, a new owner could still see the
        // old payload, and the old clear could land after the new owner's write (Rule 2).
        self.in_use.store(false, Ordering::Release);
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use crate::registry::{Record, Registry};
    use crate::sync::{AtomicPtr, Ordering};
    use concurrent_utils::CachePadded;
    use std::ptr::null_mut;

    #[test]
    fn new_registry_is_empty() {
        let registry = Registry::<AtomicPtr<()>>::new();
        assert!(registry.head.load(Ordering::Relaxed).is_null());
    }

    #[test]
    fn push_new() {
        let registry = Registry::<AtomicPtr<()>>::new();

        let returned: Vec<*const Record<AtomicPtr<()>>> = (0..3)
            .map(|_| {
                let r = registry.push_new();
                assert_eq!(r.payload().load(Ordering::Relaxed), null_mut());
                assert!(r.in_use.load(Ordering::Relaxed));
                r as *const _
            })
            .collect();

        // Three different records, not one record handed out three times.
        assert_ne!(returned[0], returned[1]);
        assert_ne!(returned[1], returned[2]);
        assert_ne!(returned[0], returned[2]);

        // And all three are in the list: walk it from `head`, newest first.
        let in_list: Vec<*const Record<AtomicPtr<()>>> =
            registry.iter().map(|r| r as *const _).collect();
        assert_eq!(in_list, [returned[2], returned[1], returned[0]]);
    }

    #[test]
    fn two_claims_give_two_records() {
        let registry = Registry::<AtomicPtr<()>>::new();
        let a = registry.claim();
        let b = registry.claim();
        assert!(!std::ptr::eq(a, b));
        assert_eq!(registry.iter().count(), 2);
    }

    #[test]
    fn a_released_record_is_claimed_again() {
        let registry = Registry::<AtomicPtr<()>>::new();
        let a = registry.claim();
        // SAFETY: `a` was claimed above and is not used again after this.
        unsafe { a.release() };
        let b = registry.claim();
        assert!(std::ptr::eq(a, b));
        assert_eq!(registry.iter().count(), 1); // reused, nothing new allocated
    }

    #[test]
    fn release_clears_the_payload() {
        let registry = Registry::<AtomicPtr<()>>::new();
        let a = registry.claim();
        a.payload().store(0x1000 as *mut (), Ordering::Relaxed);
        // SAFETY: `a` was claimed above; it is only read again through `iter`.
        unsafe { a.release() };
        let rec = registry.iter().next().unwrap();
        assert!(!rec.in_use.load(Ordering::Relaxed));
        assert!(rec.payload().load(Ordering::Relaxed).is_null());
    }

    #[test]
    fn registry_is_send_and_sync() {
        fn needs_send_sync<X: Send + Sync>() {}
        needs_send_sync::<Registry<AtomicPtr<()>>>();
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn a_record_has_its_own_cache_line() {
        type Padded = CachePadded<Record<AtomicPtr<()>>>;
        assert_eq!(align_of::<Padded>(), 128);
        assert_eq!(size_of::<Padded>() % 128, 0);
    }
}

#[cfg(all(test, loom))]
mod loom_tests {
    use crate::registry::Registry;
    use crate::sync::{AtomicPtr, Ordering};
    use crate::test_support::Arc;

    /// Two threads claim at once from a registry with one free record. Whatever the
    /// interleaving, they must end up with different records (one may be new).
    #[test]
    fn two_claimers_never_share_a_record() {
        loom::model(|| {
            let registry = Arc::new(Registry::<AtomicPtr<()>>::new());
            let first = registry.claim();
            // SAFETY: `first` was claimed just above and is not used again.
            unsafe { first.release() };

            let claimers = (0..2)
                .map(|_| {
                    let registry = registry.clone();
                    loom::thread::spawn(move || registry.claim() as *const _ as usize)
                })
                .collect::<Vec<_>>();
            let got: Vec<usize> = claimers.into_iter().map(|c| c.join().unwrap()).collect();

            assert_ne!(got[0], got[1], "two claimers got the same record");
        })
    }

    /// One thread claims from an empty registry (so `push_new` publishes a new record)
    /// and writes an address into it. The main thread scans until it finds the record,
    /// then checks it is fully built: in use, and a payload that is either still null or
    /// the address written. The scan must wait for the record: a scan that may run
    /// before the claimer finds an empty list, and loom explores only that one run.
    /// Negative controls: `push_new`'s CAS `Relaxed`, or `iter`'s load `Relaxed`, each
    /// fail with a causality violation on `next` (`Iter::next` vs `push_new`).
    #[test]
    fn a_scan_sees_a_new_record_whole() {
        loom::model(|| {
            let registry = Arc::new(Registry::<AtomicPtr<()>>::new());

            let reader = {
                let registry = registry.clone();
                loom::thread::spawn(move || {
                    let rec = registry.claim();
                    rec.payload().store(0x1000 as *mut (), Ordering::Relaxed);
                })
            };

            let rec = loop {
                if let Some(rec) = registry.iter().next() {
                    break rec;
                }
                loom::thread::yield_now();
            };
            {
                assert!(rec.in_use.load(Ordering::Relaxed));
                let p = rec.payload().load(Ordering::Relaxed);
                assert!(p.is_null() || p == 0x1000 as *mut ());
            }

            reader.join().unwrap();
        })
    }

    /// Rule 2 across threads. T1 claims a record, writes an address, and releases it;
    /// T3 claims at the same time. If T3 gets T1's record, T1's clear must already be
    /// visible to T3: otherwise T1's null could still land after T3's own write and
    /// erase T3's protection (the exhibit in `notes/reclaim_model.md` §10).
    #[test]
    fn a_claimed_record_starts_cleared() {
        loom::model(|| {
            let registry = Arc::new(Registry::<AtomicPtr<()>>::new());

            let t1 = {
                let registry = registry.clone();
                loom::thread::spawn(move || {
                    let rec = registry.claim();
                    rec.payload().store(0x1000 as *mut (), Ordering::Relaxed);
                    // SAFETY: claimed just above, not used after this.
                    unsafe { rec.release() };
                })
            };

            // Wait until T1's record exists and T1 has released it, so the claim
            // below takes T1's record. Without the wait, loom runs the claim first,
            // on an empty list, and the model checks nothing.
            loop {
                if let Some(r) = registry.iter().next() {
                    if !r.in_use.load(Ordering::Relaxed) {
                        break;
                    }
                }
                loom::thread::yield_now();
            }
            let rec = registry.claim();
            assert!(
                rec.payload().load(Ordering::Relaxed).is_null(),
                "claimed a record whose old payload is still visible"
            );

            t1.join().unwrap();
        })
    }
}
