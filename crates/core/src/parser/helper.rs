use winnow::{
    combinator::{opt, repeat},
    token::one_of,
};

use super::{expressions::expression, prelude::*, statements::statement};

pub(super) fn construct_statements_and_expression<'s, 'a>(
    arena: &'a AstArena,
    mut statements: Vec<Statement<'s, 'a>>,
    expression: Option<Expression<'s, 'a>>,
) -> (Vec<Statement<'s, 'a>>, Option<ABox<'a, Expression<'s, 'a>>>) {
    let expression = expression.map(|e| arena.alloc(e));
    if expression.is_some() || statements.is_empty() {
        return (statements, expression);
    }

    let last_statement = statements.pop().unwrap();
    let expression = match last_statement {
        Statement::BlockExpression(e) => Some(e),
        _ => {
            statements.push(last_statement);
            None
        }
    };
    (statements, expression)
}

pub(super) fn statements_and_expression<'s, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<(Vec<Statement<'s, 'a>>, Option<ABox<'a, Expression<'s, 'a>>>)> {
    let (statements, expression): (Vec<_>, _) = (
        repeat(0.., |i: &mut Input<'s>| statement(arena, i)),
        opt(|i: &mut Input<'s>| expression(arena, i)),
    )
        .parse_next(i)?;
    Ok(construct_statements_and_expression(
        arena, statements, expression,
    ))
}

pub(super) fn literal_token<'s>(i: &mut Input<'s>) -> Result<TokenRef<'s>> {
    one_of(|t: &Token<'s>| {
        matches!(t.kind, TokenKind::Number(_, _))
            || matches!(t.kind, TokenKind::Ordinal(_))
            || matches!(t.kind, TokenKind::String(_, _))
            || *t == Keyword::True
            || *t == Keyword::False
            || *t == Keyword::Nil
            || *t == Keyword::Nan
            || *t == Keyword::Inf
    })
    .map(TokenRef::borrow)
    .parse_next(i)
}

pub(super) fn variable_token<'s>(
    include_underscore: bool,
    include_global: bool,
) -> impl Parser<'s, TokenRef<'s>> {
    move |i: &mut Input<'s>| {
        let t = one_of(|t: &Token<'s>| {
            matches!(&t.kind, &TokenKind::Identifier(_))
                || matches!(&t.kind, &TokenKind::Keyword(kw)
                    if kw.is_reserved() || kw == Keyword::Underscore || kw == Keyword::Global)
        })
        .parse_next(i)?;
        let e = if !include_underscore && *t == Keyword::Underscore {
            Token::unknown(
                t.range.clone(),
                t.kind.clone(),
                DiagnosticCode::UnexpectedUnderscore,
            )
            .into()
        } else if !include_global && *t == Keyword::Global {
            Token::unknown(
                t.range.clone(),
                t.kind.clone(),
                DiagnosticCode::UnexpectedGlobal,
            )
            .into()
        } else if matches!(&t.kind, &TokenKind::Keyword(kw) if kw.is_reserved()) {
            Token::unknown(
                t.range.clone(),
                TokenKind::Keyword(Keyword::Underscore),
                DiagnosticCode::InvalidReservedKeyword,
            )
            .into()
        } else {
            t.into()
        };
        Ok(e)
    }
}

pub(super) fn token<'s>(token: impl PartialEq<Token<'s>> + Copy) -> impl Parser<'s, TokenRef<'s>> {
    move |i: &mut Input<'s>| {
        one_of(|t: &Token<'s>| token == *t)
            .map(TokenRef::borrow)
            .parse_next(i)
    }
}

pub(super) fn token_or_insert<'s>(
    token: impl Into<TokenKind<'s>> + PartialEq<Token<'s>> + Copy,
    error: DiagnosticCode,
) -> impl Parser<'s, TokenRef<'s>> {
    move |i: &mut Input<'s>| -> Result<TokenRef<'s>> {
        let pos = i.previous_token_end();
        opt(one_of(|t: &Token<'s>| token == *t))
            .map(|t: Option<&Token<'s>>| match t {
                Some(t) => t.into(),
                None => Token::unknown_at(pos, token.into(), error).into(),
            })
            .parse_next(i)
    }
}

pub(super) fn unknown_range<'s>(
    recovered: &Option<impl AstWalker<'s>>,
    tokens: &[TokenRef<'s>],
) -> SourceRange {
    let mut range = SourceRange {
        start: usize::MAX,
        end: usize::MIN,
    };
    for token in tokens {
        if token.range.start < range.start {
            range.start = token.range.start;
        }
        if token.range.end > range.end {
            range.end = token.range.end;
        }
    }
    if let Some(recovered) = recovered {
        range.start = range.start.min(recovered.range().start);
        range.end = range.end.max(recovered.range().end);
    }
    range
}
