use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

const SEG_LEN: usize = 32;

const EMPTY: usize = 0;
const WRITTEN: usize = 1;

struct Slot<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    state: AtomicUsize,
}
struct Segment<T> {
    slots: [Slot<T>; SEG_LEN],
    next: AtomicPtr<Segment<T>>,
    consumed: AtomicUsize,
    claimed: AtomicUsize,
}

pub struct SegQueue<T> {
    head: AtomicPtr<Segment<T>>,
    tail: AtomicPtr<Segment<T>>,
}

unsafe impl<T: Send> Send for SegQueue<T> {}
unsafe impl<T: Send> Sync for SegQueue<T> {}

impl<T> Segment<T> {
    fn new() -> Self {
        Segment {
            slots: std::array::from_fn(|_| Slot::new()),
            next: AtomicPtr::new(null_mut()),
            consumed: AtomicUsize::new(0),
            claimed: AtomicUsize::new(0),
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
}

impl<T> SegQueue<T> {
    pub fn new() -> SegQueue<T> {
        let segment = Box::into_raw(Box::new(Segment::new()));
        SegQueue {
            head: AtomicPtr::new(segment),
            tail: AtomicPtr::new(segment),
        }
    }

    pub fn push(&self, value: T) {
        let mut cur_seg = self.tail.load(Ordering::Relaxed);
        loop {
            let idx = unsafe { (*cur_seg).claimed.fetch_add(1, Ordering::Relaxed) };
            if idx < SEG_LEN {
                unsafe {
                    (*(*cur_seg).slots[idx].value.get()).write(value);
                    (*cur_seg).slots[idx]
                        .state
                        .store(WRITTEN, Ordering::Relaxed);
                }
                return;
            }
            cur_seg = self.advance_tail(cur_seg);
        }
    }

    fn advance_tail(&self, cur_seg: *mut Segment<T>) -> *mut Segment<T> {
        let mut next = unsafe { (*cur_seg).next.load(Ordering::Relaxed) };
        if next.is_null() {
            let raw = Box::into_raw(Box::new(Segment::new()));
            match unsafe {
                (*cur_seg).next.compare_exchange(
                    null_mut(),
                    raw,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
            } {
                Ok(_) => next = raw,
                Err(winner) => {
                    drop(unsafe { Box::from_raw(raw) });
                    next = winner;
                }
            }
        }
        let _ = self
            .tail
            .compare_exchange(cur_seg, next, Ordering::Relaxed, Ordering::Relaxed);
        next
    }

    pub fn pop(&self) -> Option<T> {
        todo!()
    }
}
