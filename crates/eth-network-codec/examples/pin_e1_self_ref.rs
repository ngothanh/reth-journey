//! E1 — Self-ref bug. Motivates Pin's existence.
//!
//! Build a `SelfRef` holding `data: [u8; 4]` and `ptr: *const u8` aimed at
//! `&data[0]`. Construct two instances, `mem::swap` them, then dereference
//! both pointers. Observe the value drift — the pointers still aim at the
//! ORIGINAL memory addresses, which now hold the SWAPPED data.
//!
//! AHA: any self-referential type breaks under move semantics. Pin is the
//! type-system mechanism that makes "moving" require explicit `unsafe`.
//!
//! Run: cargo run -p eth-network-codec --example pin_e1_self_ref

struct SelfRef {
    data: [u8; 4],
    ptr: *const u8,
}

impl SelfRef {
    fn new(d: [u8; 4]) -> Self {
        Self {
            data: d,
            ptr: &d[0],
        }
    }

    fn deref_ptr(&self) -> u8 {
        // SAFETY: caller asserts the SelfRef has not been moved since `new`.
        // (Spoiler: this is the bug — we're about to violate that.)
        unsafe { *self.ptr }
    }
}

fn main() {
    let mut a = SelfRef::new([10, 20, 30, 40]);
    let mut b = SelfRef::new([50, 60, 70, 80]);

    println!(
        "before swap: a.deref()={}, b.deref()={}",
        a.deref_ptr(),
        b.deref_ptr()
    );
    // expected: a=10, b=50 (each ptr aims at its own data[0])

    std::mem::swap(&mut a, &mut b);

    println!(
        "after swap:  a.deref()={}, b.deref()={}",
        a.deref_ptr(),
        b.deref_ptr()
    );
    // EXPECTED: NOT 50/10. The data was swapped, but the pointers still
    // aim at the original memory addresses. So a.ptr aims into b's old
    // location (which now holds 10,20,30,40 after the swap)... or worse,
    // depending on stack layout. This is UB territory.
}
