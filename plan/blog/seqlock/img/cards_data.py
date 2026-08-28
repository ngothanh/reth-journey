# (title, code, line_numbers)
CARDS = {

# ---- Part 2 ----
"rwlock_read": ("RwLock::read — every reader writes shared state",
'''// A reader must announce itself so the writer knows to wait.
let count = self.readers.fetch_add(1, Acquire);  // shared WRITE
let value = /* ... read T ... */;
self.readers.fetch_sub(1, Release);              // shared WRITE
// N readers on N cores all hammer `self.readers`:
// one cache line, bouncing between every core (MESI).''', True),

# ---- Part 3 ----
"bool_flag": ("A single \"writing\" flag — and why it tears",
'''fn store(&self, v: T) {
    self.writing.store(true, Release);   // "I'm writing"
    self.data = v;
    self.writing.store(false, Release);  // "done"
}
fn load(&self) -> T {
    while self.writing.load(Acquire) {}  // wait until not writing
    self.data                            // BUG: a whole write can start
}                                        // and finish between these two lines''', True),

"protocol": ("The odd/even protocol",
'''writer:                        reader:
  seq += 1   // even -> odd        s1 = seq
  write payload                    if s1 is odd { retry }
  seq += 1   // odd  -> even        copy payload
                                   s2 = seq
                                   if s1 == s2 { accept } else { retry }''', False),

# ---- Part 4 ----
"all_relaxed": ("Correct on paper, torn on ARM",
'''// Every access Relaxed: atomic, but NO ordering.
self.seq.fetch_add(1, Relaxed);      // open
self.data = value;                   // payload
self.seq.fetch_add(1, Relaxed);      // close
// Nothing pins the payload inside the window. On a weak
// memory model the store can drift out either end. Tears.''', True),

"wrong_release": ("Release on the bump guards the wrong side",
'''self.seq.fetch_add(1, Release);   // WRONG: Release floors what's BEFORE
self.data = value;                // payload is AFTER -> not covered
// The payload can still float up above the bump.
// Release on this op locks a door nobody walks through.''', True),

"four_gates": ("seq_lock.rs — the four gates",
'''pub fn store(&self, value: T) {
    self.state.fetch_add(1, Relaxed);   // open: even -> odd
    fence(Ordering::Release);           // ① keep payload from floating up
    // ... write payload words ...
    self.state.fetch_add(1, Release);   // ② keep payload from sinking down
}

pub fn load(&self) -> T {
    let first = self.state.load(Acquire);  // ③ keep copy from floating up
    if first % 2 == 1 { continue; }
    // ... copy payload words ...
    fence(Ordering::Acquire);              // ④ keep copy from sinking down
    let second = self.state.load(Relaxed);
    if first == second { return value; }
}''', True),

# ---- Part 5 ----
"naive_read": ("The read the memory model forbids",
'''let s1 = self.seq.load(Acquire);
let value = unsafe { ptr::read(self.data.get()) };  // plain read
let s2 = self.seq.load(Relaxed);
// data race: this non-atomic read races the writer's
// non-atomic write. Data race == UB. Not "garbage" -- UB.''', True),

"atomic_words": ("Read the payload one atomic word at a time",
'''let words = size_of::<T>() / size_of::<usize>();
let src = self.data.get() as *const usize;
for i in 0..words {
    let slot = unsafe { &*(src.add(i) as *const AtomicUsize) };
    let word = slot.load(Relaxed);   // atomic: legal to race
    unsafe { dst.add(i).write(word); }
}
// Words may still tear against each other -- that's fine,
// the seq catches it. Atomic just makes the race LEGAL.''', True),

"pod_bound": ("Pod is one gate; size + align is the other",
'''pub unsafe trait Pod: Copy {}   // no padding, every bit pattern
                               // valid, defined layout -- a promise
                               // the implementer signs, not checked.

fn require() {
    const {
        assert!(size_of::<T>() % size_of::<usize>() == 0);
        assert!(align_of::<T>() >= align_of::<usize>());
    }
}
// u8 is a perfectly valid Pod -- and still fails here.
// Pod does NOT cover alignment. Two independent gates.''', True),

# ---- Part 6 ----
"writer_cas": ("The seq is also the writers' lock",
'''// Acquire the write slot: CAS only from an EVEN value.
let mut cur = self.state.load(Relaxed);
loop {
    if cur % 2 == 1 {           // odd = another writer holds it
        spin_loop();
        cur = self.state.load(Relaxed);
        continue;
    }
    match self.state.compare_exchange_weak(
        cur, cur + 1, Relaxed, Relaxed,
    ) {
        Ok(_) => break,         // won: cur is now odd, we own it
        Err(now) => cur = now,  // lost the race, retry
    }
}''', True),

"torn_test": ("The test that must catch a tear",
'''// The writer only ever publishes [n, n, n, n].
// Any load whose four words differ is a torn read.
let v = lock.load();
assert!(
    v[0] == v[1] && v[1] == v[2] && v[2] == v[3],
    "torn read: {v:?}"
);''', True),
}
