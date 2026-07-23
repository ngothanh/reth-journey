# W005 Sat — etherscanlite design

## Problem

alloy-provider returns `alloy_primitives` types. The app runs on `eth-primitives` types.
Without a thin boundary the two families either spread through the whole app (pulling all of
alloy in, defeating the mirror discipline) or get hand-converted at every callsite
(bug-prone; endianness and checksum drift). The CLI is the smallest realistic demand.

## Requirements

- **R1** — binary crate with `main.rs` and a thin `boundary.rs`.
- **R2** — `etherscanlite <address> [--tx <hash>]` prints balance (wei), nonce, and
  optionally one transaction's hash/value/gas. **Amended** from "last 5 transactions" — see
  below.
- **R3** — `boundary.rs` defines the crate's own conversion traits and holds one impl per
  boundary type. **Amended** from `From`/`TryFrom` — see below.
- **R4** — no alloy type outside the boundary, verified mechanically.
- **R5** — `ETH_RPC_URL` from the environment only; `.envrc` documented in the README.
- **R6** — 429 retries with exponential backoff, ≤3 attempts.
- **R7** — `#[tokio::main]`, `anyhow` for reporting.

## Two amendments, both forced by reality

**R3 cannot be `From`.** E0117 — see `05_etherscanlite_prediction.md` P1. The fix is a local
trait: `FromAlloy<T>`, `TryFromAlloy<T>`, and `ToAlloy<T>` for the reverse direction (needed
by the round-trip tests, and blocked by the identical rule). Declaring the trait here is what
makes every impl legal.

**R2 cannot be "last 5 transactions".** No standard JSON-RPC method returns transactions by
address. `eth_getLogs` covers events but not plain value transfers; walking blocks backwards
needs an archive node. History-by-address is an indexer feature (`alchemy_getAssetTransfers`,
Etherscan's REST `account/txlist`), so requiring it would pin the crate to one vendor —
breaking R5's "any endpoint" — or add a second HTTP client outside alloy-provider. Fetch-by-
hash exercises the `Transaction` conversion identically.

## Design walk-through

**D1 — the impl header.** `impl FromAlloy<alloy_primitives::B256> for eth_primitives::B256`.
The two levers that satisfy the orphan rule are *make the trait local* or *make a type local*
(newtype the alloy side). Chose the trait: the newtype lever compiles too, but taxes every
callsite with a wrapper (`B256::from(Wrapper(hash))`). One trait could cover both directions
— `impl FromAlloy<eth_primitives::B256> for alloy::B256` is legal, since the trait is what is
local — but `to_alloy()` reads correctly at the callsite where `from_alloy` would read
backwards, so `ToAlloy` is separate.

**D1a — byte copy, not `mem::transmute`.** Both sides are `#[repr(transparent)]` over
`[u8; 32]`, so a transmute is sound *today*. It buys nothing: 32 bytes is a couple of cycles
and frequently elided entirely, against a 10–100 ms round trip. It costs the compile error
that would otherwise fire if alloy changed its inner representation — transmute only checks
*size*, and size equality is not layout equality. If alloy moved to `[u64; 4]`, the copy stops
compiling and the transmute silently returns byte-reversed hashes forever.

**D2 — `U256` conversion is `from_be_bytes`, not limb-copying** *in principle*. In practice
both sides are the same `ruint 1.18.0` alias, so the conversion is the identity function.
See `eth-primitives/notes/05_alloy_diff.md` D5 — this is a finding, not a convenience, and
it means `U256` has no real boundary.

**D3 — EIP-55 policy.** The input is self-declaring. All-lowercase and all-uppercase claim no
checksum and are accepted as-is (the industry default — rejecting them would break every
lowercase address in every config file). Mixed case *is* a checksum claim and is validated
against `Address::to_checksum`, with the correct casing offered in the error.

**D4 — fail fast on config.** `ETH_RPC_URL` is read on line 1 of `main` with `.context(...)`.
Lazy reading surfaces a missing variable as a connection error deep in the provider stack.
Arguments are parsed *before* connecting, too, so a typo'd address fails instantly.

**D5 — 429 policy: exponential.** W3's `RetryFuture` owns the retry count but has no notion of
delay — it re-polls immediately, and retrying a rate-limit with no wait is worse than not
retrying. `retry.rs` wraps it and injects the sleep per attempt: 0 / 100 ms / 200 ms. This is
the "if its signature doesn't fit, extract the abstraction at the boundary" escape the plan
anticipated.

**D6 — `clap`.** Chosen over `argh`/hand-rolled for the derive API and because W22 adds
subcommands. Compile-time cost is real but paid once in a leaf binary.

**D7 — pre-mortem: someone copy-pastes `alloy::B256::default()` into a non-boundary module.**
`tests/isolation.rs` walks `src/`, skips comment lines, and fails on any of
`alloy_primitives` / `alloy_rpc_types_eth` / `alloy_consensus` / `alloy_provider` / `alloy::`
outside an allowlist. Verified non-vacuous: injecting `use alloy_primitives::B256 as Leak;`
into `main.rs` fails the test with file and line; reverting passes it.

## The R4 problem the plan did not anticipate

R4 as written ("no alloy outside `boundary.rs`") is unachievable while the RPC calls live in
`main.rs`: `provider.get_balance()` returns `alloy_primitives::U256` whether or not the word
appears in the source. The grep would pass while isolation was already broken.

Fix: `client.rs` owns the provider and exposes methods taking and returning **local types
only**. `main.rs` is then genuinely alloy-free, and the allowlist is two files instead of one.
The narrowed dependencies (`alloy-primitives` + `alloy-provider` rather than the `alloy`
umbrella) matter here too — under the umbrella the real path is `alloy::primitives::B256`,
which the plan's `alloy_primitives::` grep would have missed entirely.

## Output

24 tests: 6 boundary round-trips, 8 parsing, 4 backoff, 7 CLI (1 ignored, live), 2 isolation,
1 trybuild compile-fail pinning the orphan-rule decision.
