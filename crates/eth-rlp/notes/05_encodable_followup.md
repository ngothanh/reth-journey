# W005 Tue — 5-year failure mode: the trait pair

**Trigger**: `alloy-rlp` ships the `alloy-rs/rlp#14` redesign — `RlpEncodable`/`RlpDecodable<'de>`
with `Encoder<T: BufMut>` / `Decoder<'de>` types, `rlp_len_raw()` with no default, and a
structured `Error { bytepos, kind }`.

**What breaks**: `signatures_match_alloy_rlp` — the type-system compat test asserting both
crates' traits are implemented for `u64`, `Bytes`, `Vec<u64>` — stops compiling, because the
trait names and method names change wholesale. Every downstream `impl Encodable` in this
workspace keeps working (we are not alloy's dependent), but the *drop-in compatibility*
claim in Tue R4 becomes false.

**Migration**: the compat test is the tripwire, and it is deliberately cheap — a `fn
assert_compat<T: alloy_rlp::Encodable + eth_rlp::Encodable>() {}` invoked on three types.
When it breaks, decide explicitly: follow alloy to the new API, or declare independence and
delete the test. Do not silently `#[ignore]` it — that converts a decision into drift.

**Note in our favour**: two of the redesign's changes are things we already do. `rlp_len_raw`
has no default (our `length` never did), and the byte-position-carrying error is close to
what `Header::decode` could report today, since it already knows the cursor at every failure.
If we follow, we are following toward where we already are.
