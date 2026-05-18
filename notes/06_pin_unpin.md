# Pin / Unpin: When and Why

Reference notes from W3 Wed building `MessageStream` two ways — once with
`pin_project_lite` (`stream.rs`), once with manual
`unsafe { get_unchecked_mut() }` (`stream_manual.rs`).

## What Pin guarantees

`Pin<&mut T>` is a *caller-side promise*: "I will not move T to a new memory
address." Required by `Future::poll`, `Stream::poll_next`,
`AsyncRead::poll_read`, etc., because:

- An `async fn` compiles to a self-referential state machine — its fields
  can hold references into its own locals across `.await` points.
- Moving such a state machine would invalidate the internal references → UB.
- Pin is the type-system encoding of "you cannot move this without `unsafe`."

## The Unpin opt-out

`Unpin` is an **auto-trait**. A type is `Unpin` iff every field is `Unpin`.
For Unpin types, `Pin::get_mut` is **safe** — pinning is dormant; `Pin<&mut T>`
elides to `&mut T` at zero cost.

Most things are Unpin: primitives, `String`, `Vec`, `HashMap`, `TcpStream`,
`tokio::fs::File`. The exceptional types are mostly compiler-generated async
state machines, intrusive collections, and anything holding raw self-pointers.

`PhantomPinned` is a zero-sized marker field that is itself `!Unpin`. Adding
it to a struct suppresses the auto-impl. Use it when you want to *enforce*
pin discipline regardless of generic parameters' Unpin status — see
"Soundness trap" below.

## Pin projection — the core ergonomic challenge

To call `inner.poll(cx)` on an inner future field, you need `Pin<&mut F>` —
but you only have `Pin<&mut Self>`. Bridging the two is "pin projection."

| Approach | When to reach for it | Tradeoff |
|---|---|---|
| `pin_project_lite!` macro + `#[pin]` field | default for any struct with a Future/Stream field | zero unsafe; compile-time invariant checks; survives future refactors |
| `Pin::new(&mut self.inner)` + `F: Unpin` bound | inner type known to be Unpin | restricts callers; loses generality |
| Manual `unsafe { self.get_unchecked_mut() }` + safety comment | `no_std` crates that can't pull `pin-project-lite`; learning exercises | four invariants to prove by hand; silent UB on future refactor |

For 99% of cases: the macro.

## The four-point safety invariant (manual projection)

When you write `unsafe { self.get_unchecked_mut() }` to project, the safety
comment must justify all four:

1. **Pinned fields are never moved.** Only `Pin<&mut Field>` is exposed —
   never `&mut Field` — for any field treated as pinned. No `mem::swap`,
   `mem::replace`, or direct assignment.
2. **No `Drop` impl that moves pinned fields.** If you write `Drop`, it must
   not relocate any `#[pin]` field out of `Self`. Better: don't write `Drop`.
3. **Unpinned fields can be exposed as `&mut`.** They have no pin obligation;
   `&mut` access is sound.
4. **No other public API exposes pinned fields outside `Pin<&mut _>`.** A
   single escape hatch (e.g. `fn take_inner(&mut self) -> F`) defeats the
   whole guarantee.

If any of these can be violated by a future code change, the unsafe block
becomes UB with no compiler warning. `pin_project_lite!` checks them at
compile time and rejects violating code.

## Soundness trap — `PhantomPinned` and the auto-impl

If you write a manual projection but **omit `PhantomPinned`**, and all your
fields happen to be `Unpin`, the auto-Unpin impl fires. Then `Pin::get_mut`
is safe — callers can bypass your `project()` method entirely with
`Pin::get_mut(pinned) → &mut Self → &mut self.io → mem::swap(...)`.

Concrete example from this week's `MessageStreamManual<IO, Codec>`:

- Instantiated as `<TcpStream, LengthDelimitedCodec>`: both Unpin → struct is
  auto-Unpin → the `unsafe` inside `project()` is **ceremonial**. Test passes;
  no observable bug.
- Instantiated as `<SomeAsyncReadStateMachine, _>` where the IO is `!Unpin`:
  the explicit `impl Unpin where IO: Unpin` would contradict the auto-derive.
  One of the two paths becomes UB.

**Rule**: whenever you write a manual `unsafe` projection, add
`PhantomPinned`. Zero runtime cost (ZST); restores the soundness invariant
by making the struct unconditionally `!Unpin` and forcing your explicit
`impl<...> Unpin for Self where ...` to be the sole authority.

## Decision tree for real code

```
Am I writing this code?
├── async fn / let _ = future.await           → never think about Pin
├── stream.next() / future.map() / etc.       → never think about Pin
├── Storing a Future in a struct field        → Box::pin OR pin_project! #[pin]
├── Implementing Future/Stream/Sink by hand   → think about Pin
│   ├── Inner future/stream field?            → pin_project_lite with #[pin]
│   ├── Self-referential type?                → PhantomPinned + manual unsafe project
│   └── No async fields, just signature?      → accept Pin<&mut Self>; no projection
└── Self-referential data structure (rare)    → Pin throughout; full discipline
```

Most async code is in the top two branches — Pin is invisible.

## References (lookup, not read end-to-end)

- `std::pin` module docs — best single source of truth
- `pin-project-lite` README — refresher when you forget which field needs `#[pin]`
- Async Book Ch4 ("Pinning to the stack" section) — skim only
- without.boats "Pin" blog post — the design rationale, when curious about the why
