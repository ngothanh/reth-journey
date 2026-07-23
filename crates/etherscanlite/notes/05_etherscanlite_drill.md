# W005 Sat — 🧮 Paper drill: etherscanlite

## D-LAYOUT — `alloy::B256` vs `eth_primitives::B256` vs `U256`

Both `B256`s are `#[repr(transparent)]` over `[u8; 32]`, big-endian by convention:

```
byte:   0    1    2   …   29   30   31
      ┌────┬────┬────┬───┬────┬────┬────┐
B256  │ b0 │ b1 │ b2 │ … │b29 │b30 │b31 │     b0 = most significant
      └────┴────┴────┴───┴────┴────┴────┘
```

`U256` (ruint, both sides) is four **little-endian** `u64` limbs:

```
limb:      0                1                2                3
      ┌──────────────┬──────────────┬──────────────┬──────────────┐
U256  │ bits 0..63   │ bits 64..127 │ bits 128..191│ bits 192..255│
      └──────────────┴──────────────┴──────────────┴──────────────┘
        least significant                          most significant
```

**The divergent byte.** Take the value `1`.

- As `B256` big-endian: `00 00 … 00 01` — the `01` is at **byte 31**.
- As `U256` limbs viewed as raw memory on a little-endian machine: `01 00 … 00` — the `01`
  is at **byte 0**.

So byte 0 and byte 31 swap roles. Reading a `U256`'s raw limb memory as if it were a `B256`
turns `1` into `2^248`. Ethereum's wire form is big-endian, so the conversion must route
through `to_be_bytes()` / `from_be_bytes()`, never through a limb memcpy or a transmute.

This is exactly the D1a argument: if alloy ever changed `FixedBytes` to store `[u64; 4]`,
a `transmute` would keep compiling and start producing this reversal silently.

## D-WALK — EIP-55 on one mixed-case input

Input: `0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed`

1. Lowercase the hex body: `5aaeb6053f3e94c9b9a09f33669435e7ef1beaed`.
2. `keccak256` of those 40 **ASCII characters** (not the 20 decoded bytes — a classic
   off-by-one-layer bug). Hash begins `0x0e...`.
3. For hex character *i*, take nibble *i* of the hash. If the nibble ≥ 8, the character is
   uppercase; otherwise lowercase.
4. Compare to the input.

| i | char | hash nibble | ≥ 8? | expected |
|---|---|---|---|---|
| 0 | `5` | 0 | no | `5` (digits unaffected) |
| 1 | `a` | e | yes | `A` |
| 2 | `a` | 6 | no | `a` |
| 3 | `e` | 3 | no | `e` |

Digits `0-9` have no case, so their nibble is irrelevant — only `a-f` carry a bit of
checksum each. 40 characters, of which ~25 are letters on average ⇒ roughly 25 bits of
protection. Enough to catch a typo, **not** enough to be a security boundary.

**Decision table** (D3):

| Input casing | Claim | Action |
|---|---|---|
| all lower | none | accept |
| all upper | none | accept |
| mixed | EIP-55 | recompute; reject on mismatch |

Flipping character 3 to `E` gives `0x5aAEb60…`, still valid hex, still the same 20 bytes,
but nibble 3 is `3` (< 8) so lowercase was required — rejected. That is the test
`rejects_a_wrong_mixed_case_checksum`.
