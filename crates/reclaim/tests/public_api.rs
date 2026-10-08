// Files in `tests/` are built as a separate crate, so they see only what a
// user of `reclaim` sees.
#![cfg(not(loom))]

use reclaim::{Domain, Leak};

#[test]
fn a_user_can_build_a_leak_domain() {
    let _domain = Domain::new(Leak::new());
}
