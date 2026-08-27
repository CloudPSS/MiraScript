use crate::{MiraError, MiraType, Result, Runtime, TryFromMira};

use super::{MiraValue, value::RawValue, value::ValueTag};

impl MiraValue {
    /// Create a `MiraValue` representing a boolean value.
    #[inline]
    pub const fn boolean(value: bool) -> Self {
        Self::from_raw(RawValue::from_tagged(
            ValueTag::Boolean,
            [value as u8, 0, 0, 0, 0, 0],
        ))
    }

    /// A `MiraValue` representing the boolean value `true`.
    pub const TRUE: MiraValue = MiraValue::boolean(true);
    /// A `MiraValue` representing the boolean value `false`.
    pub const FALSE: MiraValue = MiraValue::boolean(false);

    /// Check whether this value is a boolean value.
    #[inline]
    pub const fn is_boolean(&self) -> bool {
        matches!(self.value_type(), MiraType::Boolean)
    }

    /// Return the inline boolean payload, or `None` for another value type.
    #[inline]
    pub const fn as_boolean(&self) -> Option<bool> {
        if self.is_boolean() {
            Some(self.as_boolean_unchecked())
        } else {
            None
        }
    }

    /// Return the inline boolean payload.
    #[inline]
    pub const fn as_boolean_unchecked(self) -> bool {
        debug_assert!(self.is_boolean(), "MiraValue is not a boolean");
        self.raw().payload()[0] != 0
    }
}

impl From<bool> for MiraValue {
    #[inline]
    fn from(value: bool) -> Self {
        Self::boolean(value)
    }
}

impl TryFrom<MiraValue> for bool {
    type Error = Box<MiraError>;

    #[inline]
    fn try_from(value: MiraValue) -> Result<Self> {
        value
            .as_boolean()
            .ok_or_else(|| MiraError::conversion_type("bool", value.value_type()))
    }
}

impl TryFromMira<'_> for bool {
    fn from_mira(_runtime: &Runtime, value: MiraValue) -> Result<Self> {
        bool::try_from(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean() {
        let value = MiraValue::boolean(true);
        assert!(value.is_boolean());
        assert_eq!(value.as_boolean(), Some(true));

        let value = MiraValue::boolean(false);
        assert!(value.is_boolean());
        assert_eq!(value.as_boolean(), Some(false));

        let value = MiraValue::NIL;
        assert!(!value.is_boolean());
        assert_eq!(value.as_boolean(), None);
    }

    #[test]
    fn test_try_from() {
        let value = MiraValue::boolean(true);
        let boolean: bool = value.try_into().unwrap();
        assert!(boolean);

        let value = MiraValue::NIL;
        assert!(bool::try_from(value).is_err());
        assert!(bool::from_mira(&Runtime::new(), value).is_err());
    }
}
