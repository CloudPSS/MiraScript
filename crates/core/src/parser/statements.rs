use winnow::{
    combinator::{alt, dispatch, fail, opt, peek, seq},
    token::{any, one_of},
};

use super::{
    block_expressions::*,
    expressions::{expression, expression_or_insert},
    helper::{token, token_or_insert, variable_token},
    parameter_list::parameter_list,
    patterns::pattern_or_insert,
    prelude::*,
};

pub(super) fn semicolon<'s>(i: &mut Input<'s>) -> Result<TokenRef<'s>> {
    token_or_insert(Operator::Semicolon, DiagnosticCode::MissingSemicolon).parse_next(i)
}

fn empty_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    token(Operator::Semicolon)
        .map(Statement::Empty)
        .parse_next(i)
}

fn fn_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    (
        token(Keyword::Fn),
        opt(variable_token(false, false)),
        parameter_list,
        block_expression.map(Box::new),
    )
        .map(|(kw, name, params, body)| {
            let name = name.unwrap_or_else(|| {
                Token::unknown(
                    kw.range.end..kw.range.end,
                    TokenKind::Identifier("<name>"),
                    DiagnosticCode::MissingFunctionName,
                )
                .into()
            });
            Statement::Function(kw, name, params, body)
        })
        .parse_next(i)
}

fn return_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    seq!(Statement::Return(
        token(Keyword::Return),
        opt(expression.map(Box::new)),
        semicolon,
    ))
    .parse_next(i)
}

fn break_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    seq!(Statement::Break(
        token(Keyword::Break),
        opt(expression.map(Box::new)),
        semicolon,
    ))
    .parse_next(i)
}

fn continue_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    seq!(Statement::Continue(token(Keyword::Continue), semicolon,)).parse_next(i)
}

fn bind_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    seq!(Statement::Bind(
        token(Keyword::Let),
        pattern_or_insert(false, |t| *t == Operator::Assign).map(Box::new),
        token_or_insert(Operator::Assign, DiagnosticCode::MissingBindOperator),
        expression_or_insert(|t| *t == Operator::Semicolon).map(Box::new),
        semicolon,
    ))
    .parse_next(i)
}

fn rebind_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    seq!(Statement::Rebind(
        pattern_or_insert(true, |t| *t == Operator::Assign).map(Box::new),
        token(Operator::Assign),
        expression_or_insert(|t| *t == Operator::Semicolon).map(Box::new),
        semicolon,
    ))
    .parse_next(i)
}

fn const_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    seq!(Statement::Const(
        token(Keyword::Const),
        variable_token(false, false).map(|t| {
            if t.to_id_name().is_some_and(|name| !name.starts_with('@')) {
                t.wrap_as_unknown(DiagnosticCode::InvalidConstName)
            } else {
                t
            }
        }),
        token_or_insert(Operator::Assign, DiagnosticCode::MissingBindOperator),
        expression_or_insert(|t| *t == Operator::Semicolon).map(Box::new),
        semicolon,
    ))
    .parse_next(i)
}

fn assign_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    fn is_assign_op(t: &Token<'_>) -> bool {
        *t == Operator::PlusAssign
            || *t == Operator::MinusAssign
            || *t == Operator::AsteriskAssign
            || *t == Operator::SlashAssign
            || *t == Operator::PercentAssign
            || *t == Operator::CaretAssign
            || *t == Operator::LogicalAndAssign
            || *t == Operator::LogicalOrAssign
            || *t == Operator::NullCoalescingAssign
            || *t == Operator::Assign
    }
    seq!(Statement::Assign(
        expression_or_insert(is_assign_op).map(Box::new),
        one_of(is_assign_op).map(TokenRef::borrow),
        expression_or_insert(|t| *t == Operator::Semicolon).map(Box::new),
        semicolon,
    ))
    .parse_next(i)
}

fn expression_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    let mut insert_semicolon = peek(one_of(|t: &Token<'s>| {
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

fn unknown_statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
    fail.map(|t: &[Token<'s>]| {
        Statement::unknown(
            t.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
            DiagnosticCode::UnknownStatement,
        )
    })
    .parse_next(i)
}

pub(super) fn statement<'s>(i: &mut Input<'s>) -> Result<Statement<'s>> {
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
        t if *t == Keyword::Const => const_statement,

        &Token{..} => alt((
            rebind_statement,
            assign_statement,
            expression_statement,
            unknown_statement,
        )),
    }
    .parse_next(i)
}
