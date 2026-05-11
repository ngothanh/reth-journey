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

---

# Error Handling: Three Rewrites of `parse_address`

The same parsing logic written three ways to feel the trade-offs. Only #2 (Result + thiserror) lives in the crate; #1 and #3 exist for comparison.

## 1. Panic on bad input

```rust
fn parse_address(s: &str) -> Address {
    s.parse().unwrap()
}
```

**What it costs the caller:** Cannot recover from bad input. Any malformed hex aborts the thread (or process, in single-threaded code). Callers must either trust the input absolutely or wrap in `std::panic::catch_unwind`, which is a runtime cost and a code smell.

**What it gives the caller:** Terse call site (`let addr = parse_address(s);`). No error type to propagate, no `?`, no `match`. Reads like a total function.

**When this is OK:**
- Test fixtures where input is hardcoded and any failure is a test bug, not user error.
- Internal helpers behind invariants that the caller has already enforced.
- One-shot scripts where panicking IS the error handler.

**When this is wrong:**
- Library APIs. *Always.* A library that panics on its public surface forces every downstream caller to either trust the input absolutely or pay catch_unwind overhead.
- Any code path reachable from user input (CLI args, RPC payloads, file contents, network bytes).
- Long-running services where a single bad input shouldn't bring down the process.

The mental model: **panic is for bugs, not for runtime conditions.** A bad hex string is a runtime condition.

## 2. `Result` + `thiserror` — the canonical form (what we keep)

```rust
pub fn parse_address(s: &str) -> Result<Address, PrimitivesError> {
    s.parse()
}
```

**What it costs the caller:** One `?` at each call site, or one `match`/`if let` to handle the error. The error type appears in the function signature, so callers know to handle it.

**What it gives the caller:**
- **Type-specific error variants.** Callers can `match` on `PrimitivesError::InvalidLength`, `InvalidHex`, etc. and handle each differently (retry on length mismatch with padding, log on bad char, etc.).
- **Composability.** Other library functions return `Result<_, PrimitivesError>`; `?` composes them effortlessly.
- **No allocation on the happy path.** `PrimitivesError` is a stack-sized enum.
- **Documentation by signature.** `Result<Address, PrimitivesError>` tells the reader what can go wrong without reading the body.

**Why this for libraries:**
The split rule from `thiserror`/`anyhow` reading: **libraries use `thiserror`**. The reason is that library consumers need to *handle* specific errors, not just *propagate* them. A `Result<Address, PrimitivesError>` lets a consumer decide:

```rust
match parse_address(s) {
    Ok(addr) => use_it(addr),
    Err(PrimitivesError::InvalidLength { expected, got }) => {
        // maybe pad or trim?
    }
    Err(PrimitivesError::InvalidHex(_)) => {
        // log and ask user to retry
    }
    Err(_) => bail!("unrecoverable"),
}
```

If we returned `anyhow::Error` instead, all three branches would collapse into one — the consumer would have to downcast to recover specific types, which is fragile and verbose.

## 3. `anyhow::Result` — application-style

```rust
fn parse_address_anyhow(s: &str) -> anyhow::Result<Address> {
    use anyhow::Context;
    s.parse().context("invalid Ethereum address")
}
```

**What it costs the caller:** The error is a `Box<dyn Error + Send + Sync>`. Callers cannot `match` on specific variants without `downcast_ref::<PrimitivesError>()`, which is brittle (depends on knowing the exact original type) and rarely done in practice.

**What it gives the caller:**
- **Trivial propagation.** Any `Result<_, impl Error>` converts to `anyhow::Result<_>` via `?`. No `From` impls to wire up.
- **Context attachment.** `.context("invalid Ethereum address")` adds a human-readable layer; printing the final error shows the chain: `"invalid Ethereum address: invalid length: expected 40, got 8"`.
- **Backtrace capture.** With `RUST_BACKTRACE=1`, every `anyhow::Error` carries a backtrace, useful for debugging in binaries.
- **Single error type at boundaries.** A CLI's `main() -> anyhow::Result<()>` can absorb every library's error without typing out a union enum.

**When this is OK:**
- Binaries: CLIs, daemons, scripts, integration tests, build scripts.
- Application-level glue where errors are logged-and-exited, never handled programmatically.
- Prototyping — even in library crates while you figure out which errors matter; refactor to `thiserror` before shipping.

**When this is wrong:**
- **Public library APIs.** Forces downstream to handle everything as opaque blobs. Breaks the consumer's ability to recover from specific failures.
- Internal library helpers where the calling site needs type info.
- Performance-critical paths (every error allocates a heap box).

The mental model: **anyhow erases types because the consumer doesn't care about types.** That's correct for binaries; that's wrong for libraries.

## The split, one more time

| Form | Use in | Why |
|---|---|---|
| `panic!` | Test fixtures, internal invariants | Bug, not runtime condition |
| `Result<T, ThiserrorEnum>` | Library APIs | Consumer wants to `match` and handle |
| `anyhow::Result<T>` | Binaries, glue code | Consumer logs-and-exits; types are noise |

A library that uses `anyhow::Error` in its public signature is making a mistake. A binary that defines a custom `MyAppError` thiserror enum just to wrap everything is over-engineering.

## What we keep in `eth-primitives`

The only form that lives in the crate is **`parse_address` returning `Result<Address, PrimitivesError>`** — see `crates/eth-primitives/src/address.rs`. The other two are not exported; they exist only in this comparison.

If you wanted to use the anyhow variant in your future `etherscanlite` CLI binary (Week 5), you'd write a wrapper there:

```rust
// in crates/etherscanlite/src/main.rs (future)
use anyhow::Context;
use eth_primitives::parse_address;

fn cli_parse(s: &str) -> anyhow::Result<Address> {
    parse_address(s).context("invalid Ethereum address in CLI arg")
}
```

That's the right place for `anyhow`: the binary, not the library. The library exposes typed errors; the binary adds context and prints the chain.

