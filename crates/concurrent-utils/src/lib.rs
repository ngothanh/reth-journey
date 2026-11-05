//! Small building blocks shared by `concurrent` and `reclaim`.
//!
//! The same split as crossbeam: `crossbeam-utils` holds `CachePadded` and
//! `Backoff`, and both `crossbeam-epoch` and `crossbeam-queue` depend on it.
//! Here `concurrent` depends on `reclaim`, so anything both of them need has to
//! live below both, in this crate.

mod backoff;
mod cache_padded;

pub use backoff::Backoff;
pub use cache_padded::CachePadded;
