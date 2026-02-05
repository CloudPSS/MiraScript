use crate::{
    diagnostic::DiagnosticsCollector,
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
mod emitter_pub;
mod emitter_scope;
mod emitter_statement;
mod emitter_struct;
mod opcode;
mod scope;
mod static_checker;
mod utils;
mod variable;

use emitter_struct::Emitter;
pub use opcode::OpCode;
use opcode::Register;

pub fn emit<'s, 'c>(
    script: &Script<'s>,
    diagnostics_collector: &mut DiagnosticsCollector<'s, 'c>,
) -> Vec<u8> {
    let mut emitter = Emitter::new(diagnostics_collector);
    let args = Some(ParameterList(
        Token::new(TokenKind::Operator(Operator::OpenParen), 0..0).into(),
        vec![],
        Token::new(TokenKind::Operator(Operator::CloseParen), 0..0).into(),
    ));
    emitter.emit_fn_like(
        Register::EMPTY,
        0..0,
        0..script.2.range.end,
        &args,
        &script.0,
        &script.1,
    );
    let chunk = emitter.chunk.into_bytes();
    diagnostics_collector.extend(emitter.diagnostics);
    chunk
}
