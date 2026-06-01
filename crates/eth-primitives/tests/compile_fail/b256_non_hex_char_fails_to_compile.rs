// R4: a `b256!` literal containing a non-hex character must fail at COMPILE
// time. The literal is a *correct-length* 64 hex chars (so it clears the length
// check) but the final byte is `gg` — `nibble_const` panics on `g` during const
// evaluation, surfacing as a compile error.
//
// `const` context is load-bearing: it forces the panic to fire at compile time
// rather than runtime. See b256_short_literal_fails_to_compile.rs.
use eth_primitives::{b256, B256};

const _BAD: B256 = b256!("0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeegg");

fn main() {}
