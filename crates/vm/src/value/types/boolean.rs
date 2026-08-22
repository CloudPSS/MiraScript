use crate::{MiraError, Result, value::arena::MiraManageable};

use super::MiraValue;

impl MiraValue {
    /// Create a `MiraValue` representing a boolean value.
    #[inline]
    pub fn boolean(value: bool) -> Self {
        Self::boxed_boolean(value)
    }

    /// Check whether this value is a boolean value.
    #[inline]
    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }

    /// Return the inline boolean payload, or `None` for another value type.
    #[inline]
    pub fn as_boolean(&self) -> Option<bool> {
        self.boxed_boolean_value()
    }
}

impl From<bool> for MiraValue {
    #[inline]
    fn from(value: bool) -> Self {
        Self::boolean(value)
    }
}

impl From<bool> for MiraManageable {
    #[inline]
    fn from(value: bool) -> Self {
        std::convert::Into::<MiraValue>::into(value).into()
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
