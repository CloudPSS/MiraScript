use crate::{Result, Runtime};

use super::MiraValue;

/// A trait for converting values from the runtime into Rust types.
pub trait TryFromMira<'a>: Sized + 'a {
    /// Convert a [`MiraValue`] into a Rust type, returning an error if the conversion fails.
    fn from_mira(runtime: &'a Runtime, value: MiraValue) -> Result<Self>;
}

impl<'a, T> TryFromMira<'a> for Option<T>
where
    T: TryFromMira<'a>,
{
    fn from_mira(runtime: &'a Runtime, value: MiraValue) -> Result<Self> {
        if value.is_nil() {
            Ok(None)
        } else {
            Ok(Some(T::from_mira(runtime, value)?))
        }
    }
}

impl TryFromMira<'_> for MiraValue {
    fn from_mira(_runtime: &Runtime, value: MiraValue) -> Result<Self> {
        Ok(value)
    }
}
