use std::{borrow::Cow, vec};

use winnow::{
    combinator::{alt, dispatch, eof, fail, opt, peek},
    error::{ContextError, ErrMode},
    prelude::*,
    stream::{AsChar, Stream},
    token::{any, literal, one_of, take_till, take_while},
};

use crate::{
    error::{ErrorCode, SourceError, SourceRange},
    lexer::{Input, Operator, Token, TokenKind},
};

use super::{
    char_count::count_chars,
    identifier::{identifier, is_identifier_special, is_identifier_start},
    lex_balanced,
};

#[derive(Debug, Clone, PartialEq)]
enum StringFragment<'s> {
    Literal,
    EscapedChar(char),
    InvalidEscapedChar,
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
        let mut content = vec![];
        let unterminated = loop {
            let (frag, f_str) = fragment(quote_begin, leading_ats)
                .with_taken()
                .parse_next(i)?;
            let string_mode = quote_begin.is_none();
            if matches!(frag, StringFragment::EndOfString) {
                break string_mode;
            } else if matches!(frag, StringFragment::EndOfFile) {
                break !string_mode;
            } else {
                content.push((frag, f_str));
            }
        };
        if !unterminated && content.len() == 1 {
            if let (StringFragment::Literal, s) = content[0] {
                return Ok(TokenKind::String(Cow::Borrowed(s)));
            }
        }
        let start = if quote_begin.is_some() { 1 } else { 0 } + leading_ats;
        let mut errors = vec![];
        if unterminated {
            let len = content.iter().map(|(_, s)| count_chars(s)).sum::<usize>();
            errors.push(SourceError::new(
                SourceRange {
                    start: 0,
                    end: start + len,
                },
                ErrorCode::UnterminatedString,
            ));
        }
        let has_interpolation = content
            .iter()
            .any(|(frag, _)| matches!(frag, StringFragment::Interpolation(_)));

        let mut handle_literal =
            |frag: StringFragment<'s>, f_str: &'s str, offset: &mut usize| match frag {
                StringFragment::Literal => {
                    *offset += count_chars(f_str);
                    Cow::Borrowed(f_str)
                }
                StringFragment::EscapedChar(ch) => {
                    *offset += count_chars(f_str);
                    Cow::Owned(ch.to_string())
                }
                StringFragment::InvalidEscapedChar => {
                    let start = *offset;
                    *offset += count_chars(f_str);
                    errors.push(SourceError::new(
                        SourceRange {
                            start,
                            end: *offset,
                        },
                        ErrorCode::InvalidEscapeSequence,
                    ));
                    Cow::Borrowed(f_str)
                }
                _ => unreachable!(),
            };
        let token = if has_interpolation {
            let mut offset = start;
            let mut literals = vec![];
            let mut interpolations = vec![];
            let mut literal_pushed = false;
            for (frag, f_str) in content.into_iter() {
                if let StringFragment::Interpolation(mut expr) = frag {
                    if !literal_pushed {
                        literals.push(Cow::Borrowed(""));
                    }
                    for t in expr.iter_mut() {
                        t.range.start += offset;
                        t.range.end += offset;
                    }
                    interpolations.push(expr);
                    literal_pushed = false;
                    offset += count_chars(f_str);
                    continue;
                }
                let s = handle_literal(frag, f_str, &mut offset);
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
            let mut offset = start;
            let mut result = String::new();
            for (frag, f_str) in content.into_iter() {
                result.push_str(&handle_literal(frag, f_str, &mut offset));
            }
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
    let verbatim = leading_ats > 0 || quote_begin.is_none();
    move |i: &mut Input<'s>| {
        alt((
            literal_str(quote_begin, verbatim).value(StringFragment::Literal),
            escaped_char(verbatim, leading_ats),
            eof.value(StringFragment::EndOfFile),
            maybe_end_of_string(quote_begin, leading_ats),
        ))
        .parse_next(i)
    }
}

fn maybe_end_of_string<'s>(
    quote: Option<char>,
    leading_ats: usize,
) -> impl Parser<Input<'s>, StringFragment<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'s>| {
        let Some(quote) = quote else {
            return fail.parse_next(i);
        };
        (literal(quote), take_while(0..=leading_ats, ['@']))
            .take()
            .map(|s: &str| {
                if s.len() == leading_ats + 1 {
                    StringFragment::EndOfString
                } else {
                    StringFragment::Literal
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
    verbatim: bool,
    leading_ats: usize,
) -> impl Parser<Input<'s>, StringFragment<'s>, ErrMode<ContextError>> {
    move |i: &mut Input<'s>| {
        if !verbatim {
            dispatch! {one_of(['\\','$']);
                '\\' => escaped_char_impl,
                '$' => interpolation(1),
                _ => fail,
            }
            .parse_next(i)
        } else {
            let dollar_count = if leading_ats < 1 { 1 } else { leading_ats };
            // in verbatim string, '\\' escape is not allowed, only '$' is allowed
            let dollars = take_while(1..=dollar_count, ['$']).parse_next(i)?;
            if dollars.len() == dollar_count {
                // interpolation
                interpolation(dollar_count).parse_next(i)
            } else {
                // '$' is not escaped
                Ok(StringFragment::Literal)
            }
        }
    }
}

fn escaped_char_impl<'s>(i: &mut Input<'s>) -> ModalResult<StringFragment<'s>> {
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
            .map(|(_, v)| match u8::from_str_radix(v, 16) {
                Ok(ch) if ch <= 0x7f => StringFragment::EscapedChar(ch as char),
                _ => StringFragment::InvalidEscapedChar,
            }),
        ("u{", take_while(1.., AsChar::is_hex_digit), '}').map(
            |(_, v, _)| match u32::from_str_radix(v, 16) {
                Ok(ch) => {
                    if let Some(c) = char::from_u32(ch) {
                        StringFragment::EscapedChar(c)
                    } else {
                        StringFragment::InvalidEscapedChar
                    }
                }
                Err(_) => StringFragment::InvalidEscapedChar,
            },
        ),
        any.value(StringFragment::InvalidEscapedChar),
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
                return Ok(StringFragment::Literal);
            }
            // '$' identifier
            let id = identifier.with_taken().parse_next(i)?;
            let id = Token {
                kind: id.0,
                range: SourceRange {
                    start: dollar_count,
                    end: dollar_count + count_chars(id.1),
                },
                leading_trivia: vec![],
                trailing_trivia: vec![],
            };
            return Ok(StringFragment::Interpolation(vec![id]));
        }

        // '$' '{' expression '}'

        // lex until '}'
        let tokens = match lex_balanced(i, dollar_count, Operator::OpenBrace, Operator::CloseBrace)
        {
            Ok(tokens) => tokens,
            Err(e) => {
                i.reset(&cp);
                return Err(e);
            }
        };
        Ok(StringFragment::Interpolation(tokens))
    }
}

pub(super) fn apply_interpolation_offset(token: &mut TokenKind, offset: usize) {
    if let TokenKind::InterpolatedString(_, sub_tokens) = token {
        for tt in sub_tokens {
            for t in tt {
                t.range.start += offset;
                t.range.end += offset;
            }
        }
    } else if let TokenKind::Unknown {
        recovered: Some(recovered),
        errors,
    } = token
    {
        if recovered.is_string() || recovered.is_interpolated_string() {
            apply_interpolation_offset(recovered, offset);
            for e in errors {
                e.range.start += offset;
                e.range.end += offset;
            }
        }
    }
}
