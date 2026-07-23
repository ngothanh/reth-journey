# W005 Thu — lists pre-predictions

> **Provenance**: reconstructed from the resulting code, not sealed beforehand.

- **P1 — can `Vec<u8>` and `Vec<T>` both have impls?** Predicted: no, coherence conflict.
  Actual: **yes**, because `u8: Encodable` does not exist, so `Vec<T: Encodable>` can never
  instantiate to `Vec<u8>`. Non-obvious and load-bearing.
- **P2 — does `[T]::encode` need a scratch buffer?** Predicted: no. Actual: yes, as written —
  it encodes children into a `Vec` to learn the payload length before writing the header.
  `length()` is arithmetic; `encode` is not. The derive later avoided this by summing
  `length()` first.
- **P3 — `length_of_length(0)`.** Predicted: 0. Actual: 1 — defined for all inputs so
  `string_length` can call it unconditionally, even though it is only *reached* for `l >= 56`.
- **P4 — how does `Vec<T>::decode` know where the list ends?** Predicted: decode until the
  buffer is empty. Actual: decode until the **payload window** is empty —
  `&buf[..payload_length]` — which is what stops an inner item from reading past the list.
  The derive reuses this shape.
- **P5 — tests predicted to fail.** Predicted: `vec_u8_is_a_string`.
