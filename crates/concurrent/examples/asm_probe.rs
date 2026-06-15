//! `#[no_mangle]` probes that force the generic `AtomicCell::<u64>` fast path to
//! be monomorphized under a stable symbol, so `cargo asm` can disassemble it:
//!
//!   cargo asm --example asm_probe --release probe_fast_store
//!   cargo asm --example asm_probe --release probe_fast_load
//!
//! Expected on aarch64: a single `stlr` (store-release) / `ldar` (load-acquire),
//! no branch into the spinlock and no CAS loop. Output is committed to
//! `notes/atomic_cell_fast_path.asm.txt` (R3).

use concurrent::AtomicCell;
use std::hint::black_box;

#[no_mangle]
pub fn probe_fast_store(cell: &AtomicCell<u64>, v: u64) {
    cell.store(v);
}

#[no_mangle]
pub fn probe_fast_load(cell: &AtomicCell<u64>) -> u64 {
    cell.load()
}

fn main() {
    let cell = AtomicCell::new(0u64);
    probe_fast_store(&cell, black_box(7));
    black_box(probe_fast_load(&cell));
}
