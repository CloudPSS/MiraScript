use std::borrow::Cow;

use winnow::{
    combinator::{alt, preceded, repeat},
    error::{ContextError, ErrMode},
    prelude::*,
    stream::{AsChar, Stream},
    token::{any, one_of, take_till, take_while},
};

use crate::tokenizer::{Input, Range, TokenError, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    InvalidEscapedChar(Range),
}

pub(super) fn string<'a>(i: &mut Input<'a>) -> ModalResult<TokenKind<'a>> {
    let quote_begin = any
        .verify(|ch| *ch == '\'' || *ch == '"' || *ch == '`')
        .parse_next(i)?;
    let content: Vec<_> = repeat(0.., fragment(quote_begin)).parse_next(i)?;
    let quote_end = any.verify(|ch| *ch == quote_begin).parse_next(i)?;
    assert_eq!(quote_begin, quote_end);
    if content.len() == 1 {
        if let StringFragment::Literal(s) = content[0] {
            return Ok(TokenKind::String(Cow::Borrowed(s)));
        }
    }
    let mut errors = vec![];
    let result = content.into_iter().fold(String::new(), |mut str, frag| {
        match frag {
            StringFragment::Literal(s) => str.push_str(s),
            StringFragment::EscapedChar(ch) => str.push(ch),
            StringFragment::InvalidEscapedChar(mut r) => {
                let cp = i.checkpoint();
                i.reset_to_start();
                str.push_str(&i[r.clone()]);
                i.reset(&cp);
                r.start -= 1;
                errors.push(TokenError::new(r, "Invalid escape sequence"));
            }
        }
        str
    });
    if !errors.is_empty() {
        return Ok(TokenKind::unknown(
            TokenKind::String(Cow::Owned(result)),
            errors,
        ));
    }
    Ok(TokenKind::String(Cow::Owned(result)))
}

fn fragment<'a>(quote: char) -> impl Parser<Input<'a>, StringFragment<'a>, ErrMode<ContextError>> {
    move |i: &mut Input<'a>| {
        alt((literal(quote).map(StringFragment::Literal), escaped_char)).parse_next(i)
    }
}

fn literal<'a>(quote: char) -> impl Parser<Input<'a>, &'a str, ErrMode<ContextError>> {
    move |i: &mut Input<'a>| {
        take_till(1.., [quote, '\\'])
            .verify(|s: &str| !s.is_empty())
            .parse_next(i)
    }
}

fn escaped_char<'a>(i: &mut Input<'a>) -> ModalResult<StringFragment<'a>> {
    preceded(
        '\\',
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
        )),
    )
    .parse_next(i)
}
