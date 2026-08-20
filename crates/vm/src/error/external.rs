use super::MiraError;

impl From<anyhow::Error> for Box<MiraError> {
    fn from(error: anyhow::Error) -> Self {
        let error = match error.downcast::<MiraError>() {
            Ok(error) => return Box::new(error),
            Err(error) => error,
        };
        match error.downcast::<Box<MiraError>>() {
            Ok(error) => error,
            Err(error) => Box::new(MiraError::External(error)),
        }
    }
}
