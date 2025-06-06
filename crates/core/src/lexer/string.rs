use std::{borrow::Cow, vec};

use winnow::{
    combinator::{alt, dispatch, eof, fail, opt, peek},
    error::{ContextError, ErrMode},
    prelude::*,
    stream::{AsChar, Location, Stream},
    token::{any, literal, one_of, take_till, take_while},
};

use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    lexer::{Input, Operator, Token, TokenKind},
};

use super::{
    identifier::{identifier, is_identifier_special, is_identifier_start},
    lex_balanced,
};

#[derive(Debug, Clone, PartialEq)]
enum StringFragment<'s> {
    Literal(&'s str),
    EscapedChar(char),
    InvalidEscapedChar(SourceRange),
    Interpolation(Vec<Token<'s>>),
    EndOfString,
    EndOfFile,
}

const QUOTES: [char; 3] = ['\'', '"', '`'];

pub(super) fn string<'s>(i: &mut Input<'s>) -> ModalResult<TokenKind<'s>> {
    let leading_ats = take_while(0.., ['@']).map(str::len).parse_next(i)?;
    let quote_begin = one_of(QUOTES).parse_next(i)?;
    string_content(Some(quote_begin), leading_ats).parse_next(i)
}

pub(super) fn string_content<'s>(
    quote_begin: Option<char>,
    leading_ats: usize,
) -> impl Parser<Input<'s>, TokenKind<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'s>| {
        let mut content: Vec<_> = vec![];
        let unterminated = loop {
            let frag = fragment(quote_begin, leading_ats).parse_next(i)?;
            let string_mode = quote_begin.is_none();
            if matches!(frag, StringFragment::EndOfString) {
                break string_mode;
            } else if matches!(frag, StringFragment::EndOfFile) {
                break !string_mode;
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
            errors.push(SourceDiagnostic::new(
                SourceRange {
                    start: i.previous_token_end(),
                    end: i.previous_token_end(),
                },
                DiagnosticCode::UnterminatedString,
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
            errors.push(SourceDiagnostic::new(range, DiagnosticCode::InvalidEscapeSequence));
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
}

fn fragment<'s>(
    quote_begin: Option<char>,
    leading_ats: usize,
) -> impl Parser<Input<'s>, StringFragment<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'s>| {
        alt((
            literal_str(quote_begin, leading_ats > 0).map(StringFragment::Literal),
            escaped_char(leading_ats),
            eof.value(StringFragment::EndOfFile),
            maybe_end_of_string(quote_begin, leading_ats),
        ))
        .parse_next(i)
    }
}

fn maybe_end_of_string<'s>(
    quote: Option<char>,
    ats: usize,
) -> impl Parser<Input<'s>, StringFragment<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'s>| {
        let Some(quote) = quote else {
            return fail.parse_next(i);
        };
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

fn literal_str<'s>(
    quote: Option<char>,
    verbatim: bool,
) -> impl Parser<Input<'s>, &'s str, ErrMode<ContextError>> {
    move |i: &mut Input<'s>| {
        take_till(1.., |ch| {
            ch == '$' || (!verbatim && ch == '\\') || (quote.is_some() && quote.unwrap() == ch)
        })
        .parse_next(i)
    }
}

fn escaped_char<'s>(
    ats: usize,
) -> impl Parser<Input<'s>, StringFragment<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'s>| {
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

fn escaped_char_impl<'s>(i: &mut Input<'s>) -> ModalResult<StringFragment<'s>> {
    alt((
        '0'.value(StringFragment::EscapedChar('\0')),
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
        eof.value(StringFragment::EndOfFile),
    ))
    .parse_next(i)
}

fn interpolation<'s>(
    dollar_count: usize,
) -> impl Parser<Input<'s>, StringFragment<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'s>| {
        // save cp
        let cp = i.checkpoint();

        let first = peek(opt(any)).parse_next(i)?;

        if first != Some('{') {
            if first.is_none()
                || !is_identifier_start(first.unwrap()) && !is_identifier_special(first.unwrap())
            {
                // invalid identifier
                let end = i.previous_token_end();
                let start = end - dollar_count;
                i.reset_to_start();
                let result = &i[start..end];
                i.reset(&cp);
                return Ok(StringFragment::Literal(result));
            }
            // '$' identifier
            let id = identifier(true).with_span().parse_next(i)?;
            let id = Token {
                kind: id.0,
                range: id.1,
                leading_trivia: vec![],
                trailing_trivia: vec![],
            };
            return Ok(StringFragment::Interpolation(vec![id]));
        }

        // '$' '{' expression '}'

        // lex until '}'
        let tokens = match lex_balanced(i, Operator::OpenBrace, Operator::CloseBrace) {
            Ok(tokens) => tokens,
            Err(e) => {
                i.reset(&cp);
                return Err(e);
            }
        };
        Ok(StringFragment::Interpolation(tokens))
    }
}
