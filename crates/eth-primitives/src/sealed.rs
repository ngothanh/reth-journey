use crate::B256;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::OnceLock;

pub trait Sealable {
    fn hash_slow(&self) -> B256;
}

pub struct Sealed<T: Sealable> {
    inner: T,
    hash: OnceLock<B256>,
}

impl<T: Sealable> Sealed<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            hash: OnceLock::new(),
        }
    }

    pub fn new_unchecked(inner: T, hash: B256) -> Self {
        Self {
            inner,
            hash: OnceLock::from(hash),
        }
    }

    pub fn hash(&self) -> B256 {
        *self.hash.get_or_init(|| self.inner.hash_slow())
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Sealable> Deref for Sealed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Sealable> PartialEq for Sealed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

impl<T: Sealable> Eq for Sealed<T> {}

impl<T: Sealable> Hash for Sealed<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.hash().as_ref());
    }
}

impl<T: Sealable + fmt::Debug> fmt::Debug for Sealed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sealed")
            .field("inner", &self.inner)
            .field("hash", &self.hash.get())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{keccak256, Sealable, Sealed, B256};
    use std::collections::HashSet;

    #[derive(Debug)]
    struct Test {
        name: String,
    }

    impl Sealable for Test {
        fn hash_slow(&self) -> B256 {
            keccak256(self.name.as_bytes())
        }
    }

    impl Test {
        fn new(name: &str) -> Self {
            Self { name: name.into() }
        }
    }

    // Pitfall #2: if PartialEq/Hash included the OnceLock state, a warm-cache and a
    // cold-cache Sealed wrapping equal inner would be considered distinct and a
    // HashSet would store both.
    #[test]
    fn hashset_dedup_across_warm_and_cold_cache() {
        let s1 = Sealed::new(Test::new("tngo"));
        let s2 = Sealed::new(Test::new("tngo"));
        s1.hash();

        let mut set = HashSet::new();
        set.insert(s1);
        set.insert(s2);

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn hashset_keeps_distinct_inner() {
        let s1 = Sealed::new(Test::new("alice"));
        let s2 = Sealed::new(Test::new("bob"));

        let mut set = HashSet::new();
        set.insert(s1);
        set.insert(s2);

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn new_unchecked_matches_lazy_hash() {
        let lazy = Sealed::new(Test::new("tngo"));
        let precomputed = Sealed::new_unchecked(Test::new("tngo"), keccak256(b"tngo"));

        assert_eq!(lazy, precomputed);
    }

    #[test]
    fn into_inner_returns_wrapped_value() {
        let sealed = Sealed::new(Test::new("tngo"));
        let inner = sealed.into_inner();
        assert_eq!(inner.name, "tngo");
    }
}
