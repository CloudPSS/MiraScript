use winnow::{
    ModalResult, Parser,
    combinator::{alt, opt, repeat, terminated},
    token::literal,
};

use crate::lexer::Operator;

use super::{
    ArrayInitElement, Expression, Input, expression, helper::spread_expression, ranges::range,
};

fn array_element<'a>(i: &mut Input<'_, 'a>) -> ModalResult<ArrayInitElement<'a>> {
    alt((
        spread_expression.map(|e| ArrayInitElement::Spread(Box::new(e))),
        range.map(|r| ArrayInitElement::Range(Box::new(r))),
        expression.map(|e| ArrayInitElement::Expression(Box::new(e))),
    ))
    .parse_next(i)
}

pub(super) fn array_expression<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Expression<'a>> {
    literal(Operator::OpenBracket).parse_next(i)?;
    let elements: Vec<_> = terminated(
        (
            repeat(0.., terminated(array_element, literal(Operator::Comma))),
            opt(array_element),
        )
            .map(
                |(mut v, e): (Vec<ArrayInitElement<'a>>, Option<ArrayInitElement<'a>>)| {
                    if let Some(e) = e {
                        v.push(e);
                    }
                    v
                },
            ),
        literal(Operator::CloseBracket),
    )
    .parse_next(i)?;
    Ok(Expression::Array(elements))
}
