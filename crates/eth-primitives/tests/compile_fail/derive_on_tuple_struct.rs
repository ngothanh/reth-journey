// R6: deriving SimpleEncode on a tuple (unnamed-field) struct must produce a
// clean compile_error — only named-field structs are supported today.
use eth_primitives::SimpleEncode;

#[derive(SimpleEncode)]
struct T(u8, u8);

fn main() {}
