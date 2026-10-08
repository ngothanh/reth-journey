use crate::sync::AtomicPtr;
use crate::{Guard, Reclaimer, Retirable, Root};
use std::ptr::null_mut;

pub struct Leak {
    head: AtomicPtr<()>,
}

pub struct LeakGuard {}

impl Leak {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(null_mut()),
        }
    }
}

impl Default for Leak {
    fn default() -> Self {
        Self::new()
    }
}

impl Guard for LeakGuard {
    fn try_protect<T>(&mut self, addr: *mut T, root: &Root<T>) -> Result<*mut T, *mut T> {
        Ok(addr)
    }
}

unsafe impl Reclaimer for Leak {
    type Guard = LeakGuard;

    fn guard(&self) -> Self::Guard {
        todo!()
    }

    unsafe fn retire<T: Retirable>(&self, obj: *mut T) {
        todo!()
    }
}
