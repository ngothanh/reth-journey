# W005 Sat — etherscanlite pre-predictions

> **Provenance**: reconstructed after the build. P1 is the exception — the first attempt was
> literally written into `boundary.rs` before the error appeared, so that prediction and its
> outcome are genuine. The rest are labelled honestly rather than dressed up as sealed.

## P1 — the conversion primitive, sight-unseen *(genuine)*

**Predicted**: `From`. The file was written as

```rust
impl From<alloy::primitives::B256> for B256 {
    fn from(value: alloy::primitives::B256) -> Self {}
}
```

**Does it compile? Predicted: yes. Actual: no — E0117.** The orphan rule requires the trait
or one of the types to be local to `etherscanlite`; `From` is `core`'s, `eth_primitives::B256`
is `eth-primitives`', `alloy_primitives::B256` is `alloy-primitives`'. Two things made this
counter-intuitive: *I wrote `eth-primitives`* (irrelevant — locality is per-crate, not
per-author), and `B256` *looks* local (it is a type alias, and aliases are transparent to
coherence).

This invalidated the plan's R3 as written. R3 has been amended.

## P2 — wall-clock for balance + nonce + tx-by-hash

**Predicted**: ~500 ms.

**Actual**: sub-second against `https://ethereum-rpc.publicnode.com`; the ignored smoke test
completes in 0.62 s including process spawn. Balance and nonce are issued concurrently via
`tokio::try_join!`, so the pair costs roughly one round-trip, not two.

## P3 — first-run outcome with `ETH_RPC_URL` unset

**Predicted**: fails at startup with a clear message.

**Actual**: correct, but only because it was built that way deliberately (D4). The failure
mode being guarded against is real — reading the variable lazily and handing an empty string
to the provider produces a connection error tens of seconds later, pointing at the network
instead of the operator's shell. `missing_env_var_is_reported_before_any_network_work`
asserts the error contains no "connect"/"dns"/"timed out" text.

## P4 — does the first-attempt parser reject a wrong-checksum mixed-case address?

**Predicted**: no.

**Actual**: no — `eth_primitives`' `FromStr` is plain hex decoding with no checksum logic,
so `parse_address` had to add the EIP-55 policy on top (D3).

## P5 — Phase-2 tests predicted to fail first attempt

**Predicted**: the checksum test and the 429 test.

**Actual**: the checksum test passed once the policy was written. The genuine surprises were
elsewhere: `u64::try_from` became ambiguous between `core::TryFrom` and the local
`TryFromAlloy` (E0034) *inside the boundary's own impl*, and R4's isolation rule turned out
to be unenforceable with the RPC calls in `main.rs` — alloy types leak there by inference
even with no `alloy_*` path in the source.
