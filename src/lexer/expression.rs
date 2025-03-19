use winnow::prelude::*;

use super::{Expression, Input, basic_expression::or};

pub(super) fn expression<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    or.parse_next(i)
}
