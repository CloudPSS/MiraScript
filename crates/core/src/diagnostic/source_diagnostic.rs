use std::{error::Error, fmt::Display, ops::Deref};

use super::{DiagnosticCode, SourceRange};

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
            "{}({}) at {}~{}: {}",
            self.error,
            self.code(),
            self.range.start,
            self.range.end,
            self.error.message()
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
