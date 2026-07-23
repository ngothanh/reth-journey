# etherscanlite

A minimal CLI that reads an account's balance and nonce from an Ethereum RPC endpoint,
and optionally fetches one transaction by hash.

Its real job is to be the first place `eth-primitives` types meet the live network, so
the interesting code is the type boundary, not the CLI.

## Usage

```
etherscanlite <ADDRESS> [--tx <HASH>]
```

```console
$ etherscanlite 0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed
address: 0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed
balance: 1000000000000000000 wei
nonce:   42
```

Addresses are EIP-55 aware. All-lowercase and all-uppercase input claims no checksum and
is accepted as-is; **mixed-case input is a checksum claim and is validated**, so a typo
that would otherwise send funds to a wrong-but-valid address is rejected up front.

## Configuration

`ETH_RPC_URL` is required and is read at startup, before any argument is dialled — a
missing variable is a configuration error, not a connection timeout thirty seconds in.

```sh
export ETH_RPC_URL="https://eth-mainnet.g.alchemy.com/v2/<YOUR_KEY>"
```

The key never belongs in source. For a per-directory setup with [direnv](https://direnv.net):

```sh
# .envrc  — add to .gitignore, do not commit
export ETH_RPC_URL="https://eth-mainnet.g.alchemy.com/v2/<YOUR_KEY>"
```

Any JSON-RPC endpoint works; nothing here is vendor-specific.

## Layout

| File | Role |
|---|---|
| `main.rs` | Argument parsing, EIP-55 validation, output. Contains no alloy types. |
| `boundary.rs` | `FromAlloy` / `ToAlloy` / `TryFromAlloy` and one impl per boundary type. |
| `client.rs` | RPC calls. Takes local types, returns local types. |
| `retry.rs` | Exponential backoff for rate-limited calls. |

`boundary.rs` and `client.rs` are the only modules permitted to name an alloy type, and
`tests/isolation.rs` enforces that mechanically rather than by convention.

### Why not `From`?

`impl From<alloy_primitives::B256> for eth_primitives::B256` cannot be written here: the
orphan rule (E0117) requires the trait or one of the types to be local to this crate, and
none of the three is. Defining the conversion traits in `boundary.rs` makes them local,
which is what makes every impl legal.

### Why no transaction history

There is no standard JSON-RPC method that returns transactions by address. `eth_getLogs`
covers events but not plain value transfers, and scanning blocks backwards requires an
archive node. History-by-address is an indexer feature (`alchemy_getAssetTransfers`,
Etherscan's REST API), and depending on one would tie this crate to a single vendor.
`--tx <hash>` exercises the same conversion path without that cost.

## Tests

```sh
cargo test -p etherscanlite                       # unit + CLI + isolation
cargo test -p etherscanlite -- --ignored          # adds the live smoke test
```

The smoke test needs a working `ETH_RPC_URL` and is ignored by default so CI does not
burn the API quota.
