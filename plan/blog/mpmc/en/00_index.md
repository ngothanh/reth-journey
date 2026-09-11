# Designing a lock-free MPMC ring

At the write end of anything fast — a matching engine taking orders off a dozen feed
threads, a write-ahead log fanning writers into one commit — there's a queue that many
threads push and many threads pop, millions of times a second. Reach for a
`Mutex<VecDeque>` and every one of those threads serializes on one lock word that
bounces between cores. This series designs the queue that doesn't: a bounded,
allocation-free, lock-free ring shared by N producers and M consumers, where the only
cost is the hand-off itself. Four parts, ten to fifteen minutes each.

It's a design investigation, not a tutorial. Every decision is forced — by a use
case, by the simpler version failing, or by a verification tool. No lock-free
background is assumed; `compare_exchange`, `Acquire`/`Release`, `UnsafeCell`,
`MaybeUninit`, loom, Miri and `CachePadded` are introduced where the design collides
with them.

## The parts

**[Part 1 — The problem, and why the obvious queues don't fit.](01_the_problem.md)**
Three constraints — bounded and allocation-free, lock-free, many-to-many and
non-blocking — and the tour of queues that each break one. The lock turns out to be
doing two jobs, allocating turns and handing off data, and separating them is the
whole design: an array, two ticket counters, and a mask.

**[Part 2 — One counter can't say "written".](02_the_sequence.md)** The obvious ring
tears the instant a consumer runs beside a producer, because the tail moves at *claim*
time, before the write. A per-slot boolean can't fix it — a boolean has no memory, and
slots are reused across laps. Vyukov's per-slot sequence can: a number that says which
ticket the slot is ready for. A three-way gate, the publish and vacate rules, and the
contract for when `try_push` retries and when it returns.

**[Part 3 — Getting the memory ordering right.](03_memory_ordering.md)** All-`Relaxed`,
the ring is a data race. The orderings are derived, not guessed: find the two places
data crosses threads, then notice the payload can only be published through `seq`,
never through the counter — a `Release` floors what comes *before* it, and the claim
comes before the write. Floor and roof, one gate at a time, why the CAS stays
`Relaxed`, and why this ring — unlike a SeqLock — needs no fence. The heart of the
series.

**[Part 4 — Proving it: two tools, and a lying green.](04_proving_it.md)** Miri names
the race; loom, given the `Relaxed` ring, explodes instead of answering. With the
orderings in, both go green — and loom's green is worth less than it looks: it models
a compare-and-swap more strongly than the hardware, and passes a broken hand-off that
runs through one, shown by changing one line. The division of labor between the
three instruments, a `Drop` that Miri's leak checker approves, and the benchmark:
padding `head` and `tail` onto separate cache lines takes the SPSC hand-off from
~51 ns to ~9 ns.

## How to read it

In order. Each part opens where the previous one stopped and closes on the question
the next one answers. Stopping after Part 2 leaves you with a ring that's logically
correct; Parts 3 and 4 are where it meets the hardware and the verifiers, and where
the failures that pass tests live.

## Scope

A generic bounded `MpmcRing<T>` — the Vyukov algorithm, as you'd put it in a
concurrency crate. Built and measured on `aarch64` (Apple M2), because a weak memory
model is where the Part 3 failures show; on x86 the data race would hide behind a
stronger hardware guarantee. The numbers in Part 4 are from the crate's `criterion`
benches, and every claim about correctness is one loom or Miri actually checked.

## Glossary

- **MPMC ring** — a bounded circular buffer that multiple producers push and multiple
  consumers pop, lock-free and allocation-free.
- **lock-free** — some thread always makes progress; no thread holds a lock others
  wait on. Not wait-free: an individual thread may retry.
- **ticket** — the value a thread takes from `tail` (producer) or `head` (consumer);
  `ticket & (capacity − 1)` is its slot.
- **slot / cell** — one element of the buffer: a payload and its sequence number.
- **`seq`** — the per-slot counter that says which ticket the slot is ready for. It
  only increases: `pos` (producer may write), `pos + 1` (consumer may read),
  `pos + capacity` (next lap's producer may write).
- **claim vs publish** — taking a ticket on the counter vs making the payload visible
  through `seq`. Different operations, different atomics.
- **hand-off** — one thread finishing with some bytes and signaling, another picking
  them up: a sending side and a receiving side. Two per slot: A (publish, producer →
  consumer) and B (vacate, consumer → next lap's producer).
- **`compare_exchange` (CAS)** — atomic read-modify-write that succeeds only if the
  value is what you expected; how exactly one thread claims a ticket.
- **`Relaxed` / `Acquire` / `Release`** — `Relaxed` is atomic with no ordering;
  `Release` on a store floors what comes before it; `Acquire` on a load roofs what comes
  after it; an `Acquire` load that observes a `Release` store joins the two.
- **happens-before** — the cross-thread guarantee a `Release`/`Acquire` pair builds;
  without one, a plain write on one thread and a plain read on another are a data race.
- **false sharing** — two independent values on one cache line, each write invalidating
  the other core's copy; fixed by `CachePadded`.
- **loom** — a model checker that runs a small test under every interleaving; blind to
  hand-offs carried by a CAS.
- **Miri** — an interpreter that runs Rust against the memory model and reports UB —
  data races, uninitialized reads, leaks.

*Deutsch: [`../de/00_index.md`](../de/00_index.md)*
