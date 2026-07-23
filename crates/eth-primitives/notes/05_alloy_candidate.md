# W005 — First Alloy issue scan

Scanned `alloy-rs/rlp` and `alloy-rs/core` open issues (July 2026). `alloy-primitives`
lives in `alloy-rs/core`; `alloy-rlp` has its own repo and, notably, **exactly one open
issue** — a small, quiet crate, which cuts both ways: little competition for the work, but
also few maintainer eyes.

Preference per the plan: `alloy-rlp` / `alloy-primitives`, the two crates mirrored this
week, where Tue–Fri context is freshest.

---

## Front-runner — `alloy-rs/rlp#14` "New and improved library and macro API"

**Repo/file**: `alloy-rs/rlp`, `crates/rlp/src/{encode,decode,header}.rs` and
`crates/rlp-derive/src/lib.rs`.

**What it asks for**: a full trait redesign — `RlpEncodable`/`RlpDecodable<'de>` with an
`Encoder<T: BufMut>` / `Decoder<'de>` pair replacing the free functions and `Header`
methods, plus a structured `Error { bytepos, kind }` carrying byte position.

**Why this one**: the issue body specifies

```rust
fn rlp_len_raw(&self) -> usize; // Important: no default!!
```

which is precisely the divergence recorded as D1 in `05_alloy_diff.md` — our `eth-rlp`
already has no default on `length()`, for exactly the reason the issue gives. I have spent
this week building both halves of the thing being redesigned (traits, `Header`, scalar
impls, and a working `RlpEncodable`/`RlpDecodable` derive with payload-window decoding and
a trailing-bytes check), so the design context is loaded.

**Repro/fix sketch**: this is a design issue, not a bug, so the first contribution is not
the whole redesign — it is the smallest reviewable slice that moves toward it. Concretely:

1. Comment on #14 confirming the `no default` decision with the argument from D1 — that
   the current default silently converts a forgotten `length` into an allocating scratch
   encode, and that arithmetic length is what makes nested list headers O(1) rather than
   O(depth) in allocations. Offer the `derive_rlp_length_is_arithmetic_not_encode`
   hostile-encodable test as the mechanism that catches it.
2. If maintainers are receptive, take the *error* half first: `Error { bytepos, kind }` is
   separable from the trait redesign, is mechanical, and is independently useful. It also
   maps onto work already done — our `Header::decode` already knows the cursor position at
   every failure point.

**Risk**: a tracking-style redesign issue may be maintainer-owned. Confirm intent in a
comment before writing code.

---

## Backups

**`alloy-rs/core#999` — "[Feature] Move alloc feature behind a feature gate"**
*(the only `good first issue` on either repo)*
`alloy-primitives`. Remove the `alloc` dependency so `Uxxx`/`Ixxx`/`Address` work without
it; `Bytes` is the one type that would be gated off. Author says formatting paths can move
to stack buffers (`arrayvec::ArrayString`) since maximum string sizes are all known.
*Fit*: touches `Address`/`FixedBytes` formatting, which I reimplemented in `hex.rs` and
`address.rs` (including `to_checksum`, which is a `String`-building path and would need
exactly this treatment). *Risk*: the original reporter offered to implement it — check
whether it is already claimed before starting.

**`alloy-rs/core#900` — "[Feature] Unified type convert api of U256 and I256"**
`alloy-primitives`. `U256::to::<u64>()` exists but `I256::as_u64()` is the only direction
on the signed side; no `I256::as_u128`/`as_i128`; `f64::from(U256)` but no
`U256::to::<f64>()`. *Fit*: I hit `U256 → u64` narrowing this week in the etherscanlite
boundary and had to route around a `try_from` ambiguity to do it. Mechanical, well-scoped,
API-shaped. *Risk*: bikeshed potential on naming.

**`alloy-rs/core#828` — "Make UInt panic on overflow if `overflow_checks` is enabled"**
`alloy-primitives`. Honest fit but it lands in `ruint` semantics rather than alloy's own
code, so the surface is less familiar and the blast radius is wider.

---

## Note on repo health

`alloy-rs/rlp` having one open issue means the front-runner is uncontested but may also be
dormant. If #14 draws no maintainer response within a week, switch to `#999` in
`alloy-rs/core`, which is labelled `good first issue`, is scoped to `primitives`, and has
a clearly stated acceptance shape.
