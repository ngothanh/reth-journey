# W005 Mon/Tue — `Bytes` redesign pre-predictions

> **Provenance**: reconstructed from the resulting code, not sealed beforehand. The
> "actual" column is verifiable from `bytes.rs`; the "predicted" column is not. Recorded
> this way rather than presented as a genuine prediction record.

- **P1 — where does the representation tag live?** Predicted: the buffer pointer's low bit
  (the W4 scheme). Actual: the `ctx` word, with `cap` packed alongside it as
  `(cap << 1) | 1`. Moving it off the pointer is what made slicing safe.
- **P2 — does `freeze` allocate?** Predicted: yes, one control block. Actual: no — the
  control block is created lazily on first clone. This was the whole point of the redesign.
- **P3 — how many vtables?** Predicted: two (static, owned). Actual: three — `OWNED` and
  `SHARE` must differ because owned's `clone` has to *promote* first.
- **P4 — memory ordering on the refcount increment.** Predicted: `Acquire`. Actual:
  `Relaxed` is sufficient; the increment publishes only the existence of an owner, and the
  contents were already published. The `Acquire`/`Release` pair is needed on the decrement.
- **P5 — tests predicted to fail first.** Predicted: the no-alloc bench assertion.
