use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::token::take_till;

use crate::{
    diagnostic::DiagnosticCode,
    lexer::{Keyword, Operator, Token, TokenKind},
};

use super::{
    Expression, Input, basic_expressions::basic_expression,
    block_expressions::block_like_expression,
};

fn unknown_expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
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
    .map(|t: &[Token<'s>]| Expression::unknown(t, DiagnosticCode::UnknownExpression))
    .parse_next(i)
}

pub fn expression<'s>(i: &mut Input<'_, 's>) -> ModalResult<Expression<'s>> {
    alt((block_like_expression, basic_expression, unknown_expression)).parse_next(i)
}
