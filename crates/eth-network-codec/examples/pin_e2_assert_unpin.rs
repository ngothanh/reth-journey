//! E2 — Confirm Unpin is the default. PhantomPinned is the off-switch.
//!
//! Write `fn assert_unpin<T: Unpin>()` and call it for a bunch of common
//! types. They all compile. Then add a struct holding `PhantomPinned` and
//! observe the compile error.
//!
//! AHA: Unpin is auto-derived for any type whose fields are all Unpin.
//! Almost everything you've ever touched is Unpin. PhantomPinned is the
//! one zero-cost mechanism to opt out.
//!
//! Run: cargo run -p eth-network-codec --example pin_e2_assert_unpin

use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomPinned;
use tokio::net::TcpStream;

/// Trick: T: Unpin is checked at the call site. If T is not Unpin, the
/// caller fails to compile.
fn assert_unpin<T: Unpin>() {}

fn main() {
    // TODO: call assert_unpin for each of these. All should compile.
    //   u32
    //   String
    //   Vec<u8>
    //   HashMap<String, u32>
    //   TcpStream
    //   Box<dyn std::fmt::Display>
    //
    // Each call is one line: `assert_unpin::<TYPE>();`
    assert_unpin::<u32>();
    assert_unpin::<String>();
    assert_unpin::<Vec<u8>>();
    assert_unpin::<HashMap<String, u32>>();
    assert_unpin::<TcpStream>();
    assert_unpin::<Box<dyn Display>>();

    let _ = HashMap::<String, u32>::new(); // silence the unused-import warning
    let _: Option<TcpStream> = None;

    // Define a struct holding PhantomPinned and try to assert it's Unpin.
    // It should FAIL to compile. Comment out the assert call to keep this
    // file building, then UNCOMMENT to feel the error.
    struct Pinned {
        _data: u32,
        _p: PhantomPinned,
    }
    // assert_unpin::<Pinned>();
    // ↑ uncomment, observe error: "the trait bound `Pinned: Unpin` is not satisfied"

    println!("all assertions passed (you should have uncommented the Pinned one)");
}
