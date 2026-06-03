// R4: the proc-macro crate exports ONLY the derive macro, never a trait. Using
// `eth_primitives_derive::SimpleEncode` as a trait bound must fail — the trait
// lives in `eth_primitives`, the macro in `eth_primitives_derive`.
fn needs_trait<T: eth_primitives_derive::SimpleEncode>(_t: &T) {}

fn main() {}
