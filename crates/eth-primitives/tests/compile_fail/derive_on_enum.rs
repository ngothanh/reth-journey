// R6: deriving SimpleEncode on an enum must produce a clean compile_error
// pinned to the input, not a proc-macro panic.
use eth_primitives::SimpleEncode;

#[derive(SimpleEncode)]
enum E {
    A,
    B,
}

fn main() {}
