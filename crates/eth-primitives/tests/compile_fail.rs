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
