# W005 Tue — 🧮 Paper drill: the trait pair

## D-SIG — why `&mut dyn BufMut` and not `impl BufMut`

```rust
fn encode(&self, out: &mut dyn BufMut);       // object-safe
fn encode<B: BufMut>(&self, out: &mut B);     // NOT object-safe
```

A generic method cannot be dispatched through a vtable, because the compiler would need one
vtable entry per instantiation. So the second form makes `dyn Encodable` illegal.

Why that matters: `impl<T: Encodable> Encodable for [T]` and the derive both want to hold
heterogeneous children, and W9's `TxEnvelope` will want `Box<dyn Encodable>` for transaction
variants. Paying one indirect call per `encode` buys that. The cost is a vtable dispatch per
call — measurable only if `encode` is called per *byte*, which it never is; it is called per
*field*.

## D-WALK — object safety, by hand

For `dyn Encodable` to exist, every method must be callable through a pointer with no
knowledge of `Self`'s size:

| Method | Object-safe? | Why |
|---|---|---|
| `fn encode(&self, out: &mut dyn BufMut)` | yes | `&self` is a thin receiver; no generics |
| `fn length(&self) -> usize` | yes | no `Self` in the signature beyond the receiver |
| `fn decode(buf: &mut &[u8]) -> Result<Self, Error>` | **no** | returns `Self` by value |

That last row is why `Decodable::decode` carries `where Self: Sized` — the bound *excludes
that one method* from the vtable, leaving the rest of the trait object-safe. Putting `Sized`
on the trait instead (alloy's choice) excludes the whole trait.

Encode and decode therefore cannot be one trait: encode is object-safe, decode is not.
