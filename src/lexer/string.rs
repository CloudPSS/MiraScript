use std::borrow::Cow;

use winnow::{
    combinator::{alt, dispatch, eof, fail, peek, seq},
    error::{ContextError, ErrMode},
    prelude::*,
    stream::{AsChar, Location, Stream},
    token::{any, literal, one_of, take, take_till, take_while},
};

use crate::{
    lexer::{Input, Operator, Token, TokenKind},
    parser::{Expression, expression, to_input},
    utils::{SourceError, SourceRange},
};

use super::lex_balanced;
use super::{helper::is_identifier_start, tokens::identifier};

#[derive(Debug, Clone, PartialEq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    InvalidEscapedChar(SourceRange),
    Interpolation(Expression<'a>),
    EndOfString,
    EndOfFile,
}

pub(super) fn string<'a>(i: &mut Input<'a>) -> ModalResult<TokenKind<'a>> {
    let leading_ats = take_while(0.., ['@']).map(str::len).parse_next(i)?;
    let quote_begin = quote.parse_next(i)?;
    let mut content: Vec<_> = vec![];
    let unterminated = loop {
        let frag = fragment(quote_begin, leading_ats).parse_next(i)?;
        if matches!(frag, StringFragment::EndOfString) {
            break false;
        } else if matches!(frag, StringFragment::EndOfFile) {
            break true;
        } else {
            content.push(frag);
        }
    };
    if !unterminated && content.len() == 1 {
        if let StringFragment::Literal(s) = content[0] {
            return Ok(TokenKind::String(Cow::Borrowed(s)));
        }
    }
    let mut errors = vec![];
    if unterminated {
        errors.push(SourceError::new(
            SourceRange {
                start: i.previous_token_end(),
                end: i.previous_token_end(),
            },
            "Unterminated string",
        ));
    }
    let has_interpolation = content
        .iter()
        .any(|frag| matches!(frag, StringFragment::Interpolation(_)));

    let mut extract_invalid = |mut range: SourceRange| {
        let cp = i.checkpoint();
        i.reset_to_start();
        let s = &i[range.clone()];
        i.reset(&cp);
        range.start -= 1;
        errors.push(SourceError::new(range, "Invalid escape sequence"));
        Cow::Borrowed(s)
    };
    let token = if has_interpolation {
        let mut literals = vec![];
        let mut interpolations = vec![];
        let mut literal_pushed = false;
        for frag in content.into_iter() {
            if let StringFragment::Interpolation(expr) = frag {
                if !literal_pushed {
                    literals.push(Cow::Borrowed(""));
                }
                interpolations.push(expr);
                literal_pushed = false;
                continue;
            }
            let s = match frag {
                StringFragment::Literal(s) => Cow::Borrowed(s),
                StringFragment::EscapedChar(ch) => Cow::Owned(ch.to_string()),
                StringFragment::InvalidEscapedChar(r) => extract_invalid(r),
                _ => unreachable!(),
            };
            if literal_pushed {
                let last = literals.last_mut().unwrap();
                last.to_mut().push_str(&s);
            } else {
                literals.push(s);
                literal_pushed = true;
            }
        }
        if !literal_pushed {
            literals.push(Cow::Borrowed(""));
        }
        TokenKind::InterpolatedString(literals, interpolations)
    } else {
        let result = content.into_iter().fold(String::new(), |mut str, frag| {
            match frag {
                StringFragment::Literal(s) => str.push_str(s),
                StringFragment::EscapedChar(ch) => str.push(ch),
                StringFragment::InvalidEscapedChar(r) => {
                    let s = extract_invalid(r);
                    str.push_str(&s);
                }
                _ => unreachable!(),
            }
            str
        });
        TokenKind::String(Cow::Owned(result))
    };
    if !errors.is_empty() {
        return Ok(TokenKind::unknown_errors(token, errors));
    }
    Ok(token)
}

fn quote(i: &mut Input<'_>) -> ModalResult<char> {
    one_of(['\'', '"', '`']).parse_next(i)
}

fn fragment<'a>(
    quote: char,
    ats: usize,
) -> impl Parser<Input<'a>, StringFragment<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'a>| {
        alt((
            literal_str(quote, ats > 0).map(StringFragment::Literal),
            maybe_end_of_string(quote, ats),
            escaped_char(ats),
            eof.value(StringFragment::EndOfFile),
        ))
        .parse_next(i)
    }
}

fn maybe_end_of_string<'a>(
    quote: char,
    ats: usize,
) -> impl Parser<Input<'a>, StringFragment<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'a>| {
        (literal(quote), take_while(0..=ats, ['@']))
            .take()
            .map(|s: &str| {
                if s.len() == ats + 1 {
                    StringFragment::EndOfString
                } else {
                    StringFragment::Literal(s)
                }
            })
            .parse_next(i)
    }
}

fn literal_str<'a>(
    quote: char,
    verbatim: bool,
) -> impl Parser<Input<'a>, &'a str, ErrMode<ContextError>> {
    move |i: &mut Input<'a>| {
        if verbatim {
            take_till(1.., [quote, '$']).parse_next(i)
        } else {
            take_till(1.., [quote, '$', '\\']).parse_next(i)
        }
    }
}

fn escaped_char<'a>(
    ats: usize,
) -> impl Parser<Input<'a>, StringFragment<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'a>| {
        if ats == 0 {
            dispatch! {one_of(['\\','$']);
                '\\' => escaped_char_impl,
                '$' => interpolation(1),
                _ => fail,
            }
            .parse_next(i)
        } else {
            // in verbatim string, '\\' escape is not allowed, only '$' is allowed
            let dollars = take_while(1..=ats, ['$']).parse_next(i)?;
            if dollars.len() == ats {
                // interpolation
                interpolation(ats).parse_next(i)
            } else {
                // '$' is not escaped
                Ok(StringFragment::Literal(dollars))
            }
        }
    }
}

fn escaped_char_impl<'a>(i: &mut Input<'a>) -> ModalResult<StringFragment<'a>> {
    alt((
        'r'.value(StringFragment::EscapedChar('\r')),
        'n'.value(StringFragment::EscapedChar('\n')),
        't'.value(StringFragment::EscapedChar('\t')),
        'b'.value(StringFragment::EscapedChar('\x08')),
        'f'.value(StringFragment::EscapedChar('\x0C')),
        'v'.value(StringFragment::EscapedChar('\x0B')),
        '\\'.value(StringFragment::EscapedChar('\\')),
        '"'.value(StringFragment::EscapedChar('"')),
        '\''.value(StringFragment::EscapedChar('\'')),
        '`'.value(StringFragment::EscapedChar('`')),
        '$'.value(StringFragment::EscapedChar('$')),
        '0'.value(StringFragment::EscapedChar('\0')),
        (
            'x',
            (one_of(AsChar::is_hex_digit), one_of(AsChar::is_hex_digit)).take(),
        )
            .with_span()
            .map(|((_, v), r)| match u8::from_str_radix(v, 16) {
                Ok(ch) if ch <= 0x7f => StringFragment::EscapedChar(ch as char),
                _ => StringFragment::InvalidEscapedChar(r),
            }),
        ("u{", take_while(1.., AsChar::is_hex_digit), '}')
            .with_span()
            .map(|((_, v, _), r)| match u32::from_str_radix(v, 16) {
                Ok(ch) => {
                    if let Some(c) = char::from_u32(ch) {
                        StringFragment::EscapedChar(c)
                    } else {
                        StringFragment::InvalidEscapedChar(r)
                    }
                }
                Err(_) => StringFragment::InvalidEscapedChar(r),
            }),
        any.span().map(StringFragment::InvalidEscapedChar),
    ))
    .parse_next(i)
}

fn interpolation<'a>(
    dollar_count: usize,
) -> impl Parser<Input<'a>, StringFragment<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'a>| {
        // save cp
        let cp = i.checkpoint();

        let first = peek(any).parse_next(i)?;

        if first != '{' {
            if !is_identifier_start(first) {
                // invalid identifier
                let end = i.previous_token_end();
                let start = end - dollar_count;
                i.reset_to_start();
                let result = &i[start..end];
                i.reset(&cp);
                return Ok(StringFragment::Literal(result));
            }
            // '$' identifier
            let id = identifier.with_span().parse_next(i)?;
            let id = Token {
                kind: id.0,
                range: id.1,
            };
            return Ok(StringFragment::Interpolation(Expression::Variable(
                Box::new(id),
            )));
        }

        // '$' '{' expression '}'

        // lex until '}'
        let mut tokens = match lex_balanced(i, true, Operator::OpenBrace, Operator::CloseBrace) {
            Ok(tokens) => tokens,
            Err(e) => {
                i.reset(&cp);
                return Err(e);
            }
        };
        let expr: ModalResult<Expression<'_>> = {
            let mut token_input = to_input(tokens.as_slice());
            seq!(
                expression,
                _: eof,
            )
            .map(|(e,)| e)
            .parse_next(&mut token_input)
        };
        let (expr, next) = match expr {
            Ok(expr) => (expr, tokens.pop().unwrap()),
            Err(_) => {
                let last_token = tokens.pop().unwrap();
                let error = if last_token == TokenKind::Eof {
                    "Unterminated interpolation expression"
                } else {
                    "Bad interpolation expression"
                };
                let expr = if tokens.is_empty() {
                    let range = SourceRange {
                        start: last_token.range.start,
                        end: last_token.range.start,
                    };
                    Expression::unknown_range(tokens, range, error)
                } else {
                    Expression::unknown(tokens, error)
                };
                (expr, last_token)
            }
        };
        i.reset_to_start();
        take(next.range.end).parse_next(i)?;
        Ok(StringFragment::Interpolation(expr))
    }
}
