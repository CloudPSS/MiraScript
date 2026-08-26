mod array;
mod primitives;
mod shaped_array;
mod shaped_record;

use crate::{MiraArray, MiraHandle, MiraManageable, MiraRecord};

pub use array::{array_from_array, array_from_record};
pub use shaped_array::{shaped_array_from_array, shaped_array_from_record};
pub use shaped_record::{shaped_record_from_array, shaped_record_from_record};

/// Hidden routing contract used by the derive macros for projected fields.
#[doc(hidden)]
pub type MiraFieldGetter<P, T> = fn(&P, usize) -> &T;

/// Hidden routing contract used by the derive macros for projected fields.
///
/// Implementations decide whether a field is copied inline or exposed as a
/// live array/record projection backed by its parent's typed handle.
#[doc(hidden)]
pub trait MiraField: Into<MiraManageable> + Sized + 'static {
    /// Project a field from a record parent.
    #[allow(clippy::wrong_self_convention)]
    fn from_record<P: MiraRecord>(
        &self,
        parent: MiraHandle<P>,
        index: usize,
        getter: MiraFieldGetter<P, Self>,
    ) -> MiraManageable;

    /// Project a field from an array parent.
    #[allow(clippy::wrong_self_convention)]
    fn from_array<P: MiraArray>(
        &self,
        parent: MiraHandle<P>,
        index: usize,
        getter: MiraFieldGetter<P, Self>,
    ) -> MiraManageable;
}
