mod tests {
    use eth_primitives::SimpleEncode;

    #[derive(SimpleEncode)]
    struct Foo {
        a: u8,
        b: u16,
        c: u32,
    }

    #[test]
    fn derive_on_three_field_struct_emits_correct_impl() {
        let foo = Foo { a: 0x01, b: 0x0203, c: 0x04050607 };
        let mut out = Vec::new();
        foo.encode(&mut out);
        // a:1B, b:2B BE, c:4B BE  → 7 bytes, in declaration order
        assert_eq!(out, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]);
    }
}

// R5: the generated impl names every external item by absolute path
// (`::eth_primitives::…`), so the derive must work in a module that NEVER
// imports the trait or the derive macro. This module has zero `use` of
// SimpleEncode — both the `#[derive(...)]` and the encode call go through
// absolute paths. If the codegen used a bare name, this would fail to compile.
mod no_import {
    #[derive(::eth_primitives::SimpleEncode)]
    pub struct Bar {
        pub x: u8,
        pub y: u16,
    }
}

#[test]
fn derive_works_when_caller_did_not_import_simple_encode() {
    let bar = no_import::Bar { x: 0xaa, y: 0xbbcc };
    let mut out = Vec::new();
    // Fully-qualified call — needs no `use` of the trait either.
    ::eth_primitives::SimpleEncode::encode(&bar, &mut out);
    assert_eq!(out, [0xaa, 0xbb, 0xcc]);
}
