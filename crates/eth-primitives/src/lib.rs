extern crate alloc;
extern crate core;

mod address;
mod aliases;
mod bytes;
mod bytes_mut;
mod encodable;
mod error;
mod fixed_bytes;
mod hex;
mod keccak;
mod sealed;
mod uint;
#[macro_use]
mod macros;
mod simple_encode;

pub use address::{parse_address, Address};
pub use aliases::{B256, B64};
pub use bytes::{Bytes, BytesView};
pub use bytes_mut::BytesMut;
pub use encodable::Encodable;
pub use error::PrimitivesError;
pub use eth_primitives_derive::SimpleEncode;
pub use fixed_bytes::FixedBytes;
pub use keccak::keccak256;
pub use sealed::{Sealable, Sealed, SealedRef};
pub use simple_encode::SimpleEncode;
pub use uint::{U256Ext, U256};
