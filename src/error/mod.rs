use std::{error::Error, fmt::Display};

mod error_code;

pub use error_code::ErrorCode;
pub type SourceRange = std::ops::Range<usize>;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceError {
    pub range: SourceRange,
    pub error: ErrorCode,
}

impl SourceError {
    pub fn new(range: SourceRange, error: ErrorCode) -> Self {
        SourceError { range, error }
    }
}

impl Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error at {}:{}: {}",
            self.range.start, self.range.end, self.error
        )
    }
}

impl Error for SourceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
