use crate::{Address, Bytes, B256, B64};

mod private {
    pub trait Sealed {}
}

/// `Encodable` is sealed:
///
/// ```compile_fail
/// use eth_primitives::Encodable;
/// struct Outsider;
/// impl Encodable for Outsider {}
/// ``
pub trait Encodable: private::Sealed {}

impl private::Sealed for Address {}
impl private::Sealed for B256 {}
impl private::Sealed for B64 {}

impl private::Sealed for Bytes {}

impl Encodable for Address {}
impl Encodable for B256 {}

impl Encodable for B64 {}
impl Encodable for Bytes {}
