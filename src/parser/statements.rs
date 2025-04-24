use winnow::combinator::{alt, dispatch, fail, opt, peek, seq};
use winnow::prelude::*;
use winnow::token::{any, one_of};

use crate::error::{ErrorCode, SourceRange};
use crate::lexer::{Keyword, Operator, Token, TokenKind};

use super::block_expressions::*;
use super::expressions::expression;
use super::helper::{parameter_list, token_boxed, token_or_insert, variable_token};
use super::patterns::{pattern, pattern_or_insert};
use super::{Input, Statement};

fn semicolon<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Box<Token<'a>>> {
    token_or_insert(Operator::Semicolon, ErrorCode::MissingSemicolon)
        .map(Box::new)
        .parse_next(i)
}

fn empty_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    token_boxed(Operator::Semicolon)
        .map(Statement::Empty)
        .parse_next(i)
}

fn fn_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    (
        token_boxed(Keyword::Fn),
        opt(variable_token(false, false)),
        parameter_list,
        block_expression.map(Box::new),
    )
        .map(|(kw, name, params, body)| {
            let name = Box::new(name.unwrap_or_else(|| {
                Token::unknown(
                    SourceRange {
                        start: kw.range.end,
                        end: kw.range.end,
                    },
                    TokenKind::Identifier("<name>".into()),
                    ErrorCode::MissingFunctionName,
                )
            }));
            Statement::Function(kw, name, params, body)
        })
        .parse_next(i)
}

fn return_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Return(
        token_boxed(Keyword::Return),
        opt(expression.map(Box::new)),
        semicolon,
    ))
    .parse_next(i)
}

fn break_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Break(
        token_boxed(Keyword::Break),
        opt(expression.map(Box::new)),
        semicolon,
    ))
    .parse_next(i)
}

fn continue_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Continue(
        token_boxed(Keyword::Continue),
        semicolon,
    ))
    .parse_next(i)
}

fn bind_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Bind(
        token_boxed(Keyword::Let),
        pattern_or_insert(false).map(Box::new),
        token_or_insert(Operator::Equal, ErrorCode::MissingBindOperator).map(Box::new),
        expression.map(Box::new),
        semicolon,
    ))
    .parse_next(i)
}

fn rebind_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Rebind(
        pattern(true).map(Box::new),
        token_boxed(Operator::Equal),
        expression.map(Box::new),
        semicolon,
    ))
    .parse_next(i)
}

fn assign_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Assign(
        expression.map(Box::new),
        one_of(|t: &Token<'a>| {
            *t == Operator::PlusEqual
                || *t == Operator::MinusEqual
                || *t == Operator::AsteriskEqual
                || *t == Operator::SlashEqual
                || *t == Operator::PercentEqual
                || *t == Operator::CaretEqual
                || *t == Operator::LogicalAndEqual
                || *t == Operator::LogicalOrEqual
                || *t == Operator::NullCoalescingEqual
                || *t == Operator::Equal
        })
        .map(|t: &Token<'a>| Box::new(t.to_owned())),
        expression.map(Box::new),
        semicolon,
    ))
    .parse_next(i)
}

fn expression_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    let mut insert_semicolon = peek(one_of(|t: &Token<'a>| {
        *t != Operator::CloseBrace
            && *t != TokenKind::Eof
            && *t != Keyword::Case
            && *t != Keyword::Else
    }));
    seq!(Statement::Expression(
        expression.map(Box::new),
        _: insert_semicolon,
        semicolon
    ))
    .parse_next(i)
}

fn unknown_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    fail.map(|t: &[Token<'a>]| Statement::unknown(t, ErrorCode::UnknownStatement))
        .parse_next(i)
}

pub(super) fn statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    dispatch! {peek(any);
        t if *t == Operator::OpenBrace => block_expression.map(Box::new).map(Statement::BlockExpression),
        t if *t == Keyword::If => if_expression.map(Box::new).map(Statement::BlockExpression),
        t if *t == Keyword::Loop => loop_expression.map(Box::new).map(Statement::BlockExpression),
        t if *t == Keyword::While => while_expression.map(Box::new).map(Statement::BlockExpression),
        t if *t == Keyword::Match => match_expression.map(Box::new).map(Statement::BlockExpression),
        t if *t == Keyword::For => for_in_expression.map(Box::new).map(Statement::BlockExpression),

        t if *t == Keyword::Fn => fn_statement,
        t if *t == Keyword::Return => return_statement,
        t if *t == Keyword::Break => break_statement,
        t if *t == Keyword::Continue => continue_statement,

        t if *t == Operator::Semicolon => empty_statement,

        t if *t == Keyword::Let => bind_statement,

        &Token{..} => alt((
            rebind_statement,
            assign_statement,
            expression_statement,
            unknown_statement,
        )),
    }
    .parse_next(i)
}
