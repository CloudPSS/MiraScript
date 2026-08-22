use super::*;

pub(crate) fn assert_non_nil(value: MiraValue) -> Result<()> {
    if value.is_nil() {
        Err(MiraError::runtime(RuntimeErrorKind::ExpectedNonNil))
    } else {
        Ok(())
    }
}
