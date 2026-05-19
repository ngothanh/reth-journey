//! E3 — Pin elision: Pin<&mut T> → &mut T is FREE when T: Unpin.
//!
//! Write `fn touch<T: Unpin>(p: Pin<&mut T>) { let _: &mut T = p.get_mut(); }`.
//! Compiles. Then DROP the `Unpin` bound and observe the error: get_mut is
//! a method that only exists for Unpin types.
//!
//! AHA: pinning is dormant for Unpin types. All the ceremony only has teeth
//! when T: !Unpin (i.e. there's a self-reference somewhere).
//!
//! Run: cargo run -p eth-network-codec --example pin_e3_elision

use std::pin::Pin;

fn touch<T: Unpin>(p: Pin<&mut T>) {
    // TODO: call p.get_mut() and bind it to a `&mut T`. Then mutate the value
    // through that reference (e.g. set it to 999 if T is u32). Show that
    // through Pin, you reached &mut T with zero ceremony.
}

// VARIANT: drop the Unpin bound. Try to compile.
//
// fn touch_anything<T>(p: Pin<&mut T>) {
//     let _: &mut T = p.get_mut();
//     // ↑ error: get_mut requires T: Unpin
//     //   For a non-Unpin T, you'd need `unsafe { p.get_unchecked_mut() }`
//     //   and you'd be on the hook for the promise.
// }

fn main() {
    let mut x = 42u32;
    touch(Pin::new(&mut x));
    println!("x after touch: {x}");

    // TODO: also try Pin::new on a String or Vec<u8>, call touch. Same story.
}
