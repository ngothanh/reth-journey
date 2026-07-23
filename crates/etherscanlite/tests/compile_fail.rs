//! R3 + D7: pins the orphan-rule decision so a future contributor cannot quietly replace
//! the local conversion traits with `From`.
//!
//! Run: `cargo test -p etherscanlite --test compile_fail`

#[test]
fn from_impl_is_not_std_from() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/from_impl_is_not_std_from.rs");
}
