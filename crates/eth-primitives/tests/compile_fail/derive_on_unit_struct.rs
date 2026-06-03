// R6: deriving SimpleEncode on a unit struct must produce a clean compile_error.
use eth_primitives::SimpleEncode;

#[derive(SimpleEncode)]
struct U;

fn main() {}
