#![cfg(not(loom))]

use reclaim::{Domain, Guard, Leak, Retirable, RetireLink, Root};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

struct Node {
    next: *mut Node,
    link: RetireLink,
    value: u64,
}

// SAFETY: a `Node` owns only its own memory. `next` is just the address of
// the node below; no safe code reads through it, and dropping a `Node` does
// not touch that node. Code that does read through `next` is `unsafe` and
// must protect the node with a guard first.
unsafe impl Send for Node {}

unsafe impl Retirable for Node {
    fn retire_link(&self) -> &RetireLink {
        &self.link
    }

    unsafe fn reclaim(ptr: *mut Self) {
        drop(
            //SAFETY: The caller promises to call this only when no one has access to it.
            unsafe { Box::from_raw(ptr) },
        );
    }
}

/// Puts a new node on top of the stack.
///
/// `push` takes no guard. It never opens the old top: it only copies the
/// address into the new node and compares it with `head`. An address can be
/// copied and compared even if the node behind it is already gone.
fn push(head: &AtomicPtr<Node>, value: u64) {
    let new_node = Box::into_raw(Box::new(Node {
        next: null_mut(),
        link: RetireLink::new(),
        value,
    }));
    let mut top = head.load(Ordering::Relaxed);
    loop {
        // SAFETY: `new_node` came from `Box::into_raw` just above and `head`
        // does not point to it yet, so this thread is the only one that can
        // reach it.
        unsafe { (*new_node).next = top };

        match head.compare_exchange_weak(top, new_node, Ordering::Release, Ordering::Relaxed) {
            Ok(_) => return,
            Err(current) => top = current,
        }
    }
}

fn pop(domain: &Domain<Leak>, head: &AtomicPtr<Node>, root: &Root<Node>) -> Option<u64> {
    let mut guard = domain.guard();
    loop {
        let top = guard.protect(root);
        if top.is_null() {
            return None;
        }

        //SAFETY: it is not null, and it came from guard.protect, so it stays alive while guard lives
        let next = unsafe { (*top).next };
        if head
            .compare_exchange_weak(top, next, Ordering::Release, Ordering::Relaxed)
            .is_ok()
        {
            //SAFETY: it is not null, and it came from guard.protect, so it stays alive while guard lives
            let value = unsafe { (*top).value };

            // SAFETY: the exchange succeeded, so:
            // 1. `head` no longer holds `top`, and `head` is the only root, so no new
            //    reader can get it;
            // 2. only the thread whose exchange succeeded gets here, so `top` is
            //    retired once;
            // 3. `value` was read above, and `top` is not used after this line.
            unsafe {
                domain.retire(top);
            }
            return Some(value);
        }
    }
}

#[test]
fn pop_on_an_empty_stack_is_none() {
    let domain = Domain::new(Leak::new());
    let head: AtomicPtr<Node> = AtomicPtr::new(null_mut());
    // SAFETY: `head` lives to the end of this test and is not moved.
    let root = unsafe { domain.declare_root(&head) };

    assert_eq!(pop(&domain, &head, &root), None);

    domain.remove_root(root);
}

#[test]
fn pop_returns_values_in_reverse_order() {
    let domain = Domain::new(Leak::new());
    let head: AtomicPtr<Node> = AtomicPtr::new(null_mut());
    // SAFETY: `head` lives to the end of this test and is not moved.
    let root = unsafe { domain.declare_root(&head) };

    push(&head, 1);
    push(&head, 2);
    push(&head, 3);

    assert_eq!(pop(&domain, &head, &root), Some(3));
    assert_eq!(pop(&domain, &head, &root), Some(2));
    assert_eq!(pop(&domain, &head, &root), Some(1));
    assert_eq!(pop(&domain, &head, &root), None);

    domain.remove_root(root);
}

/// In a debug build every `retire` checks that no root still holds the
/// object. `pop` retires only after it moved `head` away, so the check must
/// stay quiet for the whole run. (In a release build the check is not there
/// and this test only repeats the order test.)
#[test]
fn pop_retires_without_tripping_the_check() {
    let domain = Domain::new(Leak::new());
    let head: AtomicPtr<Node> = AtomicPtr::new(null_mut());
    // SAFETY: `head` lives to the end of this test and is not moved.
    let root = unsafe { domain.declare_root(&head) };

    for value in 0..100 {
        push(&head, value);
    }
    for expected in (0..100).rev() {
        assert_eq!(pop(&domain, &head, &root), Some(expected));
    }

    domain.remove_root(root);
}

/// The other direction: a `pop` that retires the top *before* moving `head`
/// away breaks Rule 1, and the debug check must stop it.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "Rule 1 is broken")]
fn a_pop_that_retires_too_early_is_caught() {
    let domain = Domain::new(Leak::new());
    let head: AtomicPtr<Node> = AtomicPtr::new(null_mut());
    // SAFETY: `head` lives to the end of this test and is not moved.
    let root = unsafe { domain.declare_root(&head) };
    push(&head, 1);

    let mut guard = domain.guard();
    let top = guard.protect(&root);
    // SAFETY: none. `head` still holds `top`; this call breaks the first
    // promise of `retire` on purpose.
    unsafe { domain.retire(top) };
}

#[test]
fn every_value_is_popped_exactly_once() {
    // Miri runs every step in an interpreter, so it gets a smaller run.
    const THREADS: u64 = 4;
    const PER_THREAD: u64 = if cfg!(miri) { 50 } else { 10_000 };
    const TOTAL: usize = (THREADS * PER_THREAD) as usize;

    let domain = Domain::new(Leak::new());
    let head: AtomicPtr<Node> = AtomicPtr::new(null_mut());
    // SAFETY: `head` lives to the end of this test and is not moved.
    let root = unsafe { domain.declare_root(&head) };
    let popped_so_far = AtomicUsize::new(0);

    let mut seen: Vec<u64> = std::thread::scope(|scope| {
        // Each pusher pushes its own range of values, so every value in
        // `0..TOTAL` is pushed exactly once.
        for t in 0..THREADS {
            let head = &head;
            scope.spawn(move || {
                for i in 0..PER_THREAD {
                    push(head, t * PER_THREAD + i);
                }
            });
        }

        // The poppers run at the same time and stop when, together, they
        // have taken every value.
        let poppers: Vec<_> = (0..THREADS)
            .map(|_| {
                let (domain, head, root, popped_so_far) = (&domain, &head, &root, &popped_so_far);
                scope.spawn(move || {
                    let mut mine = Vec::new();
                    while popped_so_far.load(Ordering::Relaxed) < TOTAL {
                        match pop(domain, head, root) {
                            Some(value) => {
                                mine.push(value);
                                popped_so_far.fetch_add(1, Ordering::Relaxed);
                            }
                            None => std::thread::yield_now(),
                        }
                    }
                    mine
                })
            })
            .collect();

        poppers
            .into_iter()
            .flat_map(|popper| popper.join().unwrap())
            .collect()
    });

    // Nothing lost and nothing seen twice: sorted, the popped values are
    // exactly 0, 1, 2, ... TOTAL-1.
    seen.sort_unstable();
    let expected: Vec<u64> = (0..TOTAL as u64).collect();
    assert_eq!(seen, expected);
    assert_eq!(pop(&domain, &head, &root), None);

    domain.remove_root(root);
}

/// EXPERIMENT, delete after reading Miri's report. It does in small what a
/// constructor does when it declares a root on a local and then returns.
fn make(domain: &Domain<Leak>) -> (AtomicPtr<Node>, Root<Node>) {
    let head: AtomicPtr<Node> = AtomicPtr::new(null_mut());
    // SAFETY: this is the claim under test: "`head` is alive and stays at the
    // same address".
    let root = unsafe { domain.declare_root(&head) };
    (head, root)
}

#[test]
#[ignore = "experiment: run it under Miri only"]
fn a_root_declared_on_a_local_that_is_returned() {
    let domain = Domain::new(Leak::new());
    let (head, root) = make(&domain);

    println!("the place is now at   {:p}", &head);

    let mut guard = domain.guard();
    let top = guard.protect(&root);
    assert!(top.is_null());
}
