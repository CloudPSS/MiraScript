use super::*;

pub(crate) fn assert_initialized(value: &MiraAny) -> Result<()> {
    if matches!(value, MiraAny::Uninitialized) {
        Err(MiraError::runtime("Uninitialized value"))
    } else {
        Ok(())
    }
}

pub(crate) fn assert_non_nil(value: &MiraAny) -> Result<()> {
    assert_initialized(value)?;
    if matches!(value, MiraAny::Nil) {
        Err(MiraError::runtime("Expected non-nil value"))
    } else {
        Ok(())
    }
}
