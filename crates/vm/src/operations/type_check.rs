use super::*;

pub(crate) fn assert_non_nil(value: MiraValue) -> Result<()> {
    if matches!(value, MiraValue::Nil) {
        Err(MiraError::runtime(RuntimeErrorKind::ExpectedNonNil))
    } else {
        Ok(())
    }
}
