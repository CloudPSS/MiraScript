use winnow::{combinator::alt, token::take_till};

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
            || *t == Operator::Semicolon
            || *t == Operator::OpenBrace
            || *t == Operator::CloseBrace
            || *t == Operator::OpenBracket
            || *t == Operator::CloseBracket
            || *t == Operator::OpenParen
            || *t == Operator::CloseParen
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
