mod decode;
mod encode;
mod header;

pub use decode::{decode_exact, Decodable, Error};
pub use encode::{encode_header, Encodable};
pub use header::Header;

// Re-exported so downstream crates (and derive-generated code) can name the trait in
// `Encodable::encode`'s signature without taking a direct `bytes` dependency.
// `alloy-rlp` re-exports these for the same reason.
pub use bytes::{Buf, BufMut};
