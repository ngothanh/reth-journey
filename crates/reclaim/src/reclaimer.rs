use crate::Guard;
use crate::Retirable;

/// A scheme: the one that decides when a retired object can be freed.
///
/// Each scheme names its own guard type.
///
/// # Safety
///
/// Whoever implements this trait promises:
///
/// - the scheme never frees an object while a guard still protects it;
/// - the scheme frees an object only by calling that object's own
///   `Retirable::reclaim`, and at most once.
///
/// Readers rely on this. A scheme that frees too early makes a reader open
/// freed memory, even though the reader and the structure did everything right.
pub unsafe trait Reclaimer {
    /// What a reader holds while it is reading.
    type Guard: Guard;

    /// Gives a reader something to read with.
    fn guard(&self) -> Self::Guard;

    /// Hands an object to the scheme: "I am done with it."
    ///
    /// The object is not freed here. The scheme frees it later, when no
    /// reader can still hold it.
    ///
    /// # Safety
    ///
    /// The caller promises:
    ///
    /// - no new reader can get this object's address from any root;
    /// - `obj` is handed to `retire` only once;
    /// - the caller does not use `obj` after this call.
    unsafe fn retire<T: Retirable>(&self, obj: *mut T);
}
