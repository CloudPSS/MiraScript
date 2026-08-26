use super::{MiraValue, value::ValueTag};

impl MiraValue {
    /// A [`MiraValue`] representing an uninitialized value.
    /// This is used internally by the VM to represent uninitialized registers.
    /// It should never be exposed to user code, and will panic if it does.
    pub(crate) const UNINITIALIZED: MiraValue = Self::empty(ValueTag::Uninitialized);

    /// Return whether this value is uninitialized.
    /// This is used internally by the VM to check for uninitialized registers.
    #[inline]
    pub(crate) const fn is_uninitialized(&self) -> bool {
        matches!(self.tag(), Some(ValueTag::Uninitialized))
    }
}
