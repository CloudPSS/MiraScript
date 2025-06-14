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

pub(super) fn literal_token<'s>(i: &mut Input<'s>) -> Result<Token<'s>> {
    one_of(|t: &Token<'s>| {
        matches!(&t.kind, &TokenKind::Number(_))
            || matches!(&t.kind, &TokenKind::Ordinal(_))
            || matches!(&t.kind, &TokenKind::String(_))
            || *t == Keyword::True
            || *t == Keyword::False
            || *t == Keyword::Nil
            || *t == Keyword::Nan
            || *t == Keyword::Inf
    })
    .map(|t: &Token<'s>| t.to_owned())
    .parse_next(i)
}

pub(super) fn variable_token<'s>(
    include_underscore: bool,
    include_global: bool,
) -> impl Parser<'s, Token<'s>> {
    move |i: &mut Input<'s>| {
        let t = one_of(|t: &Token<'s>| {
            matches!(&t.kind, &TokenKind::Identifier(_))
                || (*t == Keyword::Underscore)
                || (*t == Keyword::Global)
        })
        .map(|t: &Token<'s>| t.to_owned())
        .parse_next(i)?;
        let e = if !include_underscore && t == Keyword::Underscore {
            Token::unknown(t.range, t.kind, DiagnosticCode::UnexpectedUnderscore)
        } else if !include_global && t == Keyword::Global {
            Token::unknown(t.range, t.kind, DiagnosticCode::UnexpectedGlobal)
        } else {
            t
        };
        // let e = match &t.kind {
        //     TokenKind::Keyword(Keyword::Underscore, str) if !include_underscore => Token::unknown(
        //         t.range,
        //         TokenKind::Identifier(str.unwrap_or("_")),
        //         DiagnosticCode::UnexpectedUnderscore,
        //     ),
        //     TokenKind::Keyword(Keyword::Global, str) if !include_global => Token::unknown(
        //         t.range,
        //         TokenKind::Identifier(str.unwrap_or("global")),
        //         DiagnosticCode::UnexpectedGlobal,
        //     ),
        //     _ => t,
        // };
        Ok(e)
    }
}

pub(super) fn token<'s>(token: impl PartialEq<Token<'s>> + Copy) -> impl Parser<'s, Token<'s>> {
    move |i: &mut Input<'s>| {
        one_of(|t: &Token<'s>| token == *t)
            .map(|t: &Token<'s>| t.to_owned())
            .parse_next(i)
    }
}
pub(super) fn token_boxed<'s>(
    token: impl PartialEq<Token<'s>> + Copy,
) -> impl Parser<'s, Box<Token<'s>>> {
    move |i: &mut Input<'s>| {
        one_of(|t: &Token<'s>| token == *t)
            .map(|t: &Token<'s>| Box::new(t.to_owned()))
            .parse_next(i)
    }
}

pub(super) fn token_or_insert<'s>(
    token: impl Into<TokenKind<'s>> + PartialEq<Token<'s>> + Copy,
    error: DiagnosticCode,
) -> impl Parser<'s, Token<'s>> {
    move |i: &mut Input<'s>| -> Result<Token<'s>> {
        let pos = i.previous_token_end();
        opt(one_of(|t: &Token<'s>| token == *t))
            .map(|t: Option<&Token<'s>>| match t {
                Some(t) => t.to_owned(),
                None => Token::unknown(
                    SourceRange {
                        start: pos,
                        end: pos,
                    },
                    token.into(),
                    error,
                ),
            })
            .parse_next(i)
    }
}
