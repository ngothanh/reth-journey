use core::cell::UnsafeCell;
use core::marker::PhantomPinned;
use core::ptr::NonNull;
use core::task::Waker;

pub struct Waiter {
    waker: UnsafeCell<Option<Waker>>,
    pub granted: bool,
    prev: Option<NonNull<Waiter>>,
    next: Option<NonNull<Waiter>>,
    _pin: PhantomPinned,
}

impl Waiter {
    pub fn new() -> Self {
        Waiter {
            waker: UnsafeCell::new(None),
            granted: false,
            prev: None,
            next: None,
            _pin: PhantomPinned,
        }
    }

    pub(crate) fn update_waker(&self, w: &Waker) {
        let slot = self.waker.get();

        //SAFETY: Waiter can only be retrieved under mutex lock
        unsafe {
            let need = match &*slot {
                Some(old) => !old.will_wake(w),
                None => true,
            };
            if need {
                *slot = Some(w.clone());
            }
        }
    }

    pub(crate) fn take_waker(&self) -> Option<Waker> {
        //SAFETY: Waiter can only be retrieved under mutex lock
        unsafe { (*self.waker.get()).take() }
    }
}

pub struct WaitList {
    head: Option<NonNull<Waiter>>,
    tail: Option<NonNull<Waiter>>,
}

impl WaitList {
    pub fn new() -> Self {
        WaitList {
            head: None,
            tail: None,
        }
    }

    pub(crate) fn push_back(&mut self, node: NonNull<Waiter>) {
        match self.tail {
            None => {
                self.tail = Some(node);
                self.head = Some(node);
            }
            Some(t) => unsafe {
                let new_tail = Some(node);
                (*t.as_ptr()).next = new_tail;
                (*node.as_ptr()).prev = Some(t);
                (*node.as_ptr()).next = None;
                self.tail = new_tail;
            },
        }
    }

    pub(crate) fn peak(&mut self) -> Option<NonNull<Waiter>> {
        self.head
    }

    pub(crate) fn pop_front(&mut self) -> Option<NonNull<Waiter>> {
        match self.head {
            None => None,
            Some(h) => unsafe {
                let next = (*h.as_ptr()).next;
                match next {
                    None => {
                        self.head = None;
                        self.tail = None;
                    }
                    Some(n) => {
                        (*h.as_ptr()).next = None;
                        (*n.as_ptr()).prev = None;
                        self.head = next;
                    }
                }

                Some(h)
            },
        }
    }

    pub(crate) fn unlink(&mut self, node: NonNull<Waiter>) {
        if Some(node) == self.head {
            self.pop_front();
            return;
        }

        if Some(node) == self.tail {
            unsafe {
                let prev = (*node.as_ptr()).prev;
                match prev {
                    None => {}
                    Some(p) => {
                        (*p.as_ptr()).next = None;
                        (*node.as_ptr()).prev = None;
                        self.tail = prev;
                    }
                }
            }
            return;
        }

        if self.head.is_some() {
            unsafe {
                let next = (*node.as_ptr()).next.unwrap();
                let prev = (*node.as_ptr()).prev.unwrap();
                (*next.as_ptr()).prev = Some(prev);
                (*prev.as_ptr()).next = Some(next);
            }
        }
        unsafe {
            (*node.as_ptr()).next = None;
            (*node.as_ptr()).prev = None;
        }
    }

    pub(crate) fn len(&self) -> usize {
        if self.head.is_none() {
            0
        } else {
            let mut start = self.head;
            let mut count = 0;
            while start.is_some() {
                count += 1;
                unsafe {
                    start = (*start.unwrap().as_ptr()).next;
                }
            }
            count
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node() -> (Box<Waiter>, NonNull<Waiter>) {
        let mut b = Box::new(Waiter::new());
        let p = NonNull::from(&mut *b);
        (b, p)
    }

    #[test]
    fn fifo_push_pop() {
        let (_a, a) = node();
        let (_b, b) = node();
        let (_c, c) = node();
        let mut list = WaitList::new();
        list.push_back(a);
        list.push_back(b);
        list.push_back(c);
        assert_eq!(list.pop_front(), Some(a));
        assert_eq!(list.pop_front(), Some(b));
        assert_eq!(list.pop_front(), Some(c));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn unlink_middle() {
        let (_a, a) = node();
        let (_b, b) = node();
        let (_c, c) = node();
        let mut list = WaitList::new();
        list.push_back(a);
        list.push_back(b);
        list.push_back(c);
        list.unlink(b);
        assert_eq!(list.pop_front(), Some(a));
        assert_eq!(list.pop_front(), Some(c));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn unlink_head() {
        let (_a, a) = node();
        let (_b, b) = node();
        let mut list = WaitList::new();
        list.push_back(a);
        list.push_back(b);
        list.unlink(a);
        assert_eq!(list.pop_front(), Some(b));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn unlink_tail() {
        let (_a, a) = node();
        let (_b, b) = node();
        let mut list = WaitList::new();
        list.push_back(a);
        list.push_back(b);
        list.unlink(b);
        assert_eq!(list.pop_front(), Some(a));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn unlink_single() {
        let (_a, a) = node();
        let mut list = WaitList::new();
        list.push_back(a);
        list.unlink(a);
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn pop_empty() {
        let mut list = WaitList::new();
        assert_eq!(list.pop_front(), None);
    }
}
