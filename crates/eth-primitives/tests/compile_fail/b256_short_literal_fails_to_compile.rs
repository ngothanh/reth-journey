// R3: a `b256!` literal that decodes to the wrong number of bytes must fail at
// COMPILE time, not panic at runtime. `"0xdead"` is only 2 bytes, not 32.
//
// The `const` item is load-bearing: a `const` initializer is const-evaluated,
// which forces `decode_hex`'s length `panic!` to fire at compile time. Calling
// the macro in a runtime position (e.g. `let _ = b256!("0xdead");`) would
// instead defer to a runtime panic and this file would compile — defeating the
// test.
use eth_primitives::{b256, B256};

const _SHORT: B256 = b256!("0xdead");

fn main() {}
