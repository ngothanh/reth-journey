# W005 Thu — lists, and the `length_of_length` helper

> ⚠️ **Scope gap, recorded honestly.** The W005 Thursday Build reads "Encodable for
> `Vec<T: Encodable>`, `Option<T>`, tuples, arrays. `length_of_length` helper." Of these,
> **`Vec<T>`, `Vec<u8>`, `[T]`, `[u8]` and `length_of_length` exist; `Option<T>`, tuples and
> `[T; N]` do not.** The item is checked in the plan; the impls are not written. Same class
> of gap as the Wednesday item, which was checked while `B256`/`Address`/`Bytes` had no
> impls until Friday. See "Not yet built" below.

## What exists

| Impl | Kind | Notes |
|---|---|---|
| `Encodable for [u8]` | string | owns the single-byte-`< 0x80` fixed point |
| `Encodable for Vec<u8>` | string | delegates to `[u8]` |
| `Encodable for [T: Encodable]` | list | scratch-buffers the payload |
| `Encodable for Vec<T: Encodable>` | list | delegates to `[T]` |
| `Encodable for &T` | passthrough | blanket, `T: ?Sized` |
| `Decodable for Vec<T>` | list | decodes within a payload window |

The `Vec<u8>` / `Vec<T>` pair does not overlap **only** because `u8` has no `Encodable`
impl. That is load-bearing — see `05_encodable_scalars_followup.md`.

## `length_of_length`

```rust
fn length_of_length(l: usize) -> usize {
    if l == 0 { 1 } else { ((usize::BITS - l.leading_zeros() + 7) / 8) as usize }
}
```

Bytes needed to write `l` in minimal big-endian. Only reached for `l >= 56`, but defined
for all `l` so `string_length` can reuse it. Note it is **private**, together with
`string_length` — which is why `Header::length()` had to be added on Friday when the derive
needed to compute a list header's size arithmetically from outside the module.

Alloy exports the equivalent publicly, and confusingly names it after the *header* length
rather than the length-of-the-length. Ours is the literal reading.

## The asymmetry worth knowing

`[T]::encode` scratch-buffers into a `Vec` to discover the payload length, then writes the
header. `[T]::length` does it arithmetically. So `encode` allocates and `length` does not —
the reverse of the usual expectation, and the exact thing the derive avoids by summing
`length()` first and writing the header before the payload. The slice impl could adopt the
same trick; it has not, because nothing yet encodes a large `[T]` on a hot path.

## Not yet built

- `Option<T>` — the interesting question is whether `None` is the empty string (`0x80`) or an
  empty list (`0xc0`). Alloy treats `Option<T>` as "omit if `None`", which only composes
  inside a struct, not standalone. Decide before W7 (`Authorization` has optional fields).
- Tuples — needed for heterogeneous fixed-arity lists; the derive covers the named-struct
  case, so this is only for ad-hoc use.
- `[T; N]` — note `[u8; N]` must stay a *string* (alloy agrees; our Friday parity test
  depends on it), while `[T; N]` for other `T` is a list. Same coherence trap as `Vec`.
