// The shim has no user until C2 (the intrusive retired list) — C1 is
// deliberately concurrency-free. Drop the `allow` when C2 lands.
#[allow(unused_imports)]
mod sync;
