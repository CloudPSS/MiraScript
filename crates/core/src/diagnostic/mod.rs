mod diagnostic;
mod diagnostic_code;
mod diagnostics_collector;
mod encode_diagnostics;

pub use diagnostic::SourceDiagnostic;
pub use diagnostic_code::DiagnosticCode;
pub use diagnostics_collector::DiagnosticsCollector;
pub use encode_diagnostics::{SerializedDiagnostics, encode_diagnostics};
pub type SourceRange = std::ops::Range<usize>;
