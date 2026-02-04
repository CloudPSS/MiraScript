use winnow::{
    combinator::{opt, peek, repeat},
    token::any,
};

use super::{
    expressions::expression,
    helper::construct_statements_and_expression,
    prelude::*,
    statements::{semicolon, statement},
};

pub fn script<'s>(i: &mut Input<'s>) -> Result<Script<'s>> {
    let mut statements = vec![];
    let exp: Option<Expression>;
    let eof: Option<&Token>;
    loop {
        let s: Vec<_> = repeat(0.., statement).parse_next(i)?;
        let s_empty = s.is_empty();
        statements.extend(s);
        let e = opt(expression).parse_next(i)?;
        let next = peek(opt(any)).parse_next(i)?;
        if let Some(next) = next {
            if next.is_eof() {
                exp = e;
                eof = any.parse_next(i)?.into();
                break;
            }
        } else {
            eof = None;
            exp = e;
            break;
        }
        if let Some(e) = e {
            let s = semicolon.parse_next(i)?;
            statements.push(Statement::Expression(e.into(), s));
        } else if s_empty {
            // eats nothing in this loop and not reach the end
            // eats next token and try again
            let next = any.parse_next(i)?;
            if *next == Operator::CloseParen {
                statements.push(Statement::unknown(
                    [next.into()],
                    DiagnosticCode::UnmatchedCloseParen,
                ));
            } else if *next == Operator::CloseBracket {
                statements.push(Statement::unknown(
                    [next.into()],
                    DiagnosticCode::UnmatchedCloseBracket,
                ));
            } else if *next == Operator::CloseBrace {
                statements.push(Statement::unknown(
                    [next.into()],
                    DiagnosticCode::UnmatchedCloseBrace,
                ));
            } else {
                statements.push(Statement::unknown(
                    [next.into()],
                    DiagnosticCode::UnexpectedToken,
                ));
            }
        }
    }
    let (statements, exp) = construct_statements_and_expression(statements, exp);
    let eof = eof.map(TokenRef::borrow).unwrap_or_else(|| {
        Token::unknown_at(0, TokenKind::Eof, DiagnosticCode::UnexpectedToken).into()
    });
    Ok(Script(statements, exp, eof))
}
