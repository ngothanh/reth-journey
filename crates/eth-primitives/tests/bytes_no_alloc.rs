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

#[test]
fn bytes_mut_freeze_no_copy_no_alloc() {
    use eth_primitives::BytesMut;
    let mut bm = BytesMut::new(1024); // cap 1024 >> len
    bm.extend_from_slice(b"payload"); // len 7
    let ptr_before = bm.as_ptr();

    let (a0, d0) = (allocs(), deallocs());
    let b = black_box(bm.freeze()); // zero-copy AND zero-alloc handoff
    let (da, dd) = (allocs() - a0, deallocs() - d0);

    assert_eq!(b.as_ptr(), ptr_before, "freeze must reuse the buffer (zero-copy, R3)");
    assert_eq!((da, dd), (0, 0), "freeze must not allocate or free (zero-alloc, R3)");
    assert_eq!(&*b, b"payload");
    // dropping `b` DOES free the buffer (one dealloc) — outside the measured window.
}

#[test]
fn cloning_static_never_allocates() {
    let b = Bytes::from_static(b"genesis");
    let (a0, d0) = (allocs(), deallocs());
    for _ in 0..1000 {
        let c = black_box(b.clone());
        black_box(&c);
        drop(c);
    }
    let (da, dd) = (allocs() - a0, deallocs() - d0);
    assert_eq!((da, dd), (0, 0), "cloning a static Bytes 1000x must not touch the heap (R2 STATIC)");
}

#[test]
fn default_is_empty_static_no_alloc() {
    let (a0, d0) = (allocs(), deallocs());
    let b = black_box(Bytes::default());
    black_box(&b);
    let empty = b.is_empty();
    drop(b);
    let (da, dd) = (allocs() - a0, deallocs() - d0);
    assert!(empty);
    assert_eq!((da, dd), (0, 0), "Bytes::default() must be empty-static, no heap (R6)");
    assert_eq!(Bytes::default(), Bytes::from_static(b""));
}
