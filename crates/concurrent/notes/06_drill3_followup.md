# Drill #3 — 5-year failure mode: SC cost under cross-NUMA / many-socket

**Trigger**: deployment moves off a single socket. A 2-socket server, a cloud instance
type whose vCPUs span NUMA nodes, or any scheduler free to place the two participating
threads on different sockets.

## What breaks

Nothing correctness-wise. `fence(SeqCst)` is exactly as correct at 2 sockets as at 1 —
the memory model does not weaken. What changes is the price, and this drill measured
precisely why.

From [`06_drill3_numbers.md`](06_drill3_numbers.md), on 1 socket:

| | uncontended | contended |
|---|---|---|
| `DekkerAcqRel` (no barrier) | 1.782 ns | 29.4 ns |
| `DekkerFence` (`dmb ish`) | 1.786 ns | 102.7 ns |

**The barrier instruction costs ~4 picoseconds. The barrier *situation* costs ~100 ns.**
Over 98% of the price is the cross-core coherence traffic the fence forces you to wait
for, not the fence itself.

That decomposition is what makes the NUMA trigger predictable. A second socket does not
make `dmb ish` slower — it makes *the thing the fence waits for* slower. A cache line
bouncing between two cores on one die is settled in the shared L2/SLC; between two
sockets it crosses the interconnect, at roughly 2–4× the latency. The plan's estimate of
**~3× total cost** is consistent with that, though it is an estimate, not something
measured here.

The nasty property is that the regression is **invisible on the dev machine**. The code
does not change, the tests still pass, the single-socket benchmark is unmoved. It shows
up only as a throughput cliff in production, on a path nobody edited.

## Migration

The fix is not a cheaper barrier. There isn't one — StoreLoad ordering is the only
reordering that requires waiting for global visibility, so any construct that provides it
pays the same traffic. **The fix is to remove the sharing, so there is no traffic to
serialise.**

In descending order of preference:

1. **Single-writer ownership (drill #14's sharding).** Give each thread its own cache line
   that only it writes. Readers may read stale values; the protocol is designed so stale
   is safe. No cross-core RMW, no SC, and — critically — the cost does not depend on
   socket topology at all, because the line stays resident in the writer's cache.
2. **Per-shard locks.** If shared mutable state is unavoidable, shrink the sharing set so
   contention is intra-shard and shards can be pinned per NUMA node. Turns one global
   barrier into N uncontended ones.
3. **Keep SC, pin the participants.** If neither restructuring is possible, pin the
   threads that share a line to the same NUMA node so the traffic stays local. Weakest
   option — it constrains the scheduler forever and silently degrades the day someone
   changes an affinity mask.

For the concrete downstream cases: **W74 VSR quorum-ack** takes option 1 — per-replica ack
slots indexed by replica-id, each written by exactly one replica, so the "did N replicas
ack?" check becomes N independent single-writer reads with no shared flag and no SC.
**W83 ledger 2PC vote** is the same shape and takes the same escape.

## Detecting it before it bites

- `numactl --hardware` / `lscpu` on the target to confirm socket and node count — do this
  when the instance type is chosen, not when the alert fires.
- Re-run the contended bench with the two threads pinned to *different* NUMA nodes, and
  again pinned to the same node. The ratio is the tax.
- `perf stat -e node-load-misses,offcore_response.*.remote_dram.*` on the hot path.

**This cannot be reproduced on the current machine.** Apple silicon is single-socket with
no NUMA, so `06_drill3_numbers.md` establishes the 1-socket baseline and nothing more. The
earliest honest test is running the bench on a 2-socket x86 host in CI.

## Second-order

Once the interconnect is in play, **false sharing gets much more expensive too** —
unrelated data that happens to share a line now drags the interconnect into a conflict
that isn't even logically contended. `CachePadded` stops being a micro-optimisation and
becomes load-bearing. Audit every struct with a hot atomic next to any other field at the
same time as the SC audit; they are the same walk of the same code.

Also: the plan's cost figures for this drill are x86 (`MFENCE`). If the deployment target
is 2-socket ARM (Graviton, Ampere), the instruction is `dmb ish` and the numbers need
re-deriving from scratch. Do not port the estimates across architectures — this drill
already showed that a plan table written for x86 was wrong by four orders of magnitude on
aarch64.
