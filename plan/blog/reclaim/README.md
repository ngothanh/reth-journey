# Building `reclaim`: safe memory reclamation in Rust

A design investigation into freeing memory that another thread may still be reading.
Every lock-free linked structure has this problem when there is no garbage collector.
The series builds a crate, `reclaim`, from an empty `lib.rs`. It starts with the model:
two rules, three roles, roots and guards. Then it builds the contract between object,
structure and scheme. It ends with the crate's first concurrent component: a pile that
holds retired objects of any type and never allocates. The pile's cost per push falls
from ≈ 125 ns (one shared head, 8 threads) to the level of a control in which no thread
shares anything (≈ 0.8 ns).

Every decision is forced by a crash, Miri, loom or a benchmark. The wrong first attempts
are part of the story. More parts follow as the crate grows.

## Languages

- **English** — [`en/00_index.md`](en/00_index.md)

## Parts

| # | English |
|---|---|
| 1 | [The problem: freeing memory another thread may be reading](en/01_the_problem.md) |
| 2 | [Roots: where every reader starts](en/02_roots.md) |
| 3 | [Two ways to say "I'm reading"](en/03_guards.md) |
| 4 | [What an unsafe trait promises](en/04_the_contract.md) |
| 5 | [Where a root lives, and what only Miri saw](en/05_where_a_root_lives.md) |
| 6 | [The pile of retired objects](en/06_the_pile.md) |
| 7 | [One `Release`, one `Acquire`, everything else `Relaxed`](en/07_orderings.md) |
| 8 | [Measuring the pile: one head, eight heads, and chains](en/08_measuring.md) |
