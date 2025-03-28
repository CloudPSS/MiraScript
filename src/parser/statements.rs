use winnow::combinator::{alt, dispatch, fail, opt, peek, seq};
use winnow::prelude::*;
use winnow::token::{any, literal, one_of};

use crate::lexer::{Keyword, Operator, Token, TokenKind};
use crate::utils::SourceRange;

use super::expressions::expression;
use super::helper::{literal_or_insert, parameter_list, variable_token};
use super::{Expression, block_expressions::*};
use super::{Input, Statement};

fn semicolon<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Token<'a>> {
    literal_or_insert(Operator::Semicolon, "Missing semicolon").parse_next(i)
}

fn expression_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Expression(
        expression.map(Box::new),
        _: peek(one_of(|t: &Token<'a>| *t != Operator::CloseBrace && *t != TokenKind::Eof)),
        semicolon.map(Box::new)
    ))
    .parse_next(i)
}

fn empty_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    one_of(|t: &Token<'a>| *t == Operator::Semicolon)
        .map(|t: &Token<'a>| Statement::Empty(Box::new(t.to_owned())))
        .parse_next(i)
}

fn fn_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    (
        literal(Keyword::Fn),
        opt(variable_token(false, false)),
        parameter_list,
        block_expression.map(Box::new),
    )
        .map(|(key, name, params, body)| {
            let key = &key[0];
            let name = Box::new(name.unwrap_or_else(|| {
                Token::unknown(
                    SourceRange {
                        start: key.range.end,
                        end: key.range.end,
                    },
                    TokenKind::Identifier("<name>".into()),
                    "Missing function name",
                )
            }));
            Statement::Function(name, params, body)
        })
        .parse_next(i)
}

fn return_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Return(
        _: literal(Keyword::Return),
        opt(expression.map(Box::new)),
        _: semicolon,
    ))
    .parse_next(i)
}

fn break_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Break(
        _: literal(Keyword::Break),
        opt(expression.map(Box::new)),
        _: semicolon,
    ))
    .parse_next(i)
}

fn continue_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    (literal(Keyword::Continue), semicolon)
        .map(|_| Statement::Continue)
        .parse_next(i)
}

fn bind_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Bind(
        one_of(|t: &Token<'a>| *t == Keyword::Var || *t == Keyword::Val).map(|t: &Token<'a>| Box::new(t.to_owned())),
        variable_token(false, false).map(Box::new),
        _: literal(Operator::Equal),
        expression.map(Box::new),
        _: semicolon,
    ))
    .parse_next(i)
}

fn rebind_assign_statement<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Statement<'a>> {
    let cp = i.checkpoint();
    let (left, right) = seq!((
        expression,
        _: literal(Operator::Equal),
        expression.map(Box::new),
        _: semicolon,
    ))
    .parse_next(i)?;
    if let Expression::Variable(id) = left {
        return Ok(Statement::Rebind(id, right));
    } else if let Expression::Access(left, var) = left {
        return Ok(Statement::Assign(left, var, right));
    }
    i.reset(&cp);
    fail.parse_next(i)
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

        t if *t == Keyword::Var || *t == Keyword::Val => bind_statement,

        &Token{..} => alt((
            rebind_assign_statement,
            expression_statement,
        )),
    }
    .parse_next(i)
}
