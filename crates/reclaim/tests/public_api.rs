// Files in `tests/` are built as a separate crate, so they see only what a
// user of `reclaim` sees.
#![cfg(not(loom))]

use reclaim::{Domain, Guard, Leak};
use std::sync::atomic::AtomicPtr;

#[test]
fn a_user_can_build_a_leak_domain() {
    let _domain = Domain::new(Leak::new());
}

#[test]
fn a_user_can_protect_through_a_root() {
    let ptr = Box::into_raw(Box::new(7u64));
    let head = AtomicPtr::new(ptr);
    let domain = Domain::new(Leak::new());
    // SAFETY: `head` lives to the end of this test and is not moved.
    let root = unsafe { domain.declare_root::<u64>(&head) };
    let mut guard = domain.guard();
    let x = guard.protect(&root);

    assert_eq!(x, ptr);
    // SAFETY: `ptr` came from `Box::into_raw` above and was never freed.
    // Nothing reads it after this line.
    unsafe { drop(Box::from_raw(ptr)) };
}
