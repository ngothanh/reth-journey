# Drill #3 — numbers

**Machine**: Apple M-series, `aarch64-apple-darwin`, 1 socket. Release build.
**Command**: `cargo bench -p concurrent --bench dekker`
**Disassembly**: [`06_drill3_dekker.asm.txt`](06_drill3_dekker.asm.txt)

> The plan's table is x86-centric (`MFENCE`, "Release-Acquire ~0 ns"). Half of it is not
> measurable on this machine. Numbers below are aarch64 and should not be compared to it
> directly.

## Prediction (written before running)

Reading the disassembly, `DekkerFence` emits an explicit `dmb ish` while `DekkerSeqCst`
emits none — ordering comes from `ldar`'s RCsc semantics enforced by the memory system.
So: **SeqCst store/load should be cheaper than the explicit fence on aarch64**, the
opposite of the plan's x86 expectation.

**This prediction was wrong.** Measured, SeqCst is consistently the *most* expensive of
the three under contention. See below.

## Uncontended — isolates barrier cost

One thread, `enter`/`exit` with the other flag always false. No retry loop, no
cross-core traffic. This is the cost of the instructions themselves.

| Variant | ns / enter+exit | Instructions |
|---|---|---|
| `acq_rel` | **1.782** [1.7817, 1.7862] | `stlrb` + `ldaprb` |
| `fence_seqcst` | **1.786** [1.7830, 1.7890] | `stlrb` + `dmb ish` + `ldaprb` |
| `seqcst_store_load` | **1.799** [1.7920, 1.8091] | `stlrb` + `ldarb` |

**All three are the same, to within 1%.** Adding a `dmb ish` costs ~4 picoseconds.

This is the headline finding and it contradicts the plan's "`DMB ish` ~20–30 ns"
estimate by four orders of magnitude. The reason: **a barrier's cost is not the
instruction, it is the coherence traffic it forces you to wait for.** Uncontended, the
cache line is exclusive in L1 and there are no outstanding memory operations, so
`dmb ish` has nothing to wait for and retires almost immediately.

Corollary worth keeping: *you cannot price a memory barrier without contention.* A
microbenchmark of a fence on one thread measures nothing.

## Contended — 2 threads, both entering

| Variant | ns / enter+exit | 95% CI | Correct? |
|---|---|---|---|
| `acq_rel` | 29.4 | [22.0, 35.7] | **NO** — not comparable |
| `fence_seqcst` | 102.7 | [89.4, 110.4] | yes |
| `seqcst_store_load` | 117.3 | [113.7, 120.4] | yes |

Two caveats that matter more than the numbers:

**`acq_rel` is not a valid baseline.** It doesn't provide mutual exclusion, so it lets
threads through that should have retried. It is fast *because* it is wrong — fewer
retries, less ping-pong. Reading it as "the cost we're paying for correctness" is
exactly backwards.

**This benchmark does not measure barrier cost.** It measures protocol throughput under
contention: barrier cost *plus* cache-line ping-pong *plus* however many times the retry
loop spun. Those are entangled and the retry dynamics dominate — see the run-to-run
variance below.

## Run-to-run variance

The contended numbers are not stable across invocations:

| Variant | run 1 median | run 2 median |
|---|---|---|
| `acq_rel` | 23.1 | 29.4 |
| `fence_seqcst` | 56.8 | 102.7 |
| `seqcst_store_load` | 75.1 | 117.3 |

Run 1's confidence intervals overlapped almost completely (fence [37.5, 77.4] vs seqcst
[49.0, 102.8]) — that run could not distinguish them at all. Run 2 separates them, but a
~1.8× shift in the same benchmark between runs means the *ordering* is the only claim
worth making, not the magnitudes. Both runs put `seqcst_store_load` above
`fence_seqcst`, so the direction is probably real; the ratio is not.

Likely cause: two threads with no affinity pinning on a heterogeneous P/E-core chip. A
schedule that lands both on performance cores behaves very differently from one that
splits them. Fixing this needs core pinning, which is out of scope here — but note it
before quoting any of these figures elsewhere.

## Against the plan's "within 30% or explain"

Explained, on three counts:

1. **The plan's x86 numbers don't apply.** No `MFENCE` here; the instruction is
   `dmb ish`, and Release/Acquire is *not* free on aarch64 — it is `stlrb`/`ldaprb`,
   real instructions rather than plain `mov`s.
2. **The ~20–30 ns fence estimate is uncontended-wrong and contended-low.** Uncontended
   it is ~0.004 ns of marginal cost; contended it is buried inside ~100 ns of coherence
   traffic. Neither is 20–30 ns, because the figure describes neither situation.
3. **My own prediction was wrong.** I expected `ldar` (no explicit barrier) to beat
   `dmb ish` + `ldapr`. It loses, consistently, by ~15%. Plausible reading: `ldar`'s
   RCsc requirement orders it against *all* prior stores, whereas `dmb ish` + `ldapr`
   lets the implementation be more surgical. Not verified — flagged as a guess.

## The transferable result

The one worth carrying to W74 (VSR quorum-ack) and W83 (2PC vote):

> **An SC barrier is free until it is contended.** Its cost is the cross-core traffic it
> serialises, not the instruction. Which means the fix for an expensive barrier is
> almost never a cheaper barrier — it is removing the sharing, so there is no traffic to
> serialise. Per-replica ack slots indexed by replica-id have no shared flag and
> therefore need no SC at all.
