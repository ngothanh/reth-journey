use crate::sync::AtomicPtr;

pub struct RetireLink {
    next: AtomicPtr<()>,
}

trait Retirable: Send {
    fn retire_link(&self) -> &RetireLink;

    unsafe fn reclaim(ptr: *mut Self);
}
