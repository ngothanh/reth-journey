# `reclaim` — the teaching plan for C1

> **Method**: the `model-first-tutor` skill. Map first, then model, then core types, then
> behaviour. Every lesson has an Input and an Output; every Output is an artifact.
> **You type everything** — code, tests, docs. I present, ask, and review.
> **Covers**: the model of safe memory reclamation (≈ 8 h) and all of C1 (7 h) — twelve
> lessons, 15 h. C2 onward stays in `plan/reclaim_ladder.md`.

| Stage | Lessons | What exists at the end | Est |
|---|---|---|---:|
| **0 Map** | L0 | your own retelling of the whole picture | 1 h |
| **1 Model** | L1–L5 | `notes/reclaim_model.md` complete, ending in every signature | 7 h |
| **2 Core** | L6–L8 | every type in the crate exists, constructs, and has its invariant tests | 3 h |
| **3 Behaviour** | L9–L11 | roots work, `Leak` runs, a Treiber `pop` uses the trait — **C1 done** | 4 h |

---

# The map

Read this before L0. It says **what exists and where it sits**. It does not say why — the
lessons are for that.

## Three layers

```
┌─ STRUCTURE ─ the queue or stack ──────────────────────────────────────┐
│   holds:  Root (one per entry point)   Domain (the handle)            │
│   calls:  guard()   try_protect()   retire()                          │
│                                                                       │
│   ┌─ OBJECT ─ the thing that gets freed (a Segment, a Node) ────┐     │
│   │   implements: Retire          contains: RetireLink          │     │
│   └─────────────────────────────────────────────────────────────┘     │
└───────────────────────────────┬───────────────────────────────────────┘
                                │ calls into
┌─ SCHEME ─ Leak, later Hazard and Epoch ───────────────────────────────┐
│   implements: Reclaim       hands out: a Guard                        │
│   owns: the retired list, and the decision of WHEN to free            │
│   Domain = a cheap handle around one scheme + its RootRegistry        │
└───────────────────────────────────────────────────────────────────────┘
```

## Components

| Name | Kind | Layer | One sentence |
|---|---|---|---|
| `Retire` | unsafe trait | object | "I can be handed to a scheme: here is my link, here is how to free me." |
| `RetireLink` | struct | object | A `next` pointer stored *inside* the object, so retired objects can be chained without allocating. |
| `Root<T>` | struct | structure | The address of one atomic that readers start from. One word. |
| `RootRegistry` | struct | scheme | The list of declared roots of one domain. Only does work in debug builds. |
| `Reclaim` | unsafe trait | scheme | What every scheme offers: give me a guard, take this retired object. |
| `Guard` | trait | scheme | What a reader holds while reading; it can be pointed at a new pointer. |
| `Domain<R>` | struct | scheme | A cloneable handle to one scheme `R` plus its registry. The structure stores one. |
| `Leak` | struct | scheme | The simplest scheme: accepts retired objects and never frees them. |

## The life of one segment

Segment `A` in your `SegQueue`, start to finish. Each step names who acts.

1. **Structure** is created with a `Domain`. It declares two roots: `head` and `tail`.
2. **Structure** allocates `A`. `A` contains a `RetireLink`, unused for now.
3. **Reader** asks the domain for a `Guard`.
4. **Reader** calls `try_protect(p, &head_root)` on the guard. While the guard points at `A`,
   `A` may be dereferenced.
5. **Structure** advances `head` past `A`. No new reader can reach `A` from a root any more.
6. **Structure** calls `retire(A)`. Ownership of `A` passes to the **scheme**, which chains it
   onto its retired list through `A`'s `RetireLink`.
7. **Scheme** (debug only) walks the `RootRegistry` and checks `A` really is unreachable.
8. **Readers** that were already holding `A` finish; their guards move on or drop.
9. **Scheme** decides that nobody can still be holding `A`. (`Leak` never decides this.)
10. **Scheme** calls `A::reclaim(ptr)`. `A` frees itself, its own way.

## Glossary

| Term | Meaning |
|---|---|
| retire | "I am done with this object." Said by the structure. Does not free. |
| reclaim | Actually free it. Done by the scheme, later. |
| grace period | The wait between retire and reclaim, until nobody can still be reading. |
| root | An atomic a reader loads a pointer from and protects through. |
| protect | Make one pointer safe to dereference, by telling the scheme you are reading it. |
| guard | The thing a reader holds that does the protecting. |
| scheme | The policy that knows when the grace period is over. |
| domain | One independent instance of a scheme with its own lists. |
| exhibit | A numbered multi-thread trace that ends in a named failure. |
| Treiber stack | The simplest lock-free stack: one atomic `head` pointing at a chain of nodes; push and pop are a CAS on `head`. First met in L2, drawn there. |
| Harris list | A lock-free sorted linked list. Deleting a node first *marks* it, and unlinks it later. First met in L2, drawn there. |

---

# Stage 0 — Map

### L0 — The whole picture
- **Where on the map**: all of it.
- **Input**: the map above; your own `seg_queue.rs`; the memory of the refcount bug.
- **Teacher presents**: the whole map, drawn and walked **in the session, by the teacher**, on
  your own `SegQueue` — not handed over as reading.
- **Questions**: (1) For each of the eight components, which layer is it in and who creates
  it? (2) In the ten steps, which ones run on the reader's thread and which on a different
  one? (3) Which two steps are separated by the grace period? (4) Which component in the
  table does `Leak` make pointless, and which does it still need?
- **Output**: `notes/reclaim_model.md` §1–§3, in your own words — the eight components in one
  sentence each, the layer diagram redrawn, and the ten steps retold for a segment of your own
  queue with the real field and function names from `seg_queue.rs` next to each step.
- **Gate**: retell the life of one object, unaided, naming who acts at each step.
- **Est**: 1 h

---

# Stage 1 — Model

Every output here is a section of `notes/reclaim_model.md`. No compiled code.

### L1 — The race, and the two halves of the proof
- **Where on the map**: steps 4–9; the boundary between structure and scheme.
- **Input**: L0's §3; the refcount proof you already did.
- **Teacher presents**: only this — a reader and a retirer run at the same time, and "free"
  is a single instruction.
- **Questions**: (1) What does a reader hold between loading a pointer and dereferencing it?
  (2) Write the sentence "dereferencing `p` after `retire(p)` is sound because ___" and try to
  finish it — where does it fail first? (3) Where does it fail the second time? (4) One of the
  two gaps can never be filled by a scheme. Which, and why is that impossible rather than hard?
- **Exhibit**: two threads on your queue, one popping and one freeing, ending in a
  use-after-free.
- **Output**: §4 — the exhibit; the two requirements stated in your words, each with its owner
  (structure or scheme); the impossibility argument in three sentences.
- **Gate**: state both halves and who owes each, without notes.
- **Est**: 1.5 h

### L2 — Roots
- **Where on the map**: `Root`, `RootRegistry`; steps 1, 4, 5, 7.
- **Input**: §4; your `seg_queue.rs` as it is today, where `protect` goes through `cur.next`.
- **Teacher presents**: a thread cannot invent a pointer — it loads one from an atomic. The
  atomics it loads from *and protects through* are the roots. Plus a drawing of a Treiber stack
  and of a Harris list, since neither has been met before.
- **Questions**: (1) `cur.next` is a field inside segment `cur`. If it is a root, what must be
  true of `cur` forever? (2) After the fix, `pop` still *reads* `cur.next` — why is reading it
  not the same as protecting through it? (3) What is the root set of a Treiber stack? (4) A
  Harris list protects through `pred.next`, a different atomic for every node — what is its
  root set, and what follows?
- **Exhibit**: your queue today — the trace showing segment `A` can never be freed. Then the
  same queue protecting through `head` and `tail` — why `A` now can.
- **Output**: §5 — both exhibits; a table of root sets for SegQueue, Treiber stack and Harris
  list with a yes/no on "can satisfy the structure's half unaided"; the one-line rule about a
  root that lives inside an object.
- **Gate**: given a structure, name its root set and say whether it can ever free anything.
- **Est**: 1.5 h

### L3 — The guard, and how a scheme knows the wait is over
- **Where on the map**: `Guard`, `Reclaim`; steps 3, 4, 8, 9.
- **Input**: §4, §5.
- **Teacher presents**: two ways a reader can tell the scheme it is reading — write down the
  *address* it is reading, or write down *when* it started. That is hazard pointers and epoch.
- **Questions**: (1) For each of the two, what does the scheme check at step 9? (2) How many
  pointers does one guard cover in each? (3) Announcing an address is "load the pointer, then
  store it in my slot" — what can happen between those two instructions? (4) In a retry loop
  that protects a fresh pointer each time round, would you rather get a new guard each
  iteration or re-point the one you hold — what goes wrong with a new one?
- **Exhibit**: the reader-side race for address announcing — the announcement landing one
  instruction too late.
- **Output**: §6 — the table (rows hazard / epoch / leak; columns: a guard is · protects ·
  cost to get one · cost per protect · how step 9 is decided); the exhibit; one paragraph on
  why a guard is re-pointed rather than re-made.
- **Gate**: explain why the two schemes mean different things by "guard", and what a trait
  shared by both is therefore forced to look like.
- **Est**: 2 h
- **Left open on purpose**: why epoch needs three generations. C6 builds two and lets loom
  answer.

### L4 — Who owns the garbage
- **Where on the map**: `RetireLink`, `Retire`, the scheme's retired list; steps 6, 9, 10.
- **Input**: §6; `crates/reclaim/src/retire.rs` (already written — read it now, not before).
- **Teacher presents**: retired objects of *different types* sit on one list; the list is made
  from links stored inside the objects themselves.
- **Questions**: (1) If the list is one chain of mixed types, what type can `next` have?
  (2) Step 10 runs on which thread, compared with step 6 — and what bound on the object does
  that force? (3) If freeing only ever happens inside `pop`, what does a queue nobody pops do?
  (4) A threshold of "1000 retired objects": name a workload that stays under it forever.
- **Output**: §7 — the life of a *retired* object from step 6 to step 10 as its own numbered
  list; the four properties `retire_link` must have, each with a one-line reason; the two
  triggers and what each one bounds.
- **Gate**: explain why a count threshold alone does not bound garbage.
- **Est**: 1 h

### L5 — The model, written as signatures
- **Where on the map**: all eight components.
- **Input**: §1–§7, and nothing else. Do not open `plan/reclaim_ladder.md`.
- **Teacher presents**: nothing new.
- **Questions**: (1) For each component: trait or struct, and what are its methods with their
  argument and return types? (2) Which methods are `unsafe fn`, and for each, what does the
  caller promise? (3) Which traits are `unsafe trait`, and what does the implementor promise?
- **Output**: §8 — one code block holding every signature in the crate, no bodies. Then §9 —
  a diff against the nine decisions recorded in `plan/reclaim_ladder.md` step 1: where you
  differ, which is right and why.
- **Gate**: defend every `unsafe` in §8.
- **Est**: 1 h — **modeling is finished here.**

---

# Stage 2 — Core

Types, fields, constructors, and the tests of what each type guarantees. Innermost first.

### L6 — The object side
- **Where on the map**: `RetireLink`, `Retire`.
- **Input**: §7, §8; `retire.rs` as it stands — the trait and docs exist, the test module is a
  `TODO(you)`.
- **Questions**: (1) Of the seven properties in the trait's safety docs, which can a
  single-threaded test catch? (2) Which two cannot, and what tool will? (3) Why is
  `RetireLink::new` not `const fn`?
- **Output**: `retire.rs` finished — a test type implementing `Retire` whose `reclaim` is not
  a plain `Box` free, and one test per testable property, each written from its exhibit.
- **Gate**: `cargo test -p reclaim` green; a comment naming the two untestable properties.
- **Est**: 1 h

### L7 — The root types
- **Where on the map**: `Root<T>`, `RootRegistry`.
- **Input**: §5, §8; `root.rs`.
- **Questions**: (1) Why a raw pointer and not a lifetime — what would `SegQueue` look like
  with the lifetime? (2) Why must `Root` stay one word and `Copy`? (3) The registry is not
  generic over `T` — so what does it store?
- **Output**: `root.rs` data complete — `Root<T>` with `Send`/`Sync`/`Copy` and its accessors;
  `RootRegistry` as a struct with `new()`; tests for "names the atomic it was given" and
  "is exactly one word".
- **Gate**: tests green; explain what breaks the size test.
- **Est**: 0.5 h
- **Open before this lesson**: `root.rs` currently holds a version I wrote. Either it is
  reverted to your four lines and you type it here, or it is kept as given and this lesson
  adds only `RootRegistry` and the tests. Your call.

### L8 — The scheme-side types
- **Where on the map**: `Reclaim`, `Guard`, `Domain<R>`, `Leak`.
- **Input**: §6, §8.
- **Questions**: (1) `Guard` is different per scheme — how does a trait say "each implementor
  brings its own guard type"? (2) What does `Domain<R>` contain, given it must be cheap to
  clone and must own a registry? (3) What fields does `Leak`'s guard need?
- **Output**: `lib.rs` — `Reclaim` and `Guard` declared with every method signature from §8;
  `domain.rs` — the `Domain<R>` struct and `new`; `leak.rs` — `Leak` and its guard as structs.
  It compiles; trait methods on `Leak` may be `todo!()`.
- **Gate**: `cargo build -p reclaim` and the `--cfg loom` build both clean.
- **Est**: 1.5 h

---

# Stage 3 — Behaviour

### L9 — Roots that work
- **Where on the map**: `RootRegistry`, `Domain::declare_root`; steps 1 and 7.
- **Input**: L7 and L8 outputs.
- **Questions**: (1) The registry only matters in debug — how simple can its storage be?
  (2) How does it vanish in release without the signature changing between builds? (3) What
  does `declare_root` promise that `assume_root` does not, and what does the caller owe each?
- **Output**: `RootRegistry::declare` / `remove` / a way to visit every root; `Domain`'s
  `declare_root` and `remove_root` forwarding to it; tests — a declared root is visited, a
  removed one is not, two domains do not see each other's roots.
- **Gate**: tests green in debug and in release.
- **Est**: 1 h

### L10 — `Leak`, the first scheme that runs
- **Where on the map**: `Leak`, its guard; steps 3, 4, 6.
- **Input**: L8, L9; §6.
- **Questions**: (1) What is the least `try_protect` can do for a scheme that never frees?
  (2) `protect` loops over `try_protect` — write it once, in the trait, so no scheme has to.
  Why does it not loop for `Leak`? (3) What are a guard's three states, and what does
  `as_ref` return in each? (4) What must `swap` leave unchanged?
- **Output**: `Leak` implementing `Reclaim` in full; its guard implementing `Guard` —
  `try_protect`, the provided `protect`, `as_ref`, `swap`; `retire` chaining objects through
  their links; tests — retired objects are never reclaimed, the chain grows, `swap` keeps both
  protections.
- **Gate**: tests green; Miri clean with `-Zmiri-ignore-leaks`.
- **Est**: 2 h

### L11 — The contract, and a real client
- **Where on the map**: all of it, used together.
- **Input**: everything above; §4–§8.
- **Questions**: (1) For each `unsafe fn`: the clauses, and the exhibit behind each. (2) Which
  clauses does a tool enforce, and which only review? (3) Write a Treiber `pop` against the
  trait — which of the ten steps does each line perform?
- **Output**: the `# Safety` blocks for `retire`, `declare_root`, `assume_root`; a doc-test on
  `Reclaim` that is a working Treiber `pop` on `Leak`; the journal entries for Stage 2–3.
- **Gate**: `cargo test -p reclaim --doc` green. **C1 is done**; next is edit 2 in
  `plan/reclaim_ladder.md`.
- **Est**: 1 h
