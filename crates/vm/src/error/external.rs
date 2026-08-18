use super::MiraError;

impl Into<Box<MiraError>> for anyhow::Error {
    fn into(self) -> Box<MiraError> {
        let err = match self.downcast::<MiraError>() {
            Ok(error) => return Box::new(error),
            Err(error) => error,
        };
        let err = match err.downcast::<Box<MiraError>>() {
            Ok(error) => return error,
            Err(error) => error,
        };
        Box::new(MiraError::External(err))
    }
}
