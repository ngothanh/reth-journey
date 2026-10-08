use crate::root::RootRegistry;
use std::sync::Arc;

pub struct Domain<R> {
    inner: Arc<Inner<R>>,
}

struct Inner<R> {
    scheme: R,
    roots: RootRegistry,
}

impl<R> Domain<R> {
    pub fn new(scheme: R) -> Self {
        Self {
            inner: Arc::new(Inner {
                scheme,
                roots: RootRegistry::new(),
            }),
        }
    }
}

impl<R> Clone for Domain<R> {
    fn clone(&self) -> Self {
        Domain {
            inner: self.inner.clone(),
        }
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;

    struct NotClone; // a scheme-like type that cannot be cloned

    #[test]
    fn domain_is_clone_for_any_scheme() {
        fn needs_clone<X: Clone>() {}
        needs_clone::<Domain<NotClone>>();
    }
}
