use super::{MiraValue, value::RawValue};
use crate::{MiraError, Result, Runtime, TryFromMira};

impl MiraValue {
    /// Create a [`MiraValue`] representing a numeric value.
    #[inline]
    pub const fn number(value: f64) -> Self {
        Self::from_raw(RawValue::from_number(value))
    }

    /// A [`MiraValue`] representing the numeric value `0.0`.
    pub const ZERO: MiraValue = MiraValue::number(0.0);
    /// A [`MiraValue`] representing the numeric value `1.0`.
    pub const ONE: MiraValue = MiraValue::number(1.0);
    /// A [`MiraValue`] representing the numeric value `-1.0`.
    pub const NEGATIVE_ONE: MiraValue = MiraValue::number(-1.0);
    /// A [`MiraValue`] representing the numeric value `f64::NAN`.
    pub const NAN: MiraValue = MiraValue::number(f64::NAN);

    /// Check whether this value is a numeric value.
    #[inline]
    pub const fn is_number(&self) -> bool {
        self.tag().is_none()
    }

    /// Return the inline numeric payload, or [`None`] for another value type.
    #[inline]
    pub const fn as_number(&self) -> Option<f64> {
        if self.is_number() {
            Some(self.as_number_unchecked())
        } else {
            None
        }
    }
    /// Return the inline numeric payload.
    #[inline]
    pub const fn as_number_unchecked(self) -> f64 {
        debug_assert!(self.is_number(), "MiraValue is not a number");
        self.raw().number()
    }
}

macro_rules! number_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl From<$ty> for MiraValue {
            fn from(value: $ty) -> Self {
                Self::number(value as f64)
            }
        }
    )* };
}

number_from!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

macro_rules! integer_try_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl TryFrom<MiraValue> for $ty {
            type Error = Box<MiraError>;

            fn try_from(value: MiraValue) -> Result<Self> {
                let Some(number) = value.as_number() else {
                    return Err(MiraError::conversion_type(stringify!($ty), value.value_type()));
                };
                if !number.is_finite()
                    || number.trunc() != number
                    || number < (<$ty>::MIN as f64)
                    || number > (<$ty>::MAX as f64)
                {
                    return Err(MiraError::conversion_number(stringify!($ty), number));
                }
                Ok(number as $ty)
            }
        }

        impl TryFromMira<'_> for $ty {
            fn from_mira(_runtime: &Runtime, value: MiraValue) -> Result<Self> {
                Self::try_from(value)
            }
        }
    )* };
}
integer_try_from!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl TryFrom<MiraValue> for f64 {
    type Error = Box<MiraError>;

    fn try_from(value: MiraValue) -> Result<Self> {
        value
            .as_number()
            .ok_or_else(|| MiraError::conversion_type("f64", value.value_type()))
    }
}

impl TryFromMira<'_> for f64 {
    fn from_mira(_runtime: &Runtime, value: MiraValue) -> Result<Self> {
        f64::try_from(value)
    }
}

impl TryFrom<MiraValue> for f32 {
    type Error = Box<MiraError>;

    fn try_from(value: MiraValue) -> Result<Self> {
        let value = f64::try_from(value)?;
        if value.is_finite() && (value < f32::MIN as f64 || value > f32::MAX as f64) {
            return Err(MiraError::conversion_number("f32", value));
        }
        Ok(value as f32)
    }
}

impl TryFromMira<'_> for f32 {
    fn from_mira(_runtime: &Runtime, value: MiraValue) -> Result<Self> {
        f32::try_from(value)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_number() {
        assert_eq!(MiraValue::number(42.0), MiraValue::number(42.0));
    }

    #[test]
    fn test_number_is_number() {
        assert!(MiraValue::number(42.0).is_number());
        assert!(!MiraValue::NIL.is_number());
    }

    #[test]
    fn test_number_as_number() {
        assert_eq!(MiraValue::number(42.0).as_number(), Some(42.0));
        assert_eq!(MiraValue::NIL.as_number(), None);
    }

    #[test]
    fn test_number_try_from() {
        let value = MiraValue::number(42.0);
        assert_eq!(u8::try_from(value).unwrap(), 42u8);
        assert_eq!(i32::try_from(value).unwrap(), 42i32);
        assert_eq!(u128::try_from(value).unwrap(), 42u128);
        assert_eq!(f32::try_from(value).unwrap(), 42f32);
        assert_eq!(f64::try_from(value).unwrap(), 42f64);
    }
    #[test]
    fn test_number_try_from_underflow() {
        let value = MiraValue::number(-1.0);
        assert!(u8::try_from(value).is_err());
        assert!(u32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_overflow() {
        let value = MiraValue::number(256.0);

        assert!(dbg!(u8::try_from(value)).is_err());
        let value = MiraValue::number(u32::MAX as f64 * 1.1);
        assert!(dbg!(u32::try_from(value)).is_err());
        let value = MiraValue::number(u128::MAX as f64 * 1.1);
        assert!(dbg!(u128::try_from(value)).is_err());
        let value = MiraValue::number(f32::MAX as f64 * 1.1);
        assert!(f32::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_non_integer() {
        let value = MiraValue::number(42.5);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_non_finite() {
        let value = MiraValue::number(f64::INFINITY);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
        let value = MiraValue::number(f64::NAN);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_non_number() {
        assert!(f64::try_from(MiraValue::NIL).is_err());

        let value = MiraValue::boolean(true);
        assert!(u8::try_from(value).is_err());
        assert!(i32::try_from(value).is_err());
        assert!(u128::try_from(value).is_err());

        let value = MiraValue::str(&"42");
        assert!(u8::try_from(value).is_err());
        assert!(f64::try_from(value).is_err());
    }

    #[test]
    fn test_number_try_from_negative_zero() {
        let value = MiraValue::number(-0.0);
        assert_eq!(u8::try_from(value).unwrap(), 0u8);
        assert_eq!(i32::try_from(value).unwrap(), 0i32);
        assert_eq!(u128::try_from(value).unwrap(), 0u128);
        assert_eq!(f32::try_from(value).unwrap(), -0.0f32);
        assert_eq!(f64::try_from(value).unwrap(), -0.0f64);
    }

    #[test]
    fn test_number_into() {
        assert_eq!(MiraValue::from(42u8), MiraValue::number(42.0));
        assert_eq!(MiraValue::from(42i32), MiraValue::number(42.0));
        assert_eq!(MiraValue::from(42u128), MiraValue::number(42.0));
        assert_eq!(MiraValue::from(42f32), MiraValue::number(42.0));
        assert_eq!(MiraValue::from(42f64), MiraValue::number(42.0));
    }
}
