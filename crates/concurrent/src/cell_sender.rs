use core::mem::MaybeUninit;

/// Loom/std shim: under `--cfg loom`, loom swaps in instrumented atomics, cell, and Arc so the
/// 2-thread loom test can explore reorderings. Mirrors `atomic_cell.rs` / `once_flag.rs`.
mod sync {
    #[cfg(not(loom))]
    pub(super) use crate::arc::Arc;
    #[cfg(not(loom))]
    pub(super) use core::cell::UnsafeCell;
    #[cfg(not(loom))]
    pub(super) use core::sync::atomic::{AtomicUsize, Ordering};

    #[cfg(loom)]
    pub(super) use loom::cell::UnsafeCell;
    #[cfg(loom)]
    pub(super) use loom::sync::atomic::{AtomicUsize, Ordering};
    #[cfg(loom)]
    pub(super) use loom::sync::Arc;
}

use sync::{Arc, AtomicUsize, Ordering, UnsafeCell};

pub struct Channel<T> {
    data: UnsafeCell<MaybeUninit<T>>,
    state: AtomicUsize,
}

const INIT: usize = 0;
const WRITING: usize = 1;
const DONE: usize = 2;

pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Option<&T> {
        if self.channel.state.load(Ordering::Acquire) != DONE {
            return None;
        }

        // Read the published payload. The cfg branch exists ONLY because loom's UnsafeCell has a
        // different access API (`.with`) than std's (`.get()`) — the logic is identical.
        #[cfg(not(loom))]
        let value = unsafe { (*self.channel.data.get()).assume_init_ref() };
        #[cfg(loom)]
        let value = self
            .channel
            .data
            .with(|p| unsafe { (*p).assume_init_ref() });

        Some(value)
    }
}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Send for Receiver<T> {}

impl<T> Channel<T> {
    pub fn new() -> (Sender<T>, Receiver<T>) {
        let channel = Channel {
            data: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicUsize::new(INIT),
        };
        let arc = Arc::new(channel);
        let sender = Sender {
            channel: arc.clone(),
        };
        let receiver = Receiver {
            channel: arc.clone(),
        };
        (sender, receiver)
    }
}

impl<T> Sender<T> {
    pub fn write(&self, val: T) -> Result<(), T> {
        if self
            .channel
            .state
            .compare_exchange(INIT, WRITING, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            return Err(val);
        }

        // Write the payload (same cfg-branch-for-access-API reason as `try_recv`).
        #[cfg(not(loom))]
        unsafe {
            (*self.channel.data.get()).write(val);
        }
        #[cfg(loom)]
        self.channel.data.with_mut(|p| unsafe {
            (*p).write(val);
        });

        self.channel.state.store(DONE, Ordering::Release);
        Ok(())
    }
}
