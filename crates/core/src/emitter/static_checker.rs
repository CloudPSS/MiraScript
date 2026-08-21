use crate::{
    DiagnosticCode, Keyword, TokenKind,
    parser::{AstWalker, Expression, Range, TokenRef},
};

use super::Emitter;

impl<'s, 'c> Emitter<'s, 'c> {
    pub(crate) fn check_static_operator_usage(
        &mut self,
        operator: &TokenRef<'s>,
        literal: &TokenRef<'s>,
        right: bool,
    ) {
        if right
            && (**operator == Keyword::In || **operator == Keyword::NotIn)
            && **literal != Keyword::Global
        {
            self.diagnostics
                .push(DiagnosticCode::NonCompoundIn, literal.range());
            return;
        }
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

    pub(super) fn check_range_item(&mut self, item: &Expression<'s>) {
        if let Expression::Literal(l) = item
            && !l.is_number_literal()
        {
            self.diagnostics
                .push(DiagnosticCode::NonNumberInRange, l.range());
        }
    }

    pub(super) fn check_range(&mut self, range: &Range<'s>) {
        let Range(start, _, end) = range;
        self.check_range_item(start);
        self.check_range_item(end);
    }
}
