use core::task::Waker;

const CAP: usize = 32;
pub(crate) struct WakeList {
    wakers: [Option<Waker>; CAP],
    len: usize,
}

impl WakeList {
    pub(crate) fn new() -> Self {
        WakeList {
            wakers: std::array::from_fn(|_| None),
            len: 0,
        }
    }

    pub(crate) fn can_push(&self) -> bool {
        self.len < CAP
    }

    pub(crate) fn push(&mut self, waker: Waker) {
        self.wakers[self.len] = Some(waker);
        self.len += 1;
    }

    pub(crate) fn wake_all(&mut self) {
        for slot in self.wakers.iter_mut() {
            if let Some(waker) = slot.take() {
                waker.wake();
            }
        }
        self.len = 0;
    }
}
