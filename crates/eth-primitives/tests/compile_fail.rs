#[test]
fn b256_short_literal_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/b256_short_literal_fails_to_compile.rs");
}

#[test]
fn b256_non_hex_char_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/b256_non_hex_char_fails_to_compile.rs");
}

#[test]
fn address_wrong_length_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/address_wrong_length_fails_to_compile.rs");
}

#[test]
fn derive_on_enum_fails_to_compile_with_clear_error() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/derive_on_enum.rs");
}

#[test]
fn derive_on_union_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/derive_on_union.rs");
}

#[test]
fn derive_on_tuple_struct_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/derive_on_tuple_struct.rs");
}

#[test]
fn derive_on_unit_struct_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/derive_on_unit_struct.rs");
}

#[test]
fn proc_macro_crate_does_not_export_simple_encode_trait() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/derive_crate_does_not_export_trait.rs");
}
