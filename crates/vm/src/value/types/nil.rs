use crate::value::arena::MiraManageable;

use super::MiraValue;

impl MiraValue {
    /// Return a `MiraValue` representing the MiraScript `nil` value.
    #[inline]
    pub fn nil() -> Self {
        Self::boxed_nil()
    }

    /// Check whether this value is the MiraScript `nil` value.
    #[inline]
    pub fn is_nil(&self) -> bool {
        self.boxed_is_nil()
    }
}

impl<T: Into<MiraValue>> From<Option<T>> for MiraValue {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => MiraValue::nil(),
        }
    }
}

impl<T: Into<MiraManageable>> From<Option<T>> for MiraManageable {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => MiraValue::nil().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nil() {
        assert_eq!(MiraValue::nil(), MiraValue::nil());
        assert!(MiraValue::nil().is_nil());
        assert!(!MiraValue::number(42.0).is_nil());
    }

    #[test]
    fn test_option_into_value() {
        let value: MiraValue = Some(42).into();
        assert_eq!(value, MiraValue::number(42.0));

        let value: MiraValue = None::<i32>.into();
        assert_eq!(value, MiraValue::nil());
    }
}
