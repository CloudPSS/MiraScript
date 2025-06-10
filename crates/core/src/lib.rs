pub mod ansi;
pub mod compile;
pub mod config;
pub mod diagnostic;
pub mod emitter;
pub mod lexer;
pub mod parser;

pub use compile::{compile_script, compile_template};
pub use config::Config;
pub use diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange};
pub use emitter::OpCode;
pub use lexer::{Keyword, Operator};

pub mod prelude {
    pub use std::str::FromStr as _;
    pub use std::string::ToString as _;
    pub use strum::VariantArray as _;
}
