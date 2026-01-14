use crate::{
    DiagnosticCode, Keyword,
    lexer::TokenKind,
    parser::{AstWalker, TokenRef},
};

use super::Emitter;

impl<'s, 'c> Emitter<'s, 'c> {
    pub(crate) fn check_static_operator_usage(
        &mut self,
        operator: &TokenRef<'s>,
        literal: &TokenRef<'s>,
    ) {
        if (**operator == Keyword::And || **operator == Keyword::Or || **operator == Keyword::Not)
            && !literal.is_boolean_literal()
        {
            self.diagnostics
                .push(DiagnosticCode::NonBooleanInLogical, literal.range());
        }
        let TokenKind::Operator(op) = &operator.kind else {
            return;
        };
        if op.is_arithmetic() {
            if !literal.is_number_nan_literal() {
                self.diagnostics
                    .push(DiagnosticCode::NonNumberInArithmetic, literal.range());
            }
        } else if op.is_logical() {
            if !literal.is_boolean_literal() {
                self.diagnostics
                    .push(DiagnosticCode::NonBooleanInLogical, literal.range());
            }
        } else if op.is_comparison() && !literal.is_number_nan_literal() && !literal.is_string() {
            self.diagnostics.push(
                DiagnosticCode::NonNumberOrStringInComparison,
                literal.range(),
            );
        }
    }
}
