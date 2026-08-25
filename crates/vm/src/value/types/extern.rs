use crate::{MiraHandle, value::types::value::ValueTag};

use super::MiraValue;

impl MiraValue {
    /// Create a `MiraValue` representing an external value.
    #[doc(hidden)]
    #[inline]
    pub fn r#extern<T: MiraExtern + ?Sized>(_: MiraHandle<T>) -> Self {
        unimplemented!("External values are not implemented in this release.");
    }

    /// Check whether this value is an external value.
    #[doc(hidden)]
    #[inline]
    pub const fn is_extern(&self) -> bool {
        matches!(self.tag(), Some(ValueTag::Extern))
    }
}

mod private {
    pub trait Sealed {}
}

/// Reserved marker for a future MiraScript external value implementation.
///
/// The trait is sealed intentionally: external values are not implemented by
/// this release and downstream crates cannot implement this marker.
pub trait MiraExtern: std::any::Any + private::Sealed + 'static {}
