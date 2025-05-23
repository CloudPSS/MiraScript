use crate::{
    error::{ErrorCode, SourceError},
    lexer::TokenKind,
    parser::Pattern::{self, *},
};

use super::{
    Emitter, OpCode,
    opcode::Register,
    variable::{self, BindType, Variable},
};

impl<'s> Emitter<'s> {
    pub fn declare_pattern(&mut self, pattern: &'s Pattern<'s>, bind_type: Option<BindType>) {
        match pattern {
            Grouping(token, pattern, token1) => todo!(),
            Constant(token, token1) => todo!(),
            Relation(token, pattern) => todo!(),
            Range(pattern, token, pattern1) => todo!(),
            Discard(token) => todo!(),
            Bind(mut_token, id_token) => {
                let TokenKind::Identifier(id) = &id_token.kind else {
                    unreachable!("Expected identifier token");
                };
                if let Some(bind_type) = bind_type {
                    if let Some(err) = self.scopes.check_local_variable(id) {
                        self.errors
                            .push(SourceError::new(id_token.range.clone(), err));
                    }
                    self.declare_variable(id, mut_token.is_some(), bind_type);
                }
            }
            Record(token, record_element_bases, token1) => todo!(),
            Array(token, array_element_bases, token1) => todo!(),
            SpreadDiscard => todo!(),
            And(pattern, token, pattern1) => todo!(),
            Or(pattern, token, pattern1) => todo!(),
            Not(token, pattern) => todo!(),
            Unknown { .. } => (),
        }
    }
    pub fn emit_pattern(
        &mut self,
        pattern: &Pattern<'s>,
        value: Register,
        bind_type: Option<BindType>,
    ) {
        match pattern {
            Grouping(token, pattern, token1) => todo!(),
            Constant(token, token1) => todo!(),
            Relation(token, pattern) => todo!(),
            Range(pattern, token, pattern1) => todo!(),
            Discard(token) => todo!(),
            Bind(_, id_token) => {
                let TokenKind::Identifier(id) = &id_token.kind else {
                    unreachable!("Expected identifier token");
                };
                let var = self.scopes.find_variable(id);
                if let Some((level, variable)) = var {
                    variable.initialize();
                    if !variable.mutable() && bind_type.is_none() {
                        self.errors.push(SourceError::new(
                            id_token.range.clone(),
                            ErrorCode::ImmutableVariableAssignment,
                        ));
                    } else if level == self.closures.len() {
                        let register = variable.register();
                        self.op_unary(register, OpCode::Assign, value);
                    } else {
                        let register = variable.register();
                        let level = self.closures.len() - level;
                        self.op_set_upvalue(value, level, register);
                    }
                } else {
                    self.errors.push(SourceError::new(
                        id_token.range.clone(),
                        ErrorCode::UndefinedVariableAssignment,
                    ));
                }
            }
            Record(token, record_element_bases, token1) => todo!(),
            Array(token, array_element_bases, token1) => todo!(),
            SpreadDiscard => todo!(),
            And(pattern, token, pattern1) => todo!(),
            Or(pattern, token, pattern1) => todo!(),
            Not(token, pattern) => todo!(),
            Unknown { .. } => (),
        }
    }
}
