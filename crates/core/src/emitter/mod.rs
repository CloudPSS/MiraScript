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
mod utils;
mod variable;

use emitter_struct::Emitter;
pub use opcode::OpCode;
use opcode::Register;

pub fn emit(script: &Script<'_>, diagnostics_collector: &mut Vec<SourceDiagnostic>) -> Vec<u8> {
    let mut emitter: Emitter<'_> = Emitter::new(diagnostics_collector);
    let args = Some(ParameterList(
        Token::new(TokenKind::Operator(Operator::OpenParen), 0..0).into(),
        vec![],
        Token::new(TokenKind::Operator(Operator::CloseParen), 0..0).into(),
    ));
    emitter.emit_fn_like(
        Register::EMPTY,
        0..script.2.range.end,
        &args,
        &script.0,
        &script.1,
    );
    emitter.chunk.into_bytes()
}
