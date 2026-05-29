// expect-fail: Parker is deliberately !Clone (single-consumer invariant).
fn main() {
    let p1 = concurrent::Parker::new();
    let p2 = p1.clone();
    let _ = p2;
}
