//! Proves the slow-path `AtomicCell::<[u8; 16]>` store loop is allocation-free
//! (the timed region of the 4-thread bench), so heap allocation is NOT what
//! inflates the contended p99.
//!
//!   cargo run --release --example atomic_cell_alloc_check
//!
//! Wraps the global allocator with a counter and reports allocations made
//! *during* a 1e6-store loop. Expected: 0.

use concurrent::AtomicCell;
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

struct Counting;
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn main() {
    let cell = AtomicCell::new([0u8; 16]);

    let before = ALLOCS.load(Ordering::Relaxed);
    for i in 0..1_000_000u64 {
        cell.store(black_box([i as u8; 16]));
    }
    black_box(cell.load());
    let after = ALLOCS.load(Ordering::Relaxed);

    println!("allocations during 1_000_000 stores: {}", after - before);
}
