# Static vs Dynamic Dispatch

Reference notes on Rust's two dispatch mechanisms — how they compile, what they cost, and when to choose which. Examples grounded in real Rust/Ethereum code.

## The two mechanisms

### Static dispatch — monomorphization

Form: `fn f<T: Trait>(x: T)` or `fn f(x: impl Trait)`.

The compiler stamps out a **separate specialized copy** of the function for every concrete type passed in. `f(Address::ZERO)` and `f(B256::default())` produce two distinct function bodies in the final binary. Each copy hard-codes the method calls — no runtime lookup.

What the binary contains, conceptually:

```rust
// You wrote:
fn encode<T: Encodable>(item: T) { item.encode_to(...); }

// Compiler emits:
fn encode__Address(item: Address) { /* inlined Address::encode_to */ }
fn encode__B256(item: B256)       { /* inlined B256::encode_to */ }
fn encode__Bytes(item: Bytes)     { /* inlined Bytes::encode_to */ }
// one copy per concrete type you actually called with
```

### Dynamic dispatch — trait objects

Form: `fn f(x: &dyn Trait)`, `Box<dyn Trait>`, `Arc<dyn Trait>`.

The compiler emits **one function**. The trait object value carries two pointers at runtime:

1. A pointer to the data.
2. A pointer to a **vtable** — a struct containing function pointers for every method of the trait, populated with the concrete type's impls.

When the function calls `x.encode_to(...)`, it dereferences the vtable pointer, finds the `encode_to` slot, and calls through that function pointer. The method to call is resolved at runtime, not compile time.

Memory layout of a `&dyn Encodable`:

```
┌─────────────┐    ┌──────────────┐
│ data ptr    │───▶│ Address bytes│
├─────────────┤    └──────────────┘
│ vtable ptr  │───▶┌────────────────────┐
└─────────────┘    │ encode_to: fn(...) │  → Address::encode_to
                   │ length:    fn(...) │  → Address::length
                   │ drop:      fn(...) │  → Address's drop_in_place
                   │ size:      usize   │
                   │ align:     usize   │
                   └────────────────────┘
```

## Trade-offs

| Dimension | Static (monomorphization) | Dynamic (trait object) |
|---|---|---|
| Runtime cost per call | Zero — direct call, often inlined | One pointer indirection (vtable lookup) |
| Binary size | Grows with type count — one copy per concrete `T` | Constant — one function, vtables are small |
| Compile time | Slower — type-checks N times | Faster — type-checks once |
| Inlining | Yes — compiler can inline through the call | No — vtable indirection blocks inlining |
| Type erasure | No — every `T` is visible at compile time | Yes — concrete type forgotten at call site |
| Heterogeneous collections | Impossible — `Vec<T>` is one type | Possible — `Vec<Box<dyn Trait>>` |
| Object safety required | No | Yes |

## Object safety — the constraint that forces choice

A trait is **object-safe** if you can create `dyn Trait` from it. If any method violates these rules, you can't use the trait dynamically — you're forced to static dispatch:

1. **No generic methods.** Each method must have concrete parameter types.
   - ❌ `fn write<W: io::Write>(&self, w: &mut W)`
   - ✅ `fn write(&self, w: &mut dyn io::Write)`

2. **No `Self` in argument or return positions** (with narrow exceptions like `Self: Sized` bounds).
   - ❌ `fn clone(&self) -> Self`
   - ❌ `fn consume(self)`
   - ✅ `fn clone_boxed(&self) -> Box<dyn Trait>`

3. **No associated types without explicit binding at the use site.**
   - ❌ `dyn Iterator` (it has `type Item;`)
   - ✅ `dyn Iterator<Item = u8>` (must specify)

4. **Methods must have a receiver** — `&self`, `&mut self`, `Box<Self>`, `Pin<Self>`, etc. — except those marked `where Self: Sized` (which then can't be called on the trait object).

The compiler enforces these. If you try to write `&dyn Trait` and the trait isn't object-safe, you get a compile error directing you to the violating method.

## When to choose which

### Reach for static (default)

- The trait is used in **hot paths** — every nanosecond matters (EVM interpreter, hash computations, RLP encoding).
- You want **inlining** — the compiler should see through to the impl.
- You don't need **heterogeneous collections** of trait-implementers.
- The trait has methods that **can't be object-safe** (generic methods, `Self` returns).

Static is the default in most Rust codebases. Trait objects are the exception, not the norm.

### Reach for dynamic when

- You need a **collection of different types behind one interface**: `Vec<Box<dyn Encodable>>` for a list of mixed transaction types.
- The trait is used outside hot paths — UI handlers, error variants, plugin/adapter boundaries.
- Binary size matters more than per-call speed — embedded contexts, large generic-heavy crates.
- You're building **plugin or trait-object-based architectures** where types aren't known until runtime.

### "Static by default, dynamic at boundaries" rule

A common pattern in production Rust code:

- **Internal APIs**: static (`impl Trait`, `<T: Trait>`).
- **Boundary APIs** (where you genuinely need to erase types): dynamic (`Box<dyn Trait>`, `&dyn Trait`).

You write fast code internally; you accept one vtable indirection at the boundary where types meet.

## Real-world examples

### alloy-rlp `Encodable`

```rust
pub trait Encodable {
    fn encode(&self, out: &mut dyn BufMut);
    fn length(&self) -> usize;
}
```

Designed object-safe on purpose: `&mut dyn BufMut` instead of `&mut impl BufMut`. The cost of one vtable lookup on the buffer side is negligible; the benefit is that `Vec<Box<dyn Encodable>>` works — useful for things like a heterogeneous list of pre-encoded items, or RLP encoding implementations stored behind a trait object.

### `std::error::Error`

```rust
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)>;
    // ...
}
```

The whole error system is built around `dyn Error` because errors propagate up the stack across many types. You can't predict at compile time which error variants you'll handle, so type erasure via `Box<dyn Error>` is the practical model. `anyhow::Error` is essentially `Box<dyn Error>` with extras.

### `std::iter::Iterator`

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // ...
}
```

**Not object-safe by default** — `type Item` has no specific binding. To use as `dyn`, you must specify: `dyn Iterator<Item = u8>`. In practice, almost all Iterator use is generic (`impl Iterator<Item = T>`) — the type is visible and the compiler inlines through. Dynamic iterators show up only when you genuinely need to erase the source.

### revm's `Database` trait

```rust
pub trait Database {
    type Error;
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;
    // ...
}
```

Used statically: `Evm<DB: Database>`. The reason is performance — the EVM hits the database on every state read. A vtable indirection per `SLOAD` would matter. Monomorphization compiles the database type into the EVM, allowing inlining.

## Quick decision table

| Situation | Pick |
|---|---|
| Hot path, single type per call site | Static |
| Heterogeneous collection | Dynamic |
| Plugin / runtime-extensible boundary | Dynamic |
| Trait has generic methods or returns `Self` | Forced to static |
| Library API exposed to many callers (small binary matters) | Dynamic |
| Performance-critical internal helper | Static |
| Error propagation | Dynamic (`Box<dyn Error>`) |
| Iterator chains | Static (`impl Iterator`) |

## Two-line summary

**Static dispatch** trades binary size for speed — many specialized functions, zero overhead, full inlining.

**Dynamic dispatch** trades speed for flexibility — one function, runtime lookup, heterogeneous collections.

Most production Rust is static by default; dynamic shows up at type-erasure boundaries (errors, plugins, UI event handlers). When designing a trait, ask: *will anyone need to put different types of this in a `Vec`?* If yes, keep it object-safe. If no, don't bother — make your methods as expressive as you want.
