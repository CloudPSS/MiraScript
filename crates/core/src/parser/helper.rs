use winnow::{
    combinator::{opt, repeat},
    token::one_of,
};

use super::{expressions::expression, prelude::*, statements::statement};

pub(super) fn construct_statements_and_expression<'s>(
    mut statements: Vec<Statement<'s>>,
    expression: Option<Expression<'s>>,
) -> (Vec<Statement<'s>>, Option<Box<Expression<'s>>>) {
    let expression = expression.map(Box::new);
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

pub(super) fn statements_and_expression<'s>(
    i: &mut Input<'s>,
) -> Result<(Vec<Statement<'s>>, Option<Box<Expression<'s>>>)> {
    let (statements, expression): (Vec<_>, _) =
        (repeat(0.., statement), opt(expression)).parse_next(i)?;
    Ok(construct_statements_and_expression(statements, expression))
}

pub(super) fn literal_token<'s>(i: &mut Input<'s>) -> Result<TokenRef<'s>> {
    one_of(|t: &Token<'s>| {
        matches!(t.kind, TokenKind::Number(_))
            || matches!(t.kind, TokenKind::Ordinal(_))
            || matches!(t.kind, TokenKind::String(_))
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
                None => Token::unknown(
                    SourceRange {
                        start: pos,
                        end: pos,
                    },
                    token.into(),
                    error,
                )
                .into(),
            })
            .parse_next(i)
    }
}
