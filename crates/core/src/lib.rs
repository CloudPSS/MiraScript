#![doc = include_str!("../README.md")]

mod compile;
mod config;
mod diagnostic;
mod emitter;
mod lexer;
mod parser;

pub use compile::{CompileResult, Compiler};
pub use config::{CompileConfig, DiagnosticPositionEncoding, InputMode};
pub use diagnostic::{
    DiagnosticCode, SerializedDiagnostics, SourceDiagnostic, SourceRange, encode_diagnostics,
};
pub use emitter::OpCode;
pub use lexer::{Keyword, Operator, Token, TokenKind};
pub use parser::Script;

#[cfg(feature = "formatter")]
pub mod formatter;
#[cfg(feature = "formatter")]
pub use formatter::format;

pub mod prelude {
    pub use std::str::FromStr as _;
    pub use std::string::ToString as _;
    pub use strum::VariantArray as _;
}

#[cfg(all(feature = "mimalloc", not(target_family = "wasm")))]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;
