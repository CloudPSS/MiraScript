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

fn empty_statement<'s, 'a>(_arena: &'a AstArena, i: &mut Input<'s>) -> Result<Statement<'s, 'a>> {
    token(Operator::Semicolon)
        .map(Statement::Empty)
        .parse_next(i)
}

fn fn_statement<'s, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Statement<'s, 'a>> {
    (
        opt(token(Keyword::Pub)),
        token(Keyword::Fn),
        opt(variable_token(false, false)),
        |i: &mut Input<'s>| parameter_list(arena, i),
        (|i: &mut Input<'s>| block_expression(arena, i)).map(|e| arena.alloc(e)),
    )
        .map(|(kw_pub, kw_fn, name, params, body)| {
            let mut name = name.unwrap_or_else(|| {
                Token::unknown_at(
                    kw_fn.range.end,
                    TokenKind::Identifier("<name>"),
                    DiagnosticCode::MissingFunctionName,
                )
                .into()
            });
            if name.to_id_name() == Some("type") {
                name.wrap_as_unknown(DiagnosticCode::MissingOpenParenAfterType);
            }
            Statement::Function(kw_pub, kw_fn, name, params, body)
        })
        .parse_next(i)
}

fn return_statement<'s, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Statement<'s, 'a>> {
    seq!(Statement::Return(
        token(Keyword::Return),
        opt((|i: &mut Input<'s>| expression(arena, i)).map(|e| arena.alloc(e))),
        semicolon,
    ))
    .parse_next(i)
}

fn break_statement<'s, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Statement<'s, 'a>> {
    seq!(Statement::Break(
        token(Keyword::Break),
        opt((|i: &mut Input<'s>| expression(arena, i)).map(|e| arena.alloc(e))),
        semicolon,
    ))
    .parse_next(i)
}

fn continue_statement<'s, 'a>(
    _arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Statement<'s, 'a>> {
    seq!(Statement::Continue(token(Keyword::Continue), semicolon,)).parse_next(i)
}

fn bind_statement<'s, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Statement<'s, 'a>> {
    seq!(Statement::Bind(
        opt(token(Keyword::Pub)),
        token(Keyword::Let),
        pattern_or_insert(arena, false, |t| *t == Operator::Assign).map(|p| arena.alloc(p)),
        token_or_insert(Operator::Assign, DiagnosticCode::MissingBindOperator),
        expression_or_insert(arena, |t| *t == Operator::Semicolon).map(|e| arena.alloc(e)),
        semicolon,
    ))
    .parse_next(i)
}

fn rebind_statement<'s, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Statement<'s, 'a>> {
    seq!(Statement::Rebind(
        pattern_or_insert(arena, true, |t| *t == Operator::Assign).map(|p| arena.alloc(p)),
        token(Operator::Assign),
        expression_or_insert(arena, |t| *t == Operator::Semicolon).map(|e| arena.alloc(e)),
        semicolon,
    ))
    .parse_next(i)
}

fn const_statement<'s, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Statement<'s, 'a>> {
    seq!(Statement::Const(
        opt(token(Keyword::Pub)),
        token(Keyword::Const),
        variable_token(false, false).map(|mut t| {
            if t.to_id_name().is_some_and(|name| !name.starts_with('@')) {
                t.wrap_as_unknown(DiagnosticCode::InvalidConstantName)
            }
            t
        }),
        token_or_insert(Operator::Assign, DiagnosticCode::MissingBindOperator),
        expression_or_insert(arena, |t| *t == Operator::Semicolon).map(|e| arena.alloc(e)),
        semicolon,
    ))
    .parse_next(i)
}

fn insert_semicolon<'s>(i: &mut Input<'s>) -> Result<()> {
    peek(one_of(|t: &Token<'s>| {
        *t != Operator::CloseBrace
            && *t != TokenKind::Eof
            && *t != Keyword::Case
            && *t != Keyword::Else
    }))
    .value(())
    .parse_next(i)
}

fn assign_or_expression_statement<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Statement<'s, 'a>> {
    fn is_assign_op(t: &Token<'_>) -> bool {
        matches!(t.kind, TokenKind::Operator(t) if t == Operator::Assign || t.is_compound())
    }

    // Common expr of expr1 = expr2; and expr1;
    let expr1 = expression_or_insert(arena, is_assign_op)
        .map(|e| arena.alloc(e))
        .parse_next(i)?;
    let cp = i.checkpoint();
    // Try to parse as assignment first
    let assign: Result<_> = one_of(is_assign_op).parse_next(i);
    if let Ok(assign) = assign {
        let expr2 = expression_or_insert(arena, |t| *t == Operator::Semicolon)
            .map(|e| arena.alloc(e))
            .parse_next(i)?;
        let semi = semicolon.parse_next(i)?;
        return Ok(Statement::Assign(
            expr1,
            TokenRef::Borrowed(assign),
            expr2,
            semi,
        ));
    }
    // Fallback to expression statement
    i.reset(&cp);
    insert_semicolon.parse_next(i)?;
    let semi = semicolon.parse_next(i)?;
    Ok(Statement::Expression(expr1, semi))
}

fn unknown_statement<'s, 'a>(_arena: &'a AstArena, i: &mut Input<'s>) -> Result<Statement<'s, 'a>> {
    fail.map(|t: &[Token<'s>]| {
        Statement::unknown(
            t.iter().map(TokenRef::borrow).collect::<Vec<_>>(),
            DiagnosticCode::UnknownStatement,
        )
    })
    .parse_next(i)
}

fn mod_statement<'s, 'a>(arena: &'a AstArena, i: &mut Input<'s>) -> Result<Statement<'s, 'a>> {
    (
        opt(token(Keyword::Pub)),
        token(Keyword::Mod),
        opt(variable_token(false, false)),
        (|i: &mut Input<'s>| block_expression_no_expr(arena, i)).map(|e| arena.alloc(e)),
    )
        .map(|(kw_pub, kw_mod, name, body)| {
            let name = name.unwrap_or_else(|| {
                Token::unknown_at(
                    kw_mod.range.end,
                    TokenKind::Identifier("<name>"),
                    DiagnosticCode::MissingModuleName,
                )
                .into()
            });
            Statement::Module(kw_pub, kw_mod, name, body)
        })
        .parse_next(i)
}

pub(super) fn statement<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Statement<'s, 'a>> {
    dispatch! {peek(any);
        t if *t == Operator::OpenBrace => (|i: &mut Input<'s>| block_expression(arena, i)).map(|e| arena.alloc(e)).map(Statement::BlockExpression),
        t if *t == Keyword::If => (|i: &mut Input<'s>| if_expression(arena, i)).map(|e| arena.alloc(e)).map(Statement::BlockExpression),
        t if *t == Keyword::Loop => (|i: &mut Input<'s>| loop_expression(arena, i)).map(|e| arena.alloc(e)).map(Statement::BlockExpression),
        t if *t == Keyword::While => (|i: &mut Input<'s>| while_expression(arena, i)).map(|e| arena.alloc(e)).map(Statement::BlockExpression),
        t if *t == Keyword::Match => (|i: &mut Input<'s>| match_expression(arena, i)).map(|e| arena.alloc(e)).map(Statement::BlockExpression),
        t if *t == Keyword::For => (|i: &mut Input<'s>| for_in_expression(arena, i)).map(|e| arena.alloc(e)).map(Statement::BlockExpression),

        t if *t == Keyword::Return => |i: &mut Input<'s>| return_statement(arena, i),
        t if *t == Keyword::Break => |i: &mut Input<'s>| break_statement(arena, i),
        t if *t == Keyword::Continue => |i: &mut Input<'s>| continue_statement(arena, i),

        t if *t == Operator::Semicolon => |i: &mut Input<'s>| empty_statement(arena, i),

        &Token{..} => alt((
            |i: &mut Input<'s>| mod_statement(arena, i),
            |i: &mut Input<'s>| fn_statement(arena, i),
            |i: &mut Input<'s>| const_statement(arena, i),
            |i: &mut Input<'s>| bind_statement(arena, i),
            |i: &mut Input<'s>| rebind_statement(arena, i),
            |i: &mut Input<'s>| assign_or_expression_statement(arena, i),
            |i: &mut Input<'s>| unknown_statement(arena, i),
        )),
    }
    .parse_next(i)
}
