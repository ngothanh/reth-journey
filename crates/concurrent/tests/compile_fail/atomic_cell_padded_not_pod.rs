//! R4 — the load-bearing artifact: a padded 8-byte type must be REJECTED at
//! compile time, not silently fed through the `AtomicU64` fast path (UB).
//!
//! `Padded` is size 8, align 8 — it passes the old `size_of`/`align_of` gate —
//! but has 2 bytes of tail padding, so it can never be `Pod`. `AtomicCell<T>`
//! is bound on `T: Pod`, so even naming `AtomicCell::<Padded>` fails to compile.
use concurrent::AtomicCell;

#[repr(C, align(8))]
#[derive(Clone, Copy)]
struct Padded {
    a: u32,
    b: u16,
}

fn main() {
    let cell = AtomicCell::new(Padded { a: 1, b: 2 });
    cell.store(Padded { a: 3, b: 4 });
}
