# Lifetime Elision

Reference notes on Rust's lifetime elision rules, grounded in real signatures from `eth-primitives::bytes`.

## The three elision rules

The compiler tries these in order. If they don't yield a unique signature, you must annotate explicitly.

1. **Each elided input lifetime gets its own parameter.**
   `fn foo(x: &i32, y: &i32)` → `fn foo<'a, 'b>(x: &'a i32, y: &'b i32)`.

2. **If there's exactly one input lifetime, it's assigned to all elided output lifetimes.**
   `fn foo(x: &i32) -> &i32` → `fn foo<'a>(x: &'a i32) -> &'a i32`.

3. **If `&self` or `&mut self` is among the inputs, `self`'s lifetime wins for all elided outputs** (overrides rule 2).
   `fn foo(&self, x: &str) -> &str` ties the output to `&self`, not to `x`.

## Example 1 — `Bytes::slice` (no lifetimes needed)

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> Self
```

- Inputs: `&self` (one lifetime), `range` (no lifetimes — `RangeBounds<usize>` doesn't borrow).
- Output: `Self` — owned `Bytes`, no lifetime in the output.
- **No elision happens.** The output isn't a borrow.

**Lesson**: lifetimes only matter when references appear in the output. Owned return types short-circuit the question entirely. That's why `Bytes::slice` allocates a fresh `Arc<[u8]>` rather than handing out a borrow — the API doesn't need lifetime annotations because there's no borrow to track.

## Example 2 — `BytesView::slice` (elision succeeds but is wrong)

Wrong:

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> BytesView<'_>
```

Right:

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> BytesView<'a>
```

The bodies are identical. Only the return type changes.

What rule 3 does (the wrong version):

- Sees `&self` among inputs.
- Assigns `&self`'s lifetime to the elided `'_` in `BytesView<'_>`.
- Result: the returned view borrows from `self` — the temporary `&BytesView<'a>`, not the underlying source.

What the explicit `'a` does (the right version):

- Pins the output to the type's own lifetime parameter `'a`.
- Result: the returned view borrows from the same source as the original `BytesView`, regardless of how short-lived the intermediate `&self` is.

**Lesson**: rule 3 is a convenience for *getter* methods (`fn name(&self) -> &str` — output really does live as long as `&self`). It's the wrong default for *sub-view* methods, where the returned borrow should outlive the temporary holder. When the type already carries a lifetime parameter, prefer it explicitly over elision.

The failure mode is subtle: the wrong version compiles. Downstream callers only hit a confusing borrow-checker error two layers up, when they try to use a sub-view past the lifetime of the intermediate view.

## Example 3 — `BytesView::split_at` (multi-output exposes elision's limits)

```rust
pub fn split_at(&self, mid: usize) -> (BytesView<'a>, BytesView<'a>)
```

- Two outputs, both borrowed from the same source.
- Naive elision (`(BytesView<'_>, BytesView<'_>)`) would tie both halves to `&self` via rule 3 — useless: both halves invalidate the moment `self` drops.
- Explicit `'a` on both ties them to the underlying source.

**Lesson**: elision can't express "these multiple outputs share a source." When a function returns multiple borrows, you almost always have to annotate explicitly. Even when elision technically picks a default, it's usually rule 3 — and rule 3 is wrong for the same reason as in example 2.

## Example 4 — when elision fails outright

```rust
fn longest(x: &str, y: &str) -> &str  // ERROR: ambiguous output lifetime
```

- Rule 1: `x: &'a str, y: &'b str` — two separate input lifetimes.
- Rule 2: doesn't apply (more than one input lifetime).
- Rule 3: doesn't apply (no `&self`).
- Compiler refuses to guess.

Fix:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str
```

**Lesson**: this case is the easy one — the compiler tells you to annotate. The dangerous case is example 2, where elision *succeeds* but with the wrong default.

## Mental checklist when writing a borrow-returning function

1. Does the output contain a reference or a type with a lifetime parameter?
   - No → no annotation needed.
2. Is there exactly one input lifetime?
   - Yes → elision picks it (rule 2). Usually correct.
3. Is `&self` an input?
   - Yes → elision ties output to `&self` (rule 3). **Stop and think**: does the returned borrow really live only as long as `&self`, or does it live as long as some lifetime parameter on `self`'s type? If the latter, annotate explicitly.
4. Multiple outputs that share a source?
   - Annotate explicitly. Elision can't express it.

## Why this matters in Reth

These lifetime patterns recur throughout Reth's borrow-heavy code:

- **Trie node iteration**: child views into a parent node need to outlive the iterator.
- **EVM stack/memory access**: borrowed slots returned to interpreter handlers must live as long as the call frame, not the temporary `&mut interpreter` borrow.
- **Database cursors**: cursor reads return borrows tied to the cursor's transaction, not the cursor itself.

Each of these is a real-world version of the `BytesView::slice` lesson. Get the elision rules wrong and the API forces unnecessary copies; get them right and the borrow checker validates zero-copy access for free.
