# W005 Wed/Thu — 5-year failure mode: scalar impls

**Trigger**: the first type that is an RLP *string* but whose natural Rust representation is
`Vec<u8>`, added by someone who does not know about the `u8`-is-not-`Encodable` decision.

**What breaks**: the coexistence of `impl Encodable for Vec<u8>` (string) and
`impl<T: Encodable> Encodable for Vec<T>` (list) is legal *only* because `u8: Encodable` does
not exist. The day anyone adds `impl Encodable for u8` — entirely reasonable-looking, and
alloy has it — those two impls overlap and the crate stops compiling with a coherence error
pointing at `Vec`, not at `u8`. The error will be baffling.

**Migration**: the fix is not to add `u8: Encodable`. It is to add a comment at the `u8`
gap explaining *why* it is absent — the comment at `encode.rs:123` does this for the `Vec`
pair, but there is nothing at the `uint_impl` site where someone would actually try to add
it. Add a negative test too: a `trybuild` compile-fail fixture asserting `impl Encodable for
u8` in a downstream crate produces the coherence error, so the constraint is executable
rather than folkloric.

**If we ever do need `u8` scalars**: encode them as `[u8; 1]` at the callsite, or introduce a
`U8(u8)` newtype. Both keep `Vec<u8>` unambiguous.
