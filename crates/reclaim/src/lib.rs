// `sync` re-exports the whole atomic set the later edits need; only `AtomicPtr`
// has a user so far. Drop the `allow` once C2 uses the rest.
#[allow(unused_imports)]
mod sync;

mod retire;

pub use retire::{Retire, RetireLink};
