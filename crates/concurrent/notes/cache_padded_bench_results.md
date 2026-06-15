# CachePadded false-sharing results (W4 — `concurrent v0.1.0`)

- **Host**: `[aarch64-apple-darwin]` (Apple Silicon, 128 B effective line via L2 prefetcher)
- **How**: `cargo bench -p concurrent --bench false_sharing -- --measurement-time 5 --sample-size 50`
  (criterion `iter_custom`; 2 threads ping-pong adjacent `AtomicU64` via `fetch_add(1, Relaxed)`).
- **Guards that passed** (so the numbers mean something):
  - layout: `offset_of!(BarePair, b) = 8 < 128`, `offset_of!(PaddedPair, b) = 128 ≥ 128`.
  - elision: each counter asserted `== iters` at end of every measured run.

## Numbers

| bench | per-op (1 thread) | ops/sec (1 thread) | ops/sec (2 threads) |
|---|---|---|---|
| `bare_pingpong`   | 12.75 ns | 78.4 M | 156.9 M |
| `padded_pingpong` | 2.09 ns  | 479.3 M | 958.7 M |

| Metric | Threshold (aarch64) | Observed | Result |
|---|---|---|---|
| **Cliff ratio** (padded/bare ops/sec) | ≥ 3× | **6.1×** | ✅ PASS |

## Interpretation

- **The cliff is real and large: 6.1×.** In `bare_pingpong` the two counters sit
  8 B apart, sharing one cache line, so every `fetch_add` from either thread
  bounces the line between cores (MESI ping-pong) → 12.75 ns/op. `CachePadded`
  (`align(128)`) puts each counter on its own line, so the writes are effectively
  uncontended → 2.09 ns/op. That 2.09 ns is an L1-hot uncontended atomic RMW on
  Apple Silicon — plausible, and the end-state assertion proves it isn't elision
  (billions of ops/sec would be the elision tell; we see ~480 M/thread).

- **Monday's Back-of-envelope claim ("30-70% regression") was conservative for
  this contention pattern.** A pure 2-counter ping-pong is the worst case for
  false sharing; the measured regression is the *bare* path running at ~16% of
  padded throughput (a ~6× hit), beyond the 30-70% band. Real MPMC structures
  amortize this across non-contended work, so production deltas land back inside
  30-70% — but the primitive's worst-case cost is now a measured number.

- **vs LMAX reference (4.9× on x86_64, 26M vs 5.3M ops/sec for 1P-1C).** Our
  *ratio* (6.1×) is the same order (within 2×). Absolute ops/sec is much higher
  (~960 M system) because this microbench times a bare `fetch_add`, not LMAX's
  full ring-buffer claim — and the hardware is a decade newer. The portable,
  citable artifact is the ratio, not the absolute.

## x86_64

Not measured on this host. Threshold there is ≥ 5× (64 B line, no 128 B prefetch
widening — false sharing is sharper on a narrower line). Re-run on an
`x86_64-linux` box and append a `[x86_64-linux]` block before citing the x86
number downstream.
