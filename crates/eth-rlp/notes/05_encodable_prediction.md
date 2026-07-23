# W005 Tue — `Encodable`/`Decodable` trait pre-predictions

> **Provenance**: reconstructed from the resulting code, not sealed beforehand.

- **P1 — `&mut dyn BufMut` or `impl BufMut`?** Predicted: `impl BufMut` (generic, no vtable).
  Actual: `&mut dyn BufMut`, matching alloy. Generics would make the trait non-object-safe,
  which the `encodable_is_object_safe` test explicitly forbids.
- **P2 — does `length` get a default body?** Predicted: yes, scratch-encode.
  Actual: **no default.** Alloy has one and it is the anti-pattern R4 forbids — see
  `../../eth-primitives/notes/05_alloy_diff.md` D1. Upstream now agrees (`alloy-rs/rlp#14`).
- **P3 — `Decodable: Sized` supertrait or method-level bound?** Predicted: supertrait, as
  alloy has. Actual: method-level `where Self: Sized`, to keep the trait object-safe. Real
  drift from alloy, deliberately taken.
- **P4 — error type shape.** Predicted: `Box<dyn Error>`. Actual: a flat `Copy` enum — no
  allocation on the error path, and `error_enum_is_flat_and_copy` pins it.
- **P5 — tests predicted to fail.** Predicted: `signatures_match_alloy_rlp`.
