# W005 Wed — 5-year failure mode: `Header`

**Trigger**: a single RLP item legitimately exceeding `u32::MAX` bytes, or a hostile peer
claiming one does.

**What breaks**: `payload_length: usize` is 64-bit on our targets, so a malicious long-form
header can declare a payload of up to `usize::MAX`. `Header::decode` already guards the
*read* (`buf.len() < 1 + len_of_len` ⇒ `InputTooShort`), and the derive re-checks
`buf.len() < payload_length` before slicing — so there is no over-read today. The exposure is
arithmetic: any future `payload_length + something` in a length computation can overflow, and
in release builds that wraps silently.

**Migration**: cap `payload_length` explicitly at `u32::MAX` on decode and return
`Error::Overflow` above it. No legitimate Ethereum RLP item approaches 4 GiB — the largest
realistic single item is a block body in the tens of MiB — so the cap costs nothing and turns
a class of arithmetic bug into a rejected frame. Audit every `+` on a length while doing it:
`string_length`, `[T]::length`, and the derive's `__payload_length` sum are the three sites.

**Trigger condition to actually do this**: when `eth-rlp` first decodes bytes from an
untrusted peer rather than from a test fixture — i.e. W6 network ingestion, not before.
