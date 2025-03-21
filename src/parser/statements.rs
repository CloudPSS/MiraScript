use winnow::combinator::{alt, dispatch, fail, opt, peek, seq};
use winnow::prelude::*;
use winnow::token::{any, literal, one_of};

use super::expressions::expression;
use super::helper::{parameter_list, variable_token};
use super::{Expression, block_expressions::*};
use crate::lexer::{Keyword, Operator, Token};

use super::{Input, Statement};

fn expression_statement<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Expression(
        expression.map(Box::new),
        _: literal(Operator::Semicolon),
    ))
    .parse_next(i)
}

fn empty_statement<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Statement<'a>> {
    literal(Operator::Semicolon)
        .map(|_| Statement::Empty)
        .parse_next(i)
}

fn fn_statement<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Function(
        _: literal(Keyword::Fn),
        variable_token(false, false).map(Box::new),
        parameter_list,
        block_expression.map(Box::new),
    ))
    .parse_next(i)
}

fn return_statement<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Return(
        _: literal(Keyword::Return),
        opt(expression.map(Box::new)),
        _: literal(Operator::Semicolon),
    ))
    .parse_next(i)
}

fn break_statement<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Break(
        _: literal(Keyword::Break),
        opt(expression.map(Box::new)),
        _: literal(Operator::Semicolon),
    ))
    .parse_next(i)
}

fn continue_statement<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Statement<'a>> {
    (literal(Keyword::Continue), literal(Operator::Semicolon))
        .map(|_| Statement::Continue)
        .parse_next(i)
}

fn bind_statement<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Statement<'a>> {
    seq!(Statement::Bind(
        one_of(|t: &Token<'a>| *t == Keyword::Var || *t == Keyword::Val).map(|t: &Token<'a>| Box::new(t.to_owned())),
        variable_token(false, false).map(Box::new),
        _: literal(Operator::Equal),
        expression.map(Box::new),
        _: literal(Operator::Semicolon),
    ))
    .parse_next(i)
}

fn rebind_assign_statement<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Statement<'a>> {
    let cp = i.checkpoint();
    let (left, right) = seq!((
        expression,
        _: literal(Operator::Equal),
        expression.map(Box::new),
        _: literal(Operator::Semicolon),
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

pub(super) fn statement<'t, 'a: 't>(i: &mut Input<'t, 'a>) -> ModalResult<Statement<'a>> {
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
