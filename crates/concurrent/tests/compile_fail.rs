//! Compile-fail tests: invariants enforced by the *type system*, not by docs.
//!
//! Each fixture under `tests/compile_fail/` is expected to FAIL to compile; the
//! committed `.stderr` snapshot pins the diagnostic. Regenerate snapshots after
//! an intentional change with: `TRYBUILD=overwrite cargo test -p concurrent`.

#[test]
fn parker_compile_fail_parker_not_clone() {
    // Asserts `Parker: !Clone` at the type level — the single-consumer invariant
    // (exactly one thread may `park`) is guaranteed by the compiler refusing to
    // duplicate a Parker, rather than by a comment asking callers not to.
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/parker_not_clone.rs");
}
