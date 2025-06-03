use crate::{error::SourceError, parser::Script};

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

pub fn emit(script: &Script<'_>) -> (Vec<SourceError>, Box<[u8]>) {
    let mut emitter: Emitter<'_> = Emitter::new();
    let args = Some(vec![]);
    emitter.emit_fn(Register::EMPTY, &args, &script.0, &script.1);
    (emitter.errors, emitter.chunk.to_bytes())
}
