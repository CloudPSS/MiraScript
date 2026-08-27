use crate::{MiraError, MiraManageable, MiraValue, Result, Runtime, TryFromMira};

/// Convert one fixed native-function argument.
pub fn native_argument<'a, T>(runtime: &'a Runtime, value: MiraValue) -> Result<T>
where
    T: TryFromMira<'a>,
{
    T::from_mira(runtime, value)
}

/// Convert one optional native-function argument.
pub fn native_argument_optional<'a, T>(
    runtime: &'a Runtime,
    value: Option<MiraValue>,
) -> Result<Option<T>>
where
    T: TryFromMira<'a>,
{
    if let Some(value) = value {
        Ok(Option::<T>::from_mira(runtime, value)?)
    } else {
        Ok(None)
    }
}

/// Convert a fallible native-function return value.
pub fn native_result<T, E>(result: std::result::Result<T, E>) -> Result<MiraManageable>
where
    T: Into<MiraManageable>,
    E: Into<anyhow::Error>,
{
    result
        .map(Into::into)
        .map_err(|error| Box::<MiraError>::from(error.into()))
}
