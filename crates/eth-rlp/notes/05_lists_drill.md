# W005 Thu — 🧮 Paper drill: lists

## D-WALK — nested list, by hand

`vec![vec![1u64], vec![2u64, 3u64]]`

Inner 1: `[1]` → payload `01` (1 byte) → header `0xc0 + 1` = `c1` → **`c1 01`** (2 bytes)
Inner 2: `[2,3]` → payload `02 03` (2 bytes) → header `c2` → **`c2 02 03`** (3 bytes)
Outer payload = 2 + 3 = 5 → header `0xc0 + 5` = `c5`

```
c5 c1 01 c2 02 03        (6 bytes)
```

Note the outer header counts the inner *headers* as payload. That is why `length()` must be
recursive and why a scratch-encode default (alloy's) costs O(depth) allocations for a deeply
nested structure — each level buffers the level below.

## D-TABLE — `length_of_length`, by hand

| `l` | binary | `leading_zeros` (64-bit) | `(64 - lz + 7) / 8` | bytes |
|---|---|---|---|---|
| 0 | — | — | special-cased | 1 |
| 55 | `110111` | 58 | `(6+7)/8` = 1 | 1 |
| 56 | `111000` | 58 | 1 | 1 |
| 255 | `11111111` | 56 | `(8+7)/8` = 1 | 1 |
| 256 | `1_00000000` | 55 | `(9+7)/8` = 2 | 2 |
| 65535 | 16 ones | 48 | `(16+7)/8` = 2 | 2 |
| 65536 | — | 47 | `(17+7)/8` = 3 | 3 |

The `+7)/8` is a ceiling-divide on bits-to-bytes. Clippy suggests `div_ceil`; the arithmetic
is identical.

## D-WALK 2 — where the payload window matters

Hostile input: a list header claiming 4 payload bytes, followed by an inner string header
claiming 10.

```
c4 8a aa aa aa       ← outer says 4 bytes of payload; inner says 10 bytes of string
```

Without a window, the inner decode reads `buf[2..12]` — past the list, into whatever the
*next* item is. With `&buf[..4]`, the inner decode sees only 3 bytes remaining and returns
`InputTooShort`. The window is a containment boundary, not just bookkeeping.
