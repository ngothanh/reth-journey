use eth_primitives::Bytes;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;

thread_local! {
    static ALLOCS: Cell<usize> = const {Cell::new(0)};
    static DEALLOCS: Cell<usize> = const {Cell::new(0)};
}

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.with(|c| c.set(c.get() + 1));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCS.with(|c| c.set(c.get() + 1));
        unsafe { System.dealloc(ptr, layout) }
    }
}

fn allocs() -> usize {
    ALLOCS.with(Cell::get)
}
fn deallocs() -> usize {
    DEALLOCS.with(Cell::get)
}

#[global_allocator]
static GLOBAL: Counting = Counting;

#[test]
fn bytes_clone_static_is_bitwise() {
    let (a0, d0) = (allocs(), deallocs());
    let bytes = black_box(Bytes::from_static(b"genesis"));
    black_box(&bytes);
    drop(bytes);

    let (da, dd) = (allocs() - a0, deallocs() - d0);
    assert_eq!((da, dd), (0, 0), "from_static + drop must not touch the heap (R5)");
}
