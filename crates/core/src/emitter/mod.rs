use crate::{
    diagnostic::{SourceDiagnostic, SourceRange},
    lexer::Token,
    parser::Script,
};

mod chunk;
mod closure;
mod constant;
mod emitter_closure;
mod emitter_codes;
mod emitter_expression;
mod emitter_pattern;
mod emitter_scope;
mod emitter_statement;
mod emitter_struct;
mod opcode;
mod scope;
mod variable;

use emitter_struct::Emitter;
pub use opcode::OpCode;
use opcode::Register;

pub fn emit(script: &Script<'_>) -> (Vec<SourceDiagnostic>, Box<[u8]>) {
    let mut emitter: Emitter<'_> = Emitter::new();
    let args = Some(vec![]);
    let token = Token {
        kind: crate::lexer::TokenKind::Unknown {
            recovered: None,
            errors: vec![],
        },
        range: SourceRange::default(),
        leading_trivia: vec![],
        trailing_trivia: vec![],
    };
    emitter.emit_fn(Register::EMPTY, &token, &args, &script.0, &script.1);
    (emitter.diagnostics, emitter.chunk.to_bytes())
}
