extern crate alloc;
extern crate core;

mod bytes;
pub mod error;
pub mod fixed_bytes;

pub use bytes::{Bytes, BytesView};
pub use fixed_bytes::FixedBytes;
