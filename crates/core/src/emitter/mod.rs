use crate::{
    diagnostic::SourceDiagnostic,
    lexer::{Operator, Token, TokenKind},
    parser::{ParameterList, Script},
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

pub fn emit(source: &str, script: &Script<'_>) -> (Vec<SourceDiagnostic>, Box<[u8]>) {
    let mut emitter: Emitter<'_> = Emitter::new(source);
    let open = Token::new(TokenKind::Operator(Operator::OpenBrace), 0..0);
    let close = Token::new(TokenKind::Operator(Operator::CloseBrace), 0..0);
    let args = Some(ParameterList(open.into(), vec![], close.into()));
    emitter.emit_fn(
        Register::EMPTY,
        0..0,
        &args,
        0..source.len(),
        &script.0,
        &script.1,
    );
    (emitter.diagnostics, emitter.chunk.to_bytes())
}
