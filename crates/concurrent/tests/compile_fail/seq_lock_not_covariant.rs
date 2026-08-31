//! `SeqLock<T>` must be INVARIANT in `T`, exactly like the `UnsafeCell<T>` it
//! wraps. Interior mutability through a shared `&SeqLock<T>` means a covariant
//! `SeqLock` would let a caller coerce `SeqLock<&'long U>` down to
//! `SeqLock<&'short U>`, `store` a short-lived reference into it, and then
//! `load` a dangling one back out through the still-`'long` handle.
//!
//! This test pins the property at the type level: the coercion below is legal
//! ONLY if `SeqLock` is covariant, so the fixture must FAIL to compile. If it
//! ever starts compiling, some field (in the real build `UnsafeCell<T>`, in the
//! loom build `PhantomData<UnsafeCell<T>>`) has silently lost its invariance
//! and the two builds no longer model the same type.
use concurrent::SeqLock;

fn assert_covariant<'a>(x: SeqLock<&'static str>) -> SeqLock<&'a str> {
    x
}

fn main() {
    let _ = assert_covariant;
}
