# Code + terminal cards for the `reclaim` series.
# Render: python3 <blog-skill>/scripts/cards.py img/cards_data.py img/src/cards
#         node <blog-skill>/scripts/render.mjs img/src/cards/<name>.html img/cards/<name>.png
# Comments with ①② render amber (writer / publish side), ③④ teal (reader side).
CARDS = {
"refcount_guard": ("refcount.rs — the obvious fix, which cannot work",
'''struct Node<T> {
    value: T,
    next: *mut Node<T>,
    readers: AtomicUsize,          // ① the announcement lives INSIDE the node
}

struct ReadGuard<T> { node: *mut Node<T> }

impl<T> ReadGuard<T> {
    fn acquire(node: *mut Node<T>) -> Self {
        // ② the first touch of the node is a write into it
        unsafe { (*node).readers.fetch_add(1, Relaxed) };
        ReadGuard { node }
    }
}

impl<T> Drop for ReadGuard<T> {
    fn drop(&mut self) {
        unsafe { (*self.node).readers.fetch_sub(1, Relaxed) };
    }
}

// the thread that popped `node`, after moving head away from it:
if unsafe { (*node).readers.load(Acquire) } == 0 {
    unsafe { drop(Box::from_raw(node)) };   // ✗ a reader may hold it but not have announced yet
}''', True),

"root_protect": ("root.rs + guard.rs — only a declared root can be passed to try_protect",
'''/// The address OF one place readers start from. One word, Copy.
pub struct Root<T> {
    src: *const AtomicPtr<T>,
}

impl<R> Domain<R> {
    /// # Safety
    /// `place` stays alive, and at the same address, until `remove_root`.
    pub unsafe fn declare_root<T>(&self, place: &AtomicPtr<T>) -> Root<T>;
    pub fn remove_root<T>(&self, root: Root<T>);
}

pub trait Guard {
    fn try_protect<T>(&mut self, addr: *mut T, root: &Root<T>)   // ① a Root, never a bare AtomicPtr
        -> Result<*mut T, *mut T>;
}

// guard.try_protect(p, &head_root)    compiles
// guard.try_protect(p, &cur.next)     does not: &AtomicPtr<T> is not &Root<T>''', True),

"guard_trait": ("guard.rs — the whole reader-side interface",
'''pub trait Guard {
    /// Announce `addr`, then check that `root` still holds it.
    /// Ok(addr):     `addr` is protected and may be opened.
    /// Err(current): the root moved on; do not open `addr`, retry with `current`.
    fn try_protect<T>(&mut self, addr: *mut T, root: &Root<T>)   // ① &mut self: reuse one guard
        -> Result<*mut T, *mut T>;                                // ② a raw pointer, not &'g T

    /// Provided: the retry loop, written once, for callers with nothing better to do.
    fn protect<T>(&mut self, root: &Root<T>) -> *mut T {
        let mut addr = root.ptr().load(Ordering::Acquire);
        loop {
            match self.try_protect(addr, root) {
                Ok(p) => return p,
                Err(current) => addr = current,
            }
        }
    }
}''', True),

"retirable": ("retire.rs — what an object must offer to be freed by someone else",
'''/// # Safety
/// The implementor promises:
/// 1. `retire_link` returns a field of this same object, shared with no other object;
/// 2. it returns the same link every time;
/// 3. from `retire(p)` until `reclaim(p)` returns, the link does not move,
///    and the object does not touch it;
/// 4. `reclaim` really frees the object, the way it was allocated, and does not
///    make it reachable again.
pub unsafe trait Retirable: Send {           // Send: freed on another thread
    /// The link the scheme uses to chain this object into its list.
    fn retire_link(&self) -> &RetireLink;

    /// Frees the object. The scheme decides WHEN; this knows HOW.
    /// No default: a Box default is silently wrong for a pool page.
    unsafe fn reclaim(ptr: *mut Self);
}''', True),

"domain": ("domain.rs + reclaimer.rs — the handle a structure holds",
'''pub unsafe trait Reclaimer {                 // implementor promises Rule 2
    type Guard: Guard;
    fn guard(&self) -> Self::Guard;
    unsafe fn retire<T: Retirable>(&self, obj: *mut T);   // caller promises Rule 1
}

/// One scheme plus its declared roots. Cheap to clone: an Arc inside.
pub struct Domain<R> {
    inner: Arc<Inner<R>>,                    // { scheme: R, roots: RootRegistry }
}

impl<R: Reclaimer> Domain<R> {
    pub unsafe fn declare_root<T>(&self, place: &AtomicPtr<T>) -> Root<T>;
    pub fn remove_root<T>(&self, root: Root<T>);
    pub fn guard(&self) -> R::Guard;
    pub unsafe fn retire<T: Retirable>(&self, obj: *mut T);  // debug: assert no root holds obj
}''', True),

"treiber_pop": ("reclaimer.rs (doc-test) — the first client: a Treiber stack's pop, on any scheme",
'''fn pop<R: Reclaimer>(domain: &Domain<R>, head: &AtomicPtr<Node>, root: &Root<Node>)
    -> Option<u64>
{
    let mut guard = domain.guard();                    // a reader takes a guard first
    loop {
        let top = guard.protect(root);                 // read the root, protect what it holds
        if top.is_null() {
            return None;
        }
        // SAFETY: not null, and protected by `guard`.
        let next = unsafe { (*top).next };

        if head.compare_exchange_weak(top, next, Release, Relaxed).is_ok() {
            // SAFETY: still protected by `guard`.
            let value = unsafe { (*top).value };
            // SAFETY: head no longer holds `top` and is the only root (Rule 1);
            // only the winner of the exchange gets here (retired once);
            // `top` is not used after this line.
            unsafe { domain.retire(top) };
            return Some(value);
        }
        // another thread moved head first: protect again
    }
}''', True),

"leak_test": ("leak.rs — a test that Leak never frees anything",
'''#[test]
fn a_retired_object_is_never_reclaimed() {
    let drops = Arc::new(AtomicUsize::new(0));
    {
        let leak = Leak::new();
        for _ in 0..3 {
            let node = Box::into_raw(Box::new(Node {
                link: RetireLink::new(),
                drops: Arc::clone(&drops),
            }));
            // SAFETY: no root holds it, retired once, not used again.
            unsafe { leak.retire(node) };
        }
    } // the scheme goes away here

    // Nothing was freed while the scheme lived, or when it went away.
    assert_eq!(drops.load(Ordering::Relaxed), 0);
}''', True),

"retire_link": ("retire.rs — three words per retirable object",
'''pub struct RetireLink {
    next: AtomicPtr<RetireLink>,                      // the next link in the pile
    obj: UnsafeCell<*mut ()>,                         // where the object starts: 900
    reclaim: UnsafeCell<Option<unsafe fn(*mut ())>>,  // which function frees it: 7000
}

/// One copy per type: reclaim_one::<Page>, reclaim_one::<Node>, ...
/// All copies have the same type, so any of them fits in the field.
unsafe fn reclaim_one<T: Retirable>(obj: *mut ()) {
    unsafe { T::reclaim(obj.cast::<T>()) }
}

#[test]
fn retire_link_is_3_words() {        // Option<fn> stores None as 0: still one word
    assert_eq!(size_of::<RetireLink>(), 3 * size_of::<usize>());
}''', True),

"pile_push": ("retire.rs — push: a Treiber push with two writes in front",
'''pub(crate) unsafe fn push<T: Retirable>(&self, obj: *mut T) {
    let link: &RetireLink = unsafe { (*obj).retire_link() };
    let link_ptr = link as *const RetireLink as *mut RetireLink;

    let in_a_pile = link.reclaim.with(|p| unsafe { (*p).is_some() });
    assert!(!in_a_pile, "object retired twice");

    link.obj.with_mut(|p| unsafe { *p = obj.cast() });                 // ① written before
    link.reclaim.with_mut(|p| unsafe { *p = Some(reclaim_one::<T>) }); // ① the link is visible

    let mut top = self.head.load(Ordering::Relaxed);
    loop {
        link.next.store(top, Ordering::Relaxed);                       // ① still private
        match self.head.compare_exchange_weak(
            top, link_ptr, Ordering::Release, Ordering::Relaxed,       // ② publish
        ) {
            Ok(_) => return,
            Err(current) => top = current,
        }
    }
}''', True),

"free_all": ("retire.rs — take everything at once, then free link by link",
'''pub(crate) fn take_all(&self) -> *mut RetireLink {
    self.head.swap(null_mut(), Ordering::Acquire)       // ③ own the whole chain, then open it
}

pub(crate) unsafe fn free_all(mut link: *mut RetireLink) {
    while !link.is_null() {
        let next = unsafe { (*link).next.load(Ordering::Relaxed) };   // ④ BEFORE reclaim
        let obj = unsafe { (*link).obj.with(|p| *p) };
        let reclaim = unsafe { (*link).reclaim.with(|p| *p).unwrap() };

        // give the link back empty, so a pooled object can be retired again;
        // must happen before reclaim: after it the link is freed memory
        unsafe { (*link).obj.with_mut(|p| *p = null_mut()) };
        unsafe { (*link).reclaim.with_mut(|p| *p = None) };

        unsafe { reclaim(obj) };                         // the link was inside obj: gone now
        link = next;
    }
}''', True),

"loom_two_push_one_taker": ("retire.rs — the loom model: two pushers, one concurrent taker",
'''#[test]
fn two_push_one_taker() {
    loom::model(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let list = Arc::new(RetiredList::new());

        let pushers = (0..2)
            .map(|i| {
                let (list, drops) = (list.clone(), drops.clone());
                loom::thread::spawn(move || unsafe {
                    list.push(Box::into_raw(Node::boxed(i, &drops)));
                })
            })
            .collect::<Vec<_>>();

        unsafe { RetiredList::free_all(list.take_all()) };   // races with the pushers

        for p in pushers {
            p.join().unwrap();
        }
        unsafe { RetiredList::free_all(list.take_all()) };   // whatever was left
        assert_eq!(drops.load(Ordering::Relaxed), 2);         // each freed exactly once
    })
}''', True),

"bench_harness": ("benches/retired_list.rs — only pushing is timed",
'''fn timed_pushes<P: Pile>(threads: usize, iters: u64) -> Duration {
    let per_thread = (iters as usize).div_ceil(threads);
    let pile = P::new();
    let barrier = Barrier::new(threads + 1);

    // untimed: every node exists before the clock starts
    let batches: Vec<Vec<SendPtr>> = allocate_nodes(threads, per_thread);

    let start = std::thread::scope(|scope| {
        for (thread, batch) in batches.into_iter().enumerate() {
            let (pile, barrier) = (&pile, &barrier);
            scope.spawn(move || {
                barrier.wait();
                for group in batch.chunks(CHAIN_LEN) {
                    unsafe { pile.push_many(thread, group) };
                }
            });
        }
        barrier.wait();
        Instant::now()            // the scope joins every pusher before it returns
    });
    let elapsed = start.elapsed();

    pile.free_everything();       // untimed: take + free

    elapsed.mul_f64(iters as f64 / (per_thread * threads) as f64)
}''', True),

"chain": ("retire.rs — a chain built privately, attached with one exchange",
'''pub(crate) struct Chain {
    top: *mut RetireLink,         // newest link
    bottom: *mut RetireLink,      // oldest link: the only one that points into the pile
}

impl Chain {
    pub(crate) unsafe fn add<T: Retirable>(&mut self, obj: *mut T) {
        let link: &RetireLink = unsafe { (*obj).retire_link() };
        let link_ptr = link as *const RetireLink as *mut RetireLink;
        // ... the same double-retire check and obj / reclaim writes as push ...
        if self.top.is_null() {
            self.bottom = link_ptr;                         // the first link stays the bottom
        } else {
            link.next.store(self.top, Ordering::Relaxed);   // private: Relaxed is enough
        }
        self.top = link_ptr;
    }
}

impl RetiredList {
    pub(crate) fn push_chain(&self, chain: Chain) {
        if chain.top.is_null() { return; }
        let mut top = self.head.load(Ordering::Relaxed);
        loop {
            // ① a retry changes only the bottom's `next`, to the current `head`; the links above never change
            unsafe { (*chain.bottom).next.store(top, Ordering::Relaxed) };
            match self.head.compare_exchange_weak(
                top, chain.top, Ordering::Release, Ordering::Relaxed,   // ② publishes the whole chain
            ) {
                Ok(_) => return,
                Err(current) => top = current,
            }
        }
    }
}''', True),
}

TERMS = {
"term_miri_moved_root": ("cargo +nightly miri test — a root declared on a local variable, then moved",
'''<span class="pr">$</span> cargo +nightly miri test -p reclaim
<span class="dim">...</span>
<span class="err">error: Undefined Behavior:</span> constructing invalid value of type &amp;AtomicPtr&lt;Node&gt;:
       encountered a dangling reference (use-after-free)
  <span class="pt">--&gt;</span> reclaim/src/root.rs:59     <span class="dim">Root::ptr: the root still points into new's stack frame</span>'''),

"term_miri_retag": ("cargo +nightly miri test — the place in a Box (shortened)",
'''<span class="pr">$</span> cargo +nightly miri test
<span class="err">error: Undefined Behavior:</span> trying to retag from &lt;tag&gt; … but that tag
       does not exist in the borrow stack for this location
  <span class="pt">--&gt;</span> reclaim/src/root.rs:59        <span class="dim">Root::ptr, from protect</span>

<span class="warn">help:</span> &lt;tag&gt; was created by a SharedReadWrite retag    <span class="dim">root.rs:53  (declare_root)</span>
<span class="warn">help:</span> &lt;tag&gt; was later invalidated by a Unique retag   <span class="dim">(the Box moved into the struct)</span>

<span class="dim">same code with</span> MIRIFLAGS=-Zmiri-tree-borrows <span class="dim">→</span> <span class="ok">test result: ok</span>'''),

"term_loom_causality": ("LOOM_LOCATION=1 cargo test --release — take_all with Release instead of Acquire",
'''<span class="pr">$</span> RUSTFLAGS="--cfg loom" LOOM_LOCATION=1 cargo test -p reclaim --release --lib retire::loom_tests
<span class="dim">running 1 test</span>
thread 'retire::loom_tests::two_push_one_taker' panicked at loom-0.7.2/src/rt/location.rs:115:9:
<span class="err">Causality violation: Concurrent load and mut accesses.</span>
    created: <span class="pt">crates/reclaim/src/retire.rs:61:19</span>          <span class="dim">RetireLink::new</span>
    load: thread #0 @ <span class="pt">crates/reclaim/src/retire.rs:147:46</span>   <span class="dim">first read of next in free_all</span>
<span class="dim">test retire::loom_tests::two_push_one_taker ...</span> <span class="err">FAILED</span>'''),
}
