//! W005 Friday — R5: unsupported input shapes must fail as a `compile_error!` pinned to
//! the `#[derive(…)]` site, never as a macro panic or a rustc-internal fallout.
//!
//! Run: `cargo test -p eth-primitives-derive --test compile_fail`
//!
//! The `.stderr` files pin the exact message. A refactor that drifts the wording turns
//! these red — which is the point: the message is part of the macro's user interface.

#[test]
fn rlp_derive_on_enum_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/rlp_derive_on_enum.rs");
}

#[test]
fn rlp_derive_on_union_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/rlp_derive_on_union.rs");
}

#[test]
fn rlp_derive_on_tuple_struct_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/rlp_derive_on_tuple_struct.rs");
}

#[test]
fn rlp_derive_on_unit_struct_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/rlp_derive_on_unit_struct.rs");
}

#[test]
fn rlp_decodable_on_enum_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/rlp_decodable_on_enum.rs");
}
