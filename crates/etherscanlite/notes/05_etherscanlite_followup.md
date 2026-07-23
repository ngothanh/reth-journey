# W005 Sat — 5-year failure mode: etherscanlite boundary

**Trigger**: alloy ships a new major with breaking changes to its primitive types — `B256`
renamed, moved crates, or `FixedBytes` changing its inner representation.

**What breaks**: every `FromAlloy` / `ToAlloy` / `TryFromAlloy` impl in `boundary.rs`, plus
the `TxSummary` conversion. That is the point of confining them to one file — the blast
radius is one module, and the failure is a compile error rather than silent corruption
(see D1a: this is precisely what the byte copy buys over a `transmute`).

**Migration**: pin exactly rather than by caret — `alloy-primitives = "=1.6.1"` — and treat
each alloy major as a deliberate ticket rather than something `cargo update` does on a
Tuesday. Currently pinned by caret in `[workspace.dependencies]`; the exact pin is the
cheap insurance and should land before this crate has real downstream consumers.

**The second failure mode, which is nearer**: `U256` version skew. `eth_primitives::U256`
and `alloy_primitives::U256` are the *same* `ruint 1.18.0` type today, so the conversion is
the identity function. The moment the two crates resolve different `ruint` majors, that
identity impl becomes a type error in code that has worked untouched for months — and the
error will surface as "expected `Uint<256, 4>`, found `Uint<256, 4>`", which is one of the
more confusing messages Rust produces. Decide before W6 whether to newtype `U256`, pin
`ruint` exactly, or delete the fake conversion so the coupling is visible rather than
disguised. See `eth-primitives/notes/05_alloy_diff.md` D5.

**Third, lower priority**: the derive-crate hazard. Generated code from
`eth-primitives-derive` hardcodes `::eth_rlp::`, so any consumer must depend on `eth-rlp`
under exactly that name. `alloy` sidesteps this by re-exporting its derive from `alloy-rlp`
behind a `derive` feature. Adopt that pattern if the derive spreads beyond this workspace.
