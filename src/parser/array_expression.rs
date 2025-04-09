use winnow::{
    ModalResult, Parser,
    combinator::{alt, opt, peek, preceded, repeat, terminated},
    token::{literal, one_of},
};

use crate::lexer::{Operator, Token};

use super::{
    ArrayInitElement, Expression, Input, expression, helper::spread_expression, ranges::range,
};

fn array_element<'a>(i: &mut Input<'_, 'a>) -> ModalResult<ArrayInitElement<'a>> {
    preceded(
        peek(one_of(|t: &Token<'a>| {
            *t != Operator::Comma && *t != Operator::CloseBracket
        })),
        alt((
            spread_expression.map(|(s, e)| ArrayInitElement::Spread(Box::new(s), Box::new(e))),
            range.map(|r| ArrayInitElement::Range(Box::new(r))),
            expression.map(|e| ArrayInitElement::Expression(Box::new(e))),
        )),
    )
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
