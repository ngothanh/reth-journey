use crate::sync::AtomicPtr;

pub struct Root<T> {
    src: *const AtomicPtr<T>,
}
