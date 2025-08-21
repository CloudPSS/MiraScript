use winnow::{
    combinator::{alt, opt, peek},
    error::EmptyError,
    token::{any, take_till},
};

use super::{
    basic_expressions::basic_expression, block_expressions::block_like_expression, prelude::*,
};

fn unknown_expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    take_till(1.., |t: &Token<'s>| {
        *t == TokenKind::Eof
            || *t == Keyword::If
            || *t == Keyword::Fn
            || *t == Keyword::Loop
            || *t == Keyword::While
            || *t == Keyword::Match
            || *t == Keyword::For
            || *t == Keyword::Let
            || *t == Keyword::Const
            || *t == Operator::Semicolon
            || *t == Operator::OpenBrace
            || *t == Operator::CloseBrace
            || *t == Operator::OpenBracket
            || *t == Operator::CloseBracket
            || *t == Operator::OpenParen
            || *t == Operator::CloseParen
            || *t == Operator::PlusAssign
            || *t == Operator::MinusAssign
            || *t == Operator::AsteriskAssign
            || *t == Operator::SlashAssign
            || *t == Operator::PercentAssign
            || *t == Operator::CaretAssign
            || *t == Operator::LogicalAndAssign
            || *t == Operator::LogicalOrAssign
            || *t == Operator::NullCoalescingAssign
            || *t == Operator::Assign
    })
    .map(|t: &[Token<'s>]| {
        Expression::unknown(
            t.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
            DiagnosticCode::UnknownExpression,
        )
    })
    .parse_next(i)
}

pub fn expression<'s>(i: &mut Input<'s>) -> Result<Expression<'s>> {
    alt((basic_expression, block_like_expression, unknown_expression)).parse_next(i)
}

pub fn expression_or_insert<'s>(
    mut insert_cond: impl FnMut(&'s Token<'s>) -> bool + Copy,
) -> impl Parser<'s, Expression<'s>> {
    move |i: &mut Input<'s>| {
        let e = opt(expression).parse_next(i)?;
        if let Some(e) = e {
            return Ok(e);
        }
        let start = i.previous_token_end();
        let next = peek(any).parse_next(i)?;
        if *next != TokenKind::Eof && !insert_cond(next) {
            return Err(winnow::error::ErrMode::Backtrack(EmptyError));
        }
        Ok(Expression::unknown_range(
            vec![Token::empty(start).into()],
            start..start,
            DiagnosticCode::ExpressionExpected,
        ))
    }
}
