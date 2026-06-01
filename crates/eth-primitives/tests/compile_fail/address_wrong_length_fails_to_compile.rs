// R3 (Address width): an `address!` literal that decodes to the wrong number of
// bytes must fail at COMPILE time. `"0x0001"` is 2 bytes, not 20, so
// `decode_hex::<20>` sees real_length 4 != 40 and panics during const
// evaluation.
//
// `const` context is load-bearing: it forces the panic to fire at compile time
// rather than runtime. See b256_short_literal_fails_to_compile.rs.
use eth_primitives::{address, Address};

const _SHORT: Address = address!("0x0001");

fn main() {}
