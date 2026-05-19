//! E4 — Manual projection failure: feel why pin_project_lite exists.
//!
//! `LoggedFuture<F>` wraps a Future, prints "polling LABEL", then delegates.
//! There are FOUR ways to write the projection. Try each in turn.
//!
//!   1. Naive: `self.inner.poll(cx)` — compile error.
//!   2. With `F: Unpin`: `Pin::new(&mut self.inner).poll(cx)` — works but
//!      restricts callers to Unpin futures (rules out compiler-generated
//!      `async {}` blocks that hold borrows across await).
//!   3. Manual unsafe: `unsafe { self.map_unchecked_mut(|s| &mut s.inner) }`.
//!      Works for any F. You promise not to move s.inner. Verbose + unsafe.
//!   4. pin_project_lite: zero unsafe, works for any F, future-refactor-safe.
//!
//! AHA: pin_project_lite generates EXACTLY what you'd write in (3), plus
//! compile-time invariant checks. There's no perf difference — the macro
//! is purely hygiene.
//!
//! Run: cargo run -p eth-network-codec --example pin_e4_projection

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct LoggedFuture<F> {
    inner: F,
    label: &'static str,
}

impl<F: Future> Future for LoggedFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("polling {}", self.label);

        // ATTEMPT 1 — uncomment, observe error:
        // self.inner.poll(cx)
        //
        // Error will say: "the method `poll` exists for type `F`, but its
        // trait bounds were not satisfied" — because you have `&mut F`, not
        // `Pin<&mut F>`, and Future::poll requires the latter.

        // ATTEMPT 2 — requires F: Unpin (add the bound to the impl block):
        // Pin::new(&mut self.inner).poll(cx)
        //
        // Compiles if F: Unpin. Try with `let fut = async { 42 };` — works
        // because that particular async block happens to be Unpin. Try with
        // an async block that borrows a local across await — fails.

        // ATTEMPT 3 — manual unsafe, works for any F:
        //
        // SAFETY: We treat `inner` as a pinned field — we only expose
        // Pin<&mut F> to call poll, never `&mut F`. We have no Drop impl
        // that moves it. No other API exposes it. Therefore taking the
        // unsafe Pin projection through map_unchecked_mut is sound.
        //
        // let inner = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
        // inner.poll(cx)

        // ATTEMPT 4 — pin_project_lite (see the sibling impl below).

        todo!("pick one of the four approaches and uncomment it")
    }
}

// ATTEMPT 4 — same impl using pin_project_lite. Uncomment the pin_project!
// block + the second Future impl to compare side-by-side. (You'll have to
// rename one of the structs since you can't have two LoggedFuture types.)
//
// use pin_project_lite::pin_project;
//
// pin_project! {
//     pub struct LoggedFutureMacro<F> {
//         #[pin]
//         inner: F,
//         label: &'static str,
//     }
// }
//
// impl<F: Future> Future for LoggedFutureMacro<F> {
//     type Output = F::Output;
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let this = self.project();
//         println!("polling {}", this.label);
//         this.inner.poll(cx)
//     }
// }

#[tokio::main]
async fn main() {
    let fut = async { 42 };
    let logged = LoggedFuture {
        inner: fut,
        label: "the-test",
    };
    let result = logged.await;
    println!("result: {result}");
}
