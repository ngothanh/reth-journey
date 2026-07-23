# W005 Wed — `Header` pre-predictions

> **Provenance**: reconstructed from the resulting code, not sealed beforehand.

- **P1 — struct shape.** Predicted: `Header { list: bool, payload_length: usize }`. Actual:
  exactly that.
- **P2 — does `encode` always emit a header byte?** Predicted: yes. Actual: **no** — the
  single-byte-below-`0x80` case is its own complete encoding with no prefix, and that check
  must come *before* the short/long-form branch. This was the R2 headline and the source of
  the first failing test.
- **P3 — where does the long form start?** Predicted: payload > 55. Actual: correct;
  `0xb7`/`0xf7` plus the length-of-length.
- **P4 — canonicity checks on decode.** Predicted: none on the first attempt. Actual: two
  were needed — a leading zero in the length bytes (`NonCanonical`) and a long-form header
  declaring a payload < 56 (`NonCanonical`), which is the same value spelled two ways.
- **P5 — tests predicted to fail.** Predicted: the ethereumjs fixture round-trip.
