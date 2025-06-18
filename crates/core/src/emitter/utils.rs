use crate::{Keyword, parser::Expression};

pub(super) fn is_global_expression(expr: &Expression) -> bool {
    matches!(expr, Expression::Variable(token) if token.kind == Keyword::Global)
}
