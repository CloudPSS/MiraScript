use winnow::combinator::{delimited, opt, peek, repeat, terminated};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, one_of};

use crate::diagnostic::{DiagnosticCode, SourceRange};
use crate::lexer::{Keyword, Operator, Token, TokenKind};

use super::{Expression, Input, Statement, expressions::expression, statements::statement};

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
    i: &mut Input<'_, 's>,
) -> ModalResult<(Vec<Statement<'s>>, Option<Box<Expression<'s>>>)> {
    let (statements, expression): (Vec<_>, _) =
        (repeat(0.., statement), opt(expression)).parse_next(i)?;
    Ok(construct_statements_and_expression(statements, expression))
}


pub(super) fn literal_token<'s>(i: &mut Input<'_, 's>) -> ModalResult<Token<'s>> {
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

pub(super) fn variable_token<'t, 's: 't>(
    include_underscore: bool,
    include_global: bool,
) -> impl Parser<Input<'t, 's>, Token<'s>, ErrMode<ContextError>> + Copy {
    move |i: &mut Input<'t, 's>| {
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
            t.to_owned()
        };
        Ok(e)
    }
}

pub(super) fn token<'t, 's: 't>(
    token: impl Into<TokenKind<'s>> + Clone + PartialEq<Token<'s>>,
) -> impl Parser<Input<'t, 's>, Token<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'t, 's>| {
        one_of(|t: &Token<'s>| token == *t)
            .map(|t: &Token<'s>| t.to_owned())
            .parse_next(i)
    }
}
pub(super) fn token_boxed<'t, 's: 't>(
    token: impl Into<TokenKind<'s>> + Clone + PartialEq<Token<'s>>,
) -> impl Parser<Input<'t, 's>, Box<Token<'s>>, ErrMode<ContextError>> {
    move |i: &mut Input<'t, 's>| {
        one_of(|t: &Token<'s>| token == *t)
            .map(|t: &Token<'s>| Box::new(t.to_owned()))
            .parse_next(i)
    }
}

pub(super) fn token_or_insert<'t, 's: 't, T>(
    token: T,
    error: DiagnosticCode,
) -> impl Parser<Input<'t, 's>, Token<'s>, ErrMode<ContextError>>
where
    T: Into<TokenKind<'s>> + Clone,
    Token<'s>: PartialEq<T>,
{
    move |i: &mut Input<'_, 's>| -> ModalResult<Token<'s>> {
        let pos = Location::previous_token_end(i);
        opt(one_of(|t: &Token<'s>| *t == token))
            .map(|t: Option<&Token<'s>>| match t {
                Some(t) => t.to_owned(),
                None => Token::unknown(
                    SourceRange {
                        start: pos,
                        end: pos,
                    },
                    token.clone(),
                    error,
                ),
            })
            .parse_next(i)
    }
}
