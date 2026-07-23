# W005 Wed — 🧮 Paper drill: `Header`

## D-TABLE — the whole prefix space, by hand

| First byte | Kind | Payload | Header bytes |
|---|---|---|---|
| `0x00`–`0x7f` | string | the byte **itself** | **0** — no header at all |
| `0x80`–`0xb7` | string | `first - 0x80` (0..55) | 1 |
| `0xb8`–`0xbf` | string | read `first - 0xb7` length bytes | `1 + n` |
| `0xc0`–`0xf7` | list | `first - 0xc0` (0..55) | 1 |
| `0xf8`–`0xff` | list | read `first - 0xf7` length bytes | `1 + n` |

Row 1 is the fixed point that breaks a naive "always write a header" encoder. Note the
cursor consequence: for `0x00..=0x7f`, `Header::decode` returns `payload_length: 1` and
**does not advance the buffer** — the byte is the payload.

## D-WALK — four encodings by hand

| Value | Reasoning | Bytes |
|---|---|---|
| `0x42` (one byte < 0x80) | fixed point | `42` |
| `0x80` (one byte ≥ 0x80) | needs a header: `0x80 + 1` | `81 80` |
| `""` (empty string) | `0x80 + 0` | `80` |
| 56 bytes of `0xaa` | 56 > 55 ⇒ long form. `0xb7 + 1` = `b8`, then `0x38` | `b8 38 aa×56` |

## D-WALK 2 — the two canonicity traps on decode

**Trap 1 — leading zero in the length.** `b9 00 38 …` claims a 2-byte length `0x0038` = 56.
Decodes to the same value as `b8 38`, so two byte strings mean one value ⇒ not canonical,
and a consensus hash computed over either would differ. Reject when `len_bytes[0] == 0`.

**Trap 2 — long form for a short payload.** `b8 20 …` claims 32 bytes via the long form,
but 32 ≤ 55 so the short form `0xa0` was required. Reject when `payload_length < 56` in the
long branch.

Both are `Error::NonCanonical`. Alloy splits these into `LeadingZero` and `NonCanonicalSize`
— see `../../eth-primitives/notes/05_alloy_diff.md`.
