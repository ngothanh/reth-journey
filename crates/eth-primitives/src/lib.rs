extern crate alloc;
extern crate core;

mod address;
mod bytes;
pub mod error;
pub mod fixed_bytes;
mod keccak;

pub use address::Address;
pub use bytes::{Bytes, BytesView};
pub use fixed_bytes::FixedBytes;
pub type B256 = FixedBytes<32>;
