use crate::sync::*;

pub struct RetireLink {
    next: AtomicPtr<()>,
}

trait Retire: Send {
    fn retire_link(&self) -> &RetireLink;

    unsafe fn reclaim(ptr: *mut Self);
}
