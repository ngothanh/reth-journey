extern crate alloc;

mod bytes;
pub mod error;
pub mod fixed_bytes;

pub use fixed_bytes::FixedBytes;
pub use bytes::{Bytes, BytesView};
