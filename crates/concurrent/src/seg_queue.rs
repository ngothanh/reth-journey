use std::cell::UnsafeCell;
use std::hint::spin_loop;
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
        let mut cur_seg = self.head.load(Ordering::Relaxed);
        let mut consuming = unsafe { (*cur_seg).consumed.load(Ordering::Relaxed) };
        loop {
            if consuming >= SEG_LEN {
                let next = unsafe { (*cur_seg).next.load(Ordering::Relaxed) };
                if next.is_null() {
                    return None;
                }
                let _ =
                    self.head
                        .compare_exchange(cur_seg, next, Ordering::Relaxed, Ordering::Relaxed);
                cur_seg = next;
                consuming = unsafe { (*cur_seg).consumed.load(Ordering::Relaxed) };
                continue;
            }

            let claimed = unsafe { (*cur_seg).claimed.load(Ordering::Relaxed) };
            if claimed <= consuming {
                return None;
            }

            unsafe {
                match (*cur_seg).consumed.compare_exchange(
                    consuming,
                    consuming + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(idx) => {
                        while (*cur_seg).slots[idx].state.load(Ordering::Relaxed) != WRITTEN {
                            spin_loop()
                        }
                        return Some((*(*cur_seg).slots[idx].value.get()).assume_init_read());
                    }
                    Err(e) => {
                        consuming = e;
                        continue;
                    }
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::seg_queue::{SegQueue, SEG_LEN};
    use std::sync::atomic::Ordering;

    #[test]
    fn test_push_pop() {
        let queue = SegQueue::new();
        queue.push(1);
        assert_eq!(queue.pop(), Some(1));
    }

    #[test]
    fn test_auto_create_new_segment() {
        let queue = SegQueue::new();
        for i in 0..SEG_LEN {
            queue.push(i);
        }

        assert_eq!(
            queue.head.load(Ordering::Relaxed),
            queue.tail.load(Ordering::Relaxed)
        );

        queue.push(10);
        assert_ne!(
            queue.head.load(Ordering::Relaxed),
            queue.tail.load(Ordering::Relaxed)
        );
    }

    #[test]
    fn test_pop_empty_queue() {
        let queue = SegQueue::<i32>::new();
        assert!(queue.pop().is_none());
    }

    #[test]
    fn test_push_pop_fifo() {
        let queue = SegQueue::new();
        queue.push(1);
        queue.push(2);

        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
    }

    #[test]
    fn test_consumer_drain_index() {
        let queue = SegQueue::new();
        queue.push(1);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), None);
        queue.push(2);
        assert_eq!(queue.pop(), Some(2));
    }

    #[test]
    fn fifo_through_edge() {
        let queue = SegQueue::new();

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
        let queue = SegQueue::new();

        for i in 0..100 {
            queue.push(i);
            assert_eq!(queue.pop(), Some(i));
        }

        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn consumer_index_drain() {
        let queue = SegQueue::new();
        queue.push(1);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), None);

        queue.push(2);
        assert_eq!(queue.pop(), Some(2));
    }
}
