use crate::B256;
use std::sync::OnceLock;

pub trait Sealable {
    fn hash_slow(&self) -> B256;
}

struct Header {}

pub struct SealedHeader<T: Sealable> {
    inner: T,
    cache: OnceLock<B256>,
}

impl Sealable for Header {
    fn hash_slow(&self) -> B256 {
        todo!()
    }
}

impl<T: Sealable> SealedHeader<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            cache: OnceLock::new(),
        }
    }

    pub fn hash(&self) -> B256 {
        *self.cache.get_or_init(|| self.inner.hash_slow())
    }
}
