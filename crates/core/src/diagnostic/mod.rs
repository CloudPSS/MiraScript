mod diagnostic_code;
mod diagnostics_collector;
mod encode_diagnostics;
mod source_diagnostic;

pub use diagnostic_code::DiagnosticCode;
pub use diagnostics_collector::DiagnosticsCollector;
pub use encode_diagnostics::{SerializedDiagnostics, encode_diagnostics};
pub use source_diagnostic::SourceDiagnostic;
pub type SourceRange = std::ops::Range<usize>;
