use std::{borrow::Cow, iter::zip, vec};

use winnow::{
    combinator::{alt, eof, fail, opt, peek, preceded},
    stream::{AsChar, Offset},
    token::{any, literal, one_of, take_till, take_while},
};

use super::{
    identifier::{identifier, is_identifier_special, is_identifier_start},
    prelude::*,
    tokens::token,
};

#[derive(Debug, Clone)]
pub enum StringFragment<'s> {
    Literal(&'s str),
    EscapedChar(char, &'s str),
    InvalidEscapedChar(SourceRange, DiagnosticCode),
    Interpolation(
        /// The dollar signs used for interpolation
        &'s str,
        /// The tokens inside the interpolation, will be taken to `TokenKind::InterpolatedString`
        Vec<Token<'s>>,
        /// The format string of the tokens inside the interpolation
        &'s str,
        /// The begin and end tokens for the interpolation, if any
        Option<Box<(Token<'s>, Token<'s>)>>,
    ),
    EndOfString,
    EndOfFile,
}

#[derive(Debug, Clone)]
pub struct StringInfo<'s> {
    pub leading_range: SourceRange,
    pub trailing_range: SourceRange,
    pub ats: usize,
    pub quote: Option<char>,
    pub content: Vec<StringFragment<'s>>,
}

const QUOTES: [char; 3] = ['\'', '"', '`'];

pub(super) fn string<'s>(i: &mut Input<'s>) -> Result<TokenKind<'s>> {
    let ((ats, quote), leading_range) = (take_while(0.., ['@']).map(str::len), one_of(QUOTES))
        .with_span()
        .parse_next(i)?;
    let info = StringInfo {
        leading_range,
        trailing_range: SourceRange::default(),
        ats,
        quote: Some(quote),
        content: vec![],
    };
    string_content(info).parse_next(i)
}

pub(super) fn string_content<'s>(mut info: StringInfo<'s>) -> impl Parser<'s, TokenKind<'s>> {
    move |i: &mut Input<'s>| {
        let unterminated = loop {
            let string_mode = info.quote.is_none();
            let frag = fragment(&mut info).parse_next(i)?;
            if matches!(frag, StringFragment::EndOfString) {
                break string_mode;
            } else if matches!(frag, StringFragment::EndOfFile) {
                break !string_mode;
            } else {
                info.content.push(frag);
            }
        };
        if !unterminated
            && info.content.len() == 1
            && let StringFragment::Literal(s) = info.content[0]
        {
            return Ok(TokenKind::String(Cow::Borrowed(s), info.clone().into()));
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
        let has_interpolation = info
            .content
            .iter()
            .any(|frag| matches!(frag, StringFragment::Interpolation(..)));

        let mut extract_invalid = |mut range: SourceRange, code: DiagnosticCode| {
            range.start -= 1;
            errors.push(SourceDiagnostic::new(range, code));
            Cow::Borrowed("")
        };
        let token = if has_interpolation {
            let mut literals = vec![];
            let mut interpolations = vec![];
            let mut literal_pushed = false;
            for frag in info.content.iter_mut() {
                if let StringFragment::Interpolation(_, expr, fmt, _) = frag {
                    if !literal_pushed {
                        literals.push(Cow::Borrowed(""));
                    }
                    interpolations.push((std::mem::take(expr), *fmt));
                    literal_pushed = false;
                    continue;
                }
                let s: Cow<'s, str> = match frag {
                    StringFragment::Literal(s) => Cow::Borrowed(s),
                    StringFragment::EscapedChar(ch, _) => Cow::Owned(ch.to_string()),
                    StringFragment::InvalidEscapedChar(r, c) => extract_invalid(r.clone(), *c),
                    _ => unreachable!(),
                };
                if s.is_empty() {
                    continue;
                }
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
            interpolations.push((vec![], ""));
            TokenKind::InterpolatedString(
                zip(literals, interpolations)
                    .map(|(l, (e, f))| (l, e, f))
                    .collect(),
                info.clone().into(),
            )
        } else {
            let result = info.content.iter().fold(String::new(), |mut str, frag| {
                match frag {
                    StringFragment::Literal(s) => str.push_str(s),
                    StringFragment::EscapedChar(ch, _) => str.push(*ch),
                    StringFragment::InvalidEscapedChar(r, c) => {
                        let s = extract_invalid(r.clone(), *c);
                        str.push_str(&s);
                    }
                    _ => unreachable!(),
                }
                str
            });
            TokenKind::String(Cow::Owned(result), info.clone().into())
        };
        if !errors.is_empty() {
            return Ok(TokenKind::unknown_errors(token, errors));
        }
        Ok(token)
    }
}

fn fragment<'s>(info: &mut StringInfo<'s>) -> impl Parser<'s, StringFragment<'s>> {
    move |i: &mut Input<'s>| {
        alt((
            literal_str(info.quote, info.ats > 0).map(StringFragment::Literal),
            escaped_char(info.ats),
            eof.value(StringFragment::EndOfFile),
            maybe_end_of_string(info),
        ))
        .parse_next(i)
    }
}

fn maybe_end_of_string<'s>(info: &mut StringInfo<'s>) -> impl Parser<'s, StringFragment<'s>> {
    move |i: &mut Input<'s>| {
        let Some(quote) = info.quote else {
            return fail.parse_next(i);
        };
        let (s, r) = (literal(quote), take_while(0..=info.ats, ['@']))
            .take()
            .with_span()
            .parse_next(i)?;

        if s.len() == info.ats + 1 {
            info.trailing_range = r;
            Ok(StringFragment::EndOfString)
        } else {
            Ok(StringFragment::Literal(s))
        }
    }
}

fn literal_str<'s>(quote: Option<char>, verbatim: bool) -> impl Parser<'s, &'s str> {
    move |i: &mut Input<'s>| {
        take_till(1.., |ch| {
            ch == '$' || (!verbatim && ch == '\\') || (quote.is_some() && quote.unwrap() == ch)
        })
        .parse_next(i)
    }
}

fn escaped_char<'s>(ats: usize) -> impl Parser<'s, StringFragment<'s>> {
    move |i: &mut Input<'s>| {
        if ats == 0 {
            alt((
                preceded(literal("\\"), escaped_char_impl),
                move |i: &mut Input<'s>| {
                    let dollar = literal("$").parse_next(i)?;
                    interpolation(dollar).parse_next(i)
                },
            ))
            .parse_next(i)
        } else {
            // in verbatim string, '\\' escape is not allowed, only '$' is allowed
            let dollars = take_while(1..=ats, ['$']).parse_next(i)?;
            if dollars.len() == ats {
                // interpolation
                interpolation(dollars).parse_next(i)
            } else {
                // '$' is not escaped
                Ok(StringFragment::Literal(dollars))
            }
        }
    }
}

fn escaped_char_impl<'s>(i: &mut Input<'s>) -> Result<StringFragment<'s>> {
    alt((
        '0'.value(StringFragment::EscapedChar('\0', "0")),
        'r'.value(StringFragment::EscapedChar('\r', "r")),
        'n'.value(StringFragment::EscapedChar('\n', "n")),
        't'.value(StringFragment::EscapedChar('\t', "t")),
        'b'.value(StringFragment::EscapedChar('\x08', "b")),
        'f'.value(StringFragment::EscapedChar('\x0C', "f")),
        'v'.value(StringFragment::EscapedChar('\x0B', "v")),
        '\\'.value(StringFragment::EscapedChar('\\', "\\")),
        '"'.value(StringFragment::EscapedChar('"', "\"")),
        '\''.value(StringFragment::EscapedChar('\'', "'")),
        '`'.value(StringFragment::EscapedChar('`', "`")),
        '$'.value(StringFragment::EscapedChar('$', "$")),
        (
            'x',
            (one_of(AsChar::is_hex_digit), one_of(AsChar::is_hex_digit)),
        )
            .take()
            .with_span()
            .map(|(s, r): (&str, _)| match u8::from_str_radix(&s[1..], 16) {
                Ok(ch) if ch <= 0x7f => StringFragment::EscapedChar(ch as char, s),
                _ => {
                    StringFragment::InvalidEscapedChar(r, DiagnosticCode::InvalidHexEscapeSequence)
                }
            }),
        ("u{", take_while(1.., AsChar::is_hex_digit), '}')
            .take()
            .with_span()
            .map(
                |(s, r): (&str, _)| match u32::from_str_radix(&s[2..s.len() - 1], 16) {
                    Ok(ch) => {
                        if let Some(c) = char::from_u32(ch) {
                            StringFragment::EscapedChar(c, s)
                        } else {
                            StringFragment::InvalidEscapedChar(
                                r,
                                DiagnosticCode::InvalidUnicodeEscapeSequence,
                            )
                        }
                    }
                    Err(_) => StringFragment::InvalidEscapedChar(
                        r,
                        DiagnosticCode::InvalidUnicodeEscapeSequence,
                    ),
                },
            ),
        any.span()
            .map(|r| StringFragment::InvalidEscapedChar(r, DiagnosticCode::InvalidEscapeSequence)),
        eof.value(StringFragment::EndOfFile),
    ))
    .parse_next(i)
}

fn format_string<'s>(i: &mut Input<'s>) -> Result<&'s str> {
    let start = i.checkpoint();

    let mut depth = 0; // 括号深度
    let mut in_class = false; // 是否在字符类 [... ] 内

    while !i.is_empty() {
        let checkpoint = i.checkpoint();

        match i.next_token() {
            Some('\\') => {
                // 转义：跳过下一个字符
                let _ = i.next_token();
            }
            Some('[') if !in_class => {
                in_class = true;
            }
            Some(']') if in_class => {
                in_class = false;
            }
            Some('(') if !in_class => {
                depth += 1;
            }
            Some(')') if !in_class => {
                if depth == 0 {
                    // 找到插值结束符，回退这个 ')'
                    // 返回从 start 到当前位置的字符串
                    let offset = checkpoint.offset_from(&start);
                    i.reset(&start);
                    return Ok(i.next_slice(offset));
                } else {
                    depth -= 1;
                }
            }
            Some(_) => {
                // 其他字符，继续
            }
            None => break,
        }
    }

    // 如果到这里说明没找到匹配的 ')'
    i.reset(&start);
    let offset = i.eof_offset();
    Ok(i.next_slice(offset))
}

fn interpolation<'s>(dollars: &'s str) -> impl Parser<'s, StringFragment<'s>> {
    move |i: &mut Input<'s>| -> Result<StringFragment<'s>> {
        // save cp
        let cp = i.checkpoint();

        let first = peek(opt(any)).parse_next(i)?;

        let part = match first {
            Some('{') => {
                // '$' block_expression
                let mut depth = 0;
                let tokens = match lex_impl(i, |t| {
                    if *t == Operator::OpenBrace {
                        depth += 1;
                    } else if *t == Operator::CloseBrace {
                        depth -= 1;
                    }
                    depth <= 0
                }) {
                    Ok(tokens) => tokens,
                    Err(e) => {
                        i.reset(&cp);
                        return Err(e);
                    }
                };
                StringFragment::Interpolation(dollars, tokens, "", None)
            }
            Some('(') => {
                // '$' '(' expression [ ':' format ] ')'
                let mut depth = 0;
                let mut brace_depth = 0;
                let mut has_format = false;
                let mut tokens = lex_impl(i, |t| {
                    if *t == Operator::OpenParen {
                        depth += 1;
                    } else if *t == Operator::CloseParen {
                        depth -= 1;
                    } else if *t == Operator::OpenBrace {
                        brace_depth += 1;
                    } else if *t == Operator::CloseBrace {
                        brace_depth -= 1;
                    } else if depth == 1 && brace_depth == 0 && *t == Operator::Colon {
                        has_format = true;
                        return true;
                    }
                    depth <= 0
                })
                .inspect_err(|_| {
                    i.reset(&cp);
                })?;

                let format = if has_format {
                    format_string.parse_next(i)?
                } else {
                    ""
                };

                let end = if has_format {
                    let last = tokens.pop_if(|f| *f == Operator::Colon);
                    debug_assert!(last.is_some());
                    Some(token(i, None)?)
                } else {
                    tokens.pop_if(|f| *f == Operator::CloseParen)
                }
                .unwrap_or_else(|| Token::empty(tokens.last().unwrap().range.end));
                let begin = tokens.remove(0);

                let filtered_tokens = if tokens.is_empty() {
                    vec![Token::empty(begin.range.end)]
                } else {
                    tokens
                };
                StringFragment::Interpolation(
                    dollars,
                    filtered_tokens,
                    format,
                    Some(Box::new((begin, end))),
                )
            }
            Some(ch) if is_identifier_start(ch) || is_identifier_special(ch) => {
                // '$' identifier
                let (mut kind, range) = identifier(true).with_span().parse_next(i)?;
                if let TokenKind::Keyword(kw) = kind
                    && !kw.is_constant()
                {
                    kind = TokenKind::unknown(
                        // Recover to nil for further analysis
                        Keyword::Nil,
                        range.clone(),
                        if kw.is_reserved() {
                            DiagnosticCode::InvalidReservedKeyword
                        } else {
                            DiagnosticCode::InvalidKeyword
                        },
                    );
                }
                let id = Token::new(kind, range);
                StringFragment::Interpolation(dollars, vec![id], "", None)
            }
            _ => {
                // invalid interpolation, return as literal
                StringFragment::Literal(dollars)
            }
        };

        Ok(part)
    }
}
