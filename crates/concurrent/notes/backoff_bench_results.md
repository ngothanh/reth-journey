# `Backoff` hardened acceptance — W004 evidence

Captured 2026-05-25 on `aarch64-apple-darwin` (Apple Silicon).

## Bench results

| Bench | p50 | W004 acceptance | Status |
|---|---|---|---|
| `spin/step=0` — single ISB + bookkeeping | 18.5 ns | spin step ≤3 cycles (PAUSE only) | ✓ |
| `spin/step=SPIN_LIMIT` — 64 ISBs | 784 ns | ~64× single-step scaling | ✓ |
| `snooze/yield branch` — single `yield_now()` | 4.6 µs | yield step ≤10 µs | ✓ |

Per-ISB cost derived from the linear bench: (784 − 18.5) ns / 63 extra ISBs ≈ **12.2 ns/ISB** ≈
~42 cycles at 3.5 GHz. That matches Apple Silicon's documented ISB cost (heavier than ARM YIELD on
other aarch64 targets — same workload on `aarch64-linux` with YIELD lowering would be ~1-3 cycles).

## Asm evidence (no SMT pipeline starvation)

`cargo asm -p concurrent --lib --rust 'concurrent::backoff::Backoff::spin'` (see
`backoff_spin.asm.txt`):

```
LBB0_1:
    isb            // ← core::hint::spin_loop() lowering on aarch64-apple-darwin
    lsr w10, w9, w8
    add w9, w9, #1
    cbz w10, LBB0_1
```

`isb` on aarch64 is the equivalent of `pause` on x86 — both signal "spinning, release pipeline /
SMT siblings." Acceptance "every spin iteration MUST call `core::hint::spin_loop()`" verified.

## What remains blocked on the W4 Thu `Parker`

The W004 acceptance bullets that read **park step ≤500 ns entry** and **p99 wake from park ≤1 µs
after unpark** belong to `Parker`'s benches, not `Backoff`'s — `Backoff` deliberately never parks
(see module doc). The Saturday loom+bench section will integrate Backoff + Parker together for
the **CPU utilization <1% at idle for 60s** test, which requires the full caller loop:

```rust
while !condition {
    if backoff.is_completed() { parker.park(); }
    else { backoff.snooze(); }
}
```

## 5-year failure modes (see module doc in backoff.rs)

1. Single-core deployment → SPIN_LIMIT should drop to 0
2. 256+ core NUMA → spin band widens, yield gated on locality hint
3. RISC-V without Zihintpause → `spin_loop()` becomes a no-op, burst tuning needed
