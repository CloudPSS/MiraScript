use crate::{MiraError, MiraType, Result};

use super::{MiraValue, value::RawValue, value::ValueTag};

impl MiraValue {
    /// Create a `MiraValue` representing a boolean value.
    #[inline]
    pub const fn boolean(value: bool) -> Self {
        Self::from_raw(RawValue::tagged(
            ValueTag::Boolean,
            [value as u8, 0, 0, 0, 0, 0],
        ))
    }

    /// Check whether this value is a boolean value.
    #[inline]
    pub const fn is_boolean(&self) -> bool {
        matches!(self.value_type(), MiraType::Boolean)
    }

    /// Return the inline boolean payload, or `None` for another value type.
    #[inline]
    pub const fn as_boolean(&self) -> Option<bool> {
        if !self.is_boolean() {
            None
        } else {
            Some(self.raw().payload()[0] != 0)
        }
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

        let value = MiraValue::nil();
        assert!(!value.is_boolean());
        assert_eq!(value.as_boolean(), None);
    }

    #[test]
    fn test_try_from() {
        let value = MiraValue::boolean(true);
        let boolean: bool = value.try_into().unwrap();
        assert!(boolean);

        let value = MiraValue::nil();
        assert!(bool::try_from(value).is_err());
    }
}
