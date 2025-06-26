pub mod ansi;
pub mod compile;
pub mod config;
pub mod diagnostic;
pub mod emitter;
pub mod formatter;
pub mod lexer;
pub mod parser;

pub use compile::{Compiler, SerializedDiagnostics};
pub use config::Config;
pub use diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange};
pub use emitter::OpCode;
pub use formatter::{format, format_statement};
pub use lexer::{Keyword, Operator};
pub use parser::{Expression, Pattern, Script, Statement};

pub mod prelude {
    pub use std::str::FromStr as _;
    pub use std::string::ToString as _;
    pub use strum::VariantArray as _;
}
