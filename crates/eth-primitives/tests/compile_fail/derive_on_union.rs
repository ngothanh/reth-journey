// R6: deriving SimpleEncode on a union must produce a clean compile_error.
use eth_primitives::SimpleEncode;

#[derive(SimpleEncode)]
union U {
    a: u32,
    b: f32,
}

fn main() {}
