//! E5 — Build `Map<F, G>`, the canonical futures-util combinator shape.
//!
//! `Map<F, G>` wraps a Future F. When F resolves with value V, calls G(V)
//! and produces the result. This is `Future::map` from futures-util.
//!
//! Key design points:
//!   - `future: F` is `#[pin]` (we need to poll it).
//!   - `op: Option<G>` (NOT just G). G is FnOnce — we can only call it
//!     once. After we call it on Ready, we take() it from the Option,
//!     leaving None. If anyone polls after we returned Ready, we panic.
//!   - The struct must not be polled again after returning Ready —
//!     standard Future contract.
//!
//! AHA: every futures-util combinator (Then, AndThen, Inspect, MapErr,
//! OrElse, ...) has this exact shape. Master this and you can reimplement
//! any of them in 15 minutes.
//!
//! Run: cargo run -p eth-network-codec --example pin_e5_map

use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll::Pending;
use std::task::{Context, Poll};
use Poll::Ready;

pin_project! {
    pub struct Map<F, G> {
        #[pin]
        future: F,
        // TODO: explain to yourself WHY Option<G> instead of just G.
        op: Option<G>,
    }
}

impl<F, G> Map<F, G> {
    pub fn new(future: F, op: G) -> Self {
        Self {
            future,
            op: Some(op),
        }
    }
}

impl<F, G, T> Future for Map<F, G>
where
    F: Future,
    G: FnOnce(F::Output) -> T,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO:
        //   1. Poll the inner future via `this.future.poll(cx)`.
        //   2. On Poll::Pending → return Pending.
        //   3. On Poll::Ready(val):
        //      a. Take the closure: `let op = this.op.take().expect("polled after Ready")`
        //      b. Apply it: return `Poll::Ready(op(val))`.
        let this = self.project();
        match this.future.poll(cx) {
            Ready(val) => {
                let op = this.op.take().expect("polled after ready");
                Ready(op(val))
            }
            Pending => Pending,
        }
    }
}

#[tokio::main]
async fn main() {
    let fut = async { 7_u32 };
    let mapped = Map::new(fut, |x| x * 2);
    let result = mapped.await;
    assert_eq!(result, 14);
    println!("Map worked: {result}");

    // BONUS: chain two Maps. Map<Map<async{...}, _>, _>. Confirm it composes.
    let chained = Map::new(Map::new(async { 3_u32 }, |x| x + 1), |y| y * 10);
    let result = chained.await;
    assert_eq!(result, 40);
    println!("Chained Map worked: {result}");
}
