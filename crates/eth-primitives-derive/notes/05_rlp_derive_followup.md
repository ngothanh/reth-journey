# W005 Fri — 5-year failure mode: RLP derive

**Trigger**: the first derived struct that needs field-level customisation. Concretely
EIP-7702 authorization lists, where some fields are omitted on encode but required on
decode, and any struct with a field that should not participate in the hash.

**What breaks**: the derive has no attribute parsing at all. `#[proc_macro_derive(RlpEncodable)]`
declares no `attributes(...)`, so `#[rlp(skip)]` on a field is not merely ignored — it is a
hard "cannot find attribute `rlp`" compile error at the *use* site, which reads as the
user's mistake rather than a missing feature.

**Migration**:

1. `#[proc_macro_derive(RlpEncodable, attributes(rlp))]` on both derives.
2. Parse per-field `#[rlp(skip)]` / `#[rlp(default)]` in `named_fields`, returning a
   `Vec<(Ident, Type, FieldOpts)>` instead of the raw `Punctuated`.
3. Encode side: skipped fields drop out of both the `length` sum and the encode sequence.
4. Decode side: skipped fields are **not** read from the payload — they take
   `Default::default()`. This is the part that silently corrupts if done wrong, because the
   payload-window trailing-bytes check will still pass as long as the *remaining* fields
   consume everything.
5. Add a fixture pinning the bytes for a struct with a skipped field, and a round-trip test
   asserting the skipped field comes back as its default rather than as whatever the
   previous value was.

**Second-order risk**: once `skip` exists, encode and decode are no longer inverses, and the
`_round_trip` tests stop being a complete correctness statement. At that point the derive
needs an explicit "encode-decode is lossy for skipped fields" doc-comment, and the golden
fixtures become the primary check rather than the round trips.

**Why not now**: no current consumer needs it, and attribute parsing roughly doubles the
derive's surface. Adding it speculatively would mean maintaining an untested code path
through W6–W9, which is when the derive is under the most change pressure.
