use crate::{
    error::{ErrorCode, SourceError},
    lexer::TokenKind,
    parser::{
        Pattern::{self, *},
        RecordPattern,
    },
};

use super::{
    Emitter, OpCode,
    opcode::Register,
    variable::{self, BindType, Variable},
};

impl<'s> Emitter<'s> {
    pub fn declare_pattern(&mut self, pattern: &'s Pattern<'s>, bind_type: Option<BindType>) {
        match pattern {
            Grouping(_, pattern, _) => self.declare_pattern(pattern, bind_type),
            Constant(token, token1) => todo!(),
            Relation(token, pattern) => todo!(),
            Range(pattern, token, pattern1) => todo!(),
            Discard(_) => (),
            Bind(mut_token, id_token) => {
                if let Some(bind_type) = bind_type {
                    self.declare_variable(id_token, mut_token.is_some(), bind_type);
                }
            }
            Record(_, elements, _) => {
                for element in elements {
                    match element {
                        RecordPattern::Named(_, _, pattern, _)
                        | RecordPattern::InterpolateNamed(_, _, pattern, _)
                        | RecordPattern::OmitNamed(_, pattern, _)
                        | RecordPattern::Unnamed(pattern, _)
                        | RecordPattern::Spread(_, pattern, _) => {
                            self.declare_pattern(pattern, bind_type);
                        }
                    }
                }
            }
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
        pattern: &'s Pattern<'s>,
        value: Register,
        bind_type: Option<BindType>,
    ) {
        match pattern {
            Grouping(token, pattern, token1) => todo!(),
            Constant(token, token1) => todo!(),
            Relation(token, pattern) => todo!(),
            Range(pattern, token, pattern1) => todo!(),
            Discard(_) => (),
            Bind(_, id_token) => {
                let TokenKind::Identifier(id) = &id_token.kind else {
                    return;
                };
                let var = self.scopes.find_variable(id);
                if let Some((level, variable)) = var {
                    let bind = bind_type.is_some();
                    let initialized = if bind {
                        variable.initialize();
                        true
                    } else {
                        variable.initialized()
                    };
                    if !variable.mutable() && !bind {
                        self.errors.push(SourceError::new(
                            id_token.range.clone(),
                            ErrorCode::ImmutableVariableAssignment,
                        ));
                    } else if level == self.closures.len() {
                        if !initialized {
                            self.errors.push(SourceError::new(
                                id_token.range.clone(),
                                ErrorCode::UninitializedVariable,
                            ));
                        }
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
            Record(_, elements, _) => {
                for (i, element) in elements.iter().enumerate() {
                    match element {
                        RecordPattern::Named(token, _, pattern, _) => {
                            let Some(id) = token.to_prop_name() else {
                                unreachable!("Expected identifier token");
                            };
                            let ret = self.add_reg();
                            self.op_get(ret, value, id);
                            self.emit_pattern(pattern, ret, bind_type);
                        }
                        RecordPattern::InterpolateNamed(..) => {}
                        RecordPattern::OmitNamed(_, pattern, _) => {
                            let Pattern::Bind(_, id) = pattern.as_ref() else {
                                unreachable!("Expected identifier token");
                            };
                            let TokenKind::Identifier(id) = &id.kind else {
                                unreachable!("Expected identifier token");
                            };
                            let ret = self.add_reg();
                            self.op_get(ret, value, id.as_ref());
                            self.emit_pattern(pattern, ret, bind_type);
                        }
                        RecordPattern::Unnamed(pattern, _) => {
                            let ret = self.add_reg();
                            self.op_get_index(ret, value, i);
                            self.emit_pattern(pattern, ret, bind_type);
                        }
                        RecordPattern::Spread(_, pattern, _) => {
                            todo!()
                        }
                    }
                }
            }
            Array(token, array_element_bases, token1) => todo!(),
            SpreadDiscard => todo!(),
            And(pattern, token, pattern1) => todo!(),
            Or(pattern, token, pattern1) => todo!(),
            Not(token, pattern) => todo!(),
            Unknown { .. } => (),
        }
    }
}
