use eth_primitives_derive::RlpEncodable;

#[derive(RlpEncodable)]
union U {
    a: u64,
    b: u64,
}

fn main() {}
