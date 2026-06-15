/// Plain old data: a fixed-size value that is just bytes, with no padding and
/// no invalid bit patterns, so it can be freely reinterpreted to/from `u64`.
///
/// # Safety
///
/// Implementing `Pod` is a promise that the type satisfies ALL of:
///
/// 1. **No padding / no uninitialized bytes** — every one of the
///    `size_of::<Self>()` bytes belongs to a field. Required so reading the
///    value *as* raw bytes never touches uninit memory (the `store` direction).
/// 2. **Every bit pattern is valid** — any `size_of::<Self>()` bytes form a
///    valid `Self`. Required so *reconstructing* the value from the atomic's
///    bytes is never UB (the `load` direction). This rules out `bool`, `char`,
///    field-less enums, `NonZero*`, and references.
/// 3. A defined layout (`#[repr(C)]`/`#[repr(transparent)]`) — `repr(Rust)`
///    leaves padding unspecified, so you can't reason about (1).
///
/// The compiler checks none of this; that is why the trait is `unsafe`.
pub unsafe trait Pod: Copy {}

unsafe impl Pod for u8 {}
unsafe impl Pod for i8 {}
unsafe impl Pod for isize {}
unsafe impl Pod for u64 {}
unsafe impl Pod for u32 {}
unsafe impl Pod for i64 {}
unsafe impl Pod for f64 {}
unsafe impl Pod for f32 {}
unsafe impl Pod for usize {}

unsafe impl<T> Pod for *mut T {}

unsafe impl<T> Pod for *const T {}

unsafe impl<T: Pod, const N: usize> Pod for [T; N] {}
