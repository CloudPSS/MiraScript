use std::{error::Error, fmt::Display, ops::Deref};

mod diagnostic_code;

pub use diagnostic_code::DiagnosticCode;
pub type SourceRange = std::ops::Range<usize>;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceDiagnostic {
    pub range: SourceRange,
    pub error: DiagnosticCode,
}

impl SourceDiagnostic {
    pub fn new(range: SourceRange, error: DiagnosticCode) -> Self {
        SourceDiagnostic { range, error }
    }
}

impl Display for SourceDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "at {}:{}: {}",
            self.range.start, self.range.end, self.error
        )
    }
}

impl Error for SourceDiagnostic {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Deref for SourceDiagnostic {
    fn deref(&self) -> &DiagnosticCode {
        &self.error
    }

    type Target = DiagnosticCode;
}
