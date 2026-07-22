//! `BytesMut::freeze` budget bench — bar (c) anti-shortcut trap.
//!
//! `freeze` must be a *pointer-move*: hand the exact `(ptr, len, cap)` buffer to
//! `Bytes` with no copy and no allocation. The expected cost is ~2 ns (a few struct
//! moves); we set a generous 100 ns ceiling. A "correct but copying" freeze
//! (`from_raw_parts → into_boxed_slice`, which reallocs when cap > len) lands ≈250 ns
//! at 1 KiB — it passes every correctness test but FAILS this bar. Exercises R3 + R8.
//!
//! Run: `cargo bench -p eth-primitives --bench freeze`

use eth_primitives::{Bytes, BytesMut};
use std::hint::black_box;
use std::time::Instant;

const CAP: usize = 1024;
const PAYLOAD: &[u8] = b"payload";

fn fresh() -> BytesMut {
    let mut bm = BytesMut::new(CAP);
    bm.extend_from_slice(PAYLOAD);
    bm
}

/// Startup guard, BEFORE any timing: prove the handoff is zero-copy. If the ownership
/// transfer is wrong, abort loudly rather than carefully measuring a memcpy and
/// reporting it as data.
fn assert_zero_copy() {
    let bm = fresh();
    let ptr_before = bm.as_ptr();
    let frozen = bm.freeze();
    assert_eq!(
        frozen.as_ptr(),
        ptr_before,
        "freeze must reuse the buffer (pointer-move); a copying freeze fails bar (c)",
    );
}

/// Hard gate: measure freeze in isolation and assert it is under the pointer-move budget.
///
/// Isolation matters. Building each input allocates + memcpys (~the very cost we're
/// ruling out), so setup is done OUTSIDE the timed window. And each `freeze` returns an
/// OWNED `Bytes` whose Drop would `dealloc` the buffer — so we stash the results in a
/// pre-reserved Vec to keep dealloc out of the window too. What remains timed is the
/// freeze itself: `mem::forget` + struct construction.
fn assert_freeze_under_budget() {
    const ITERS: u32 = 50_000;

    let mut inputs: Vec<BytesMut> = (0..ITERS).map(|_| fresh()).collect();
    let mut out: Vec<Bytes> = Vec::with_capacity(ITERS as usize);

    let start = Instant::now();
    for bm in inputs.drain(..) {
        out.push(black_box(bm).freeze());
    }
    let per = start.elapsed() / ITERS;

    black_box(&out); // keep results alive so their Drop stays outside the timed window
    assert!(
        per.as_nanos() < 100,
        "freeze budget blown: {per:?}/op ≥ 100 ns — a pointer-move should be ~2 ns; \
         a copying freeze (~250 ns @ 1 KiB) fails bar (c)",
    );
    println!("freeze: {per:?}/op (budget < 100 ns)");
}

fn main() {
    assert_zero_copy();
    assert_freeze_under_budget();
}