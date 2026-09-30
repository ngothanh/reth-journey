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
        let mut cur_seg = self.tail.load(Ordering::Acquire);
        loop {
            let idx = unsafe { (*cur_seg).claimed.fetch_add(1, Ordering::Relaxed) };
            if idx < SEG_LEN {
                unsafe {
                    (*(*cur_seg).slots[idx].value.get()).write(value);
                    (*cur_seg).slots[idx]
                        .state
                        .store(WRITTEN, Ordering::Release);
                }
                return;
            }
            cur_seg = self.advance_tail(cur_seg);
        }
    }

    fn advance_tail(&self, cur_seg: *mut Segment<T>) -> *mut Segment<T> {
        let mut next = unsafe { (*cur_seg).next.load(Ordering::Acquire) };
        if next.is_null() {
            let raw = Box::into_raw(Box::new(Segment::new()));
            match unsafe {
                (*cur_seg).next.compare_exchange(
                    null_mut(),
                    raw,
                    Ordering::Release,
                    Ordering::Acquire,
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
            .compare_exchange(cur_seg, next, Ordering::Release, Ordering::Relaxed);
        next
    }

    pub fn pop(&self) -> Option<T> {
        let mut cur_seg = self.head.load(Ordering::Acquire);
        let mut consuming = unsafe { (*cur_seg).consumed.load(Ordering::Relaxed) };
        loop {
            if consuming >= SEG_LEN {
                let next = unsafe { (*cur_seg).next.load(Ordering::Acquire) };
                if next.is_null() {
                    return None;
                }
                let _ =
                    self.head
                        .compare_exchange(cur_seg, next, Ordering::Release, Ordering::Relaxed);
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
                        while (*cur_seg).slots[idx].state.load(Ordering::Acquire) != WRITTEN {
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
    use std::hint::spin_loop;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Barrier;
    use std::thread;

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

    #[test]
    fn concurrent_mpmc() {
        const PRODUCERS: usize = 4;
        const CONSUMERS: usize = 4;
        const PER_PRODUCER: usize = 20_000;
        const TOTAL: usize = PRODUCERS * PER_PRODUCER;

        let queue = SegQueue::<usize>::new();
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
}
