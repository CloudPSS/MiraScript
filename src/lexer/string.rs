use std::borrow::Cow;

use winnow::{
    combinator::{alt, dispatch, eof, fail, repeat},
    error::{ContextError, ErrMode},
    prelude::*,
    stream::{AsChar, Stream},
    token::{any, one_of, take, take_till, take_while},
};

use crate::{
    lexer::{Input, Operator, Range, Token, TokenError, TokenKind},
    parser::{Expression, expression, to_input},
};

use super::lex_balanced;

#[derive(Debug, Clone, PartialEq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    InvalidEscapedChar(Range),
    Interpolation(Expression<'a>),
}

pub(super) fn string<'a>(i: &mut Input<'a>) -> ModalResult<TokenKind<'a>> {
    let quote_begin = any
        .verify(|ch| *ch == '\'' || *ch == '"' || *ch == '`')
        .parse_next(i)?;
    let content: Vec<_> = repeat(0.., fragment(quote_begin)).parse_next(i)?;
    let unterminated: ModalResult<_> = eof.span().parse_next(i);
    if unterminated.is_err() {
        let quote_end = any.verify(|ch| *ch == quote_begin).parse_next(i)?;
        assert_eq!(quote_begin, quote_end);
    }
    if content.len() == 1 {
        if let StringFragment::Literal(s) = content[0] {
            return Ok(TokenKind::String(Cow::Borrowed(s)));
        }
    }
    let mut errors = vec![];
    if let Ok(r) = unterminated {
        errors.push(TokenError::new(r, "Unterminated string"));
    }
    let has_interpolation = content
        .iter()
        .any(|frag| matches!(frag, StringFragment::Interpolation(_)));

    let mut extract_invalid = |mut range: Range| {
        let cp = i.checkpoint();
        i.reset_to_start();
        let s = &i[range.clone()];
        i.reset(&cp);
        range.start -= 1;
        errors.push(TokenError::new(range, "Invalid escape sequence"));
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
                StringFragment::Interpolation(_) => unreachable!(),
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
                StringFragment::Interpolation(_) => unreachable!(),
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

fn fragment<'a>(quote: char) -> impl Parser<Input<'a>, StringFragment<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'a>| {
        alt((
            literal_str(quote).map(StringFragment::Literal),
            escaped_char,
        ))
        .parse_next(i)
    }
}

fn literal_str<'a>(quote: char) -> impl Parser<Input<'a>, &'a str, ErrMode<ContextError>> {
    move |i: &mut Input<'a>| {
        take_till(1.., [quote, '\\', '$'])
            .verify(|s: &str| !s.is_empty())
            .parse_next(i)
    }
}

fn escaped_char<'a>(i: &mut Input<'a>) -> ModalResult<StringFragment<'a>> {
    dispatch! {one_of(['\\','$']);
        '\\' => escaped_char_impl,
        '$' => interpolation,
        _ => fail,
    }
    .parse_next(i)
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

fn interpolation<'a>(i: &mut Input<'a>) -> ModalResult<StringFragment<'a>> {
    // next character must be '{'
    one_of('{').parse_next(i)?;

    // save cp
    let cp = i.checkpoint();

    // lex until '}'
    let tokens = match lex_balanced(i, true, Operator::OpenBrace, Operator::CloseBrace) {
        Ok(tokens) => tokens,
        Err(e) => {
            i.reset(&cp);
            return Err(e);
        }
    };
    let mut token_input = to_input(tokens.as_slice());
    let expr = match expression.parse_next(&mut token_input) {
        Ok(expr) => expr,
        Err(e) => {
            i.reset(&cp);
            return Err(e);
        }
    };
    let next_token =
        match one_of(|t: &Token<'a>| *t == Operator::CloseBrace).parse_next(&mut token_input) {
            Ok(token) => token,
            Err(e) => {
                i.reset(&cp);
                return Err(e);
            }
        };
    i.reset_to_start();
    take(next_token.range.end).parse_next(i)?;
    Ok(StringFragment::Interpolation(expr))
}
