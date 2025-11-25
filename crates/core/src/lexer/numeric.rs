use std::num::IntErrorKind;

use winnow::{
    combinator::{opt, trace},
    token::{one_of, take_while},
};

use super::{identifier::is_identifier_continue, prelude::*};

#[derive(Debug, Clone)]
pub enum NumberInfo<'s> {
    Invalid,

    Decimal(&'s str),

    // 存储原始值，避免超 f64 精度导致回写问题
    Hexadecimal(u64),
    Octal(u64),
    Binary(u64),
}

#[derive(Debug, Clone)]
struct ParsedPart {
    number: u64,
    ordinal: i32,
    radix: u32,
    integer_overflow: bool,
    is_valid_ordinal: bool,
    has_invalid_char: bool,
    has_leading_underscore: bool,
    has_trailing_underscore: bool,
}

fn parse_part(str: &str, radix: u32, is_valid_char: impl Fn(u8) -> bool) -> ParsedPart {
    let bytes = str.as_bytes();
    if bytes == b"0" {
        return ParsedPart {
            radix,
            number: 0,
            ordinal: 0,
            integer_overflow: false,
            is_valid_ordinal: true,
            has_invalid_char: false,
            has_leading_underscore: false,
            has_trailing_underscore: false,
        };
    }
    let mut has_invalid_char = bytes.iter().any(|&b| b != b'_' && !is_valid_char(b));
    let has_underscore = bytes.contains(&b'_');
    let has_leading_underscore = bytes.first() == Some(&b'_');
    let has_leading_zero = bytes.first() == Some(&b'0');
    let has_trailing_underscore = bytes.last() == Some(&b'_');

    let num = if has_invalid_char || has_underscore {
        let s: Vec<_> = bytes
            .iter()
            .filter(|&&b| is_valid_char(b))
            .cloned()
            .collect();
        // SAFETY: `from_utf8_unchecked` is safe here because we filter out invalid characters.
        u64::from_str_radix(unsafe { std::str::from_utf8_unchecked(&s) }, radix)
    } else {
        u64::from_str_radix(str, radix)
    };

    let integer_overflow = num
        .as_ref()
        .is_err_and(|e| e.kind() == &IntErrorKind::PosOverflow);
    has_invalid_char = has_invalid_char
        || num.as_ref().is_err_and(|e| {
            e.kind() == &IntErrorKind::Empty || e.kind() == &IntErrorKind::InvalidDigit
        });
    let number = num.unwrap_or(u64::MAX);
    let ordinal = number.try_into().ok();
    let is_valid_ordinal = !has_underscore && ordinal.is_some() && !has_leading_zero;
    let ordinal = ordinal.unwrap_or(i32::MAX);
    ParsedPart {
        radix,
        number,
        integer_overflow,
        ordinal,
        is_valid_ordinal,
        has_invalid_char,
        has_leading_underscore,
        has_trailing_underscore,
    }
}

fn number_part<'s>(i: &mut Input<'s>) -> Result<&'s str> {
    trace("number_part", take_while(1.., is_identifier_continue)).parse_next(i)
}

fn float_part<'s>(i: &mut Input<'s>) -> Result<&'s str> {
    trace(
        "number_part",
        take_while(1.., |c| is_identifier_continue(c) && c != 'e' && c != 'E'),
    )
    .parse_next(i)
}

fn is_valid_float_char(c: u8) -> bool {
    c.is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' || c == b'+' || c == b'-'
}

pub(super) fn number<'s>(i: &mut Input<'s>) -> Result<TokenKind<'s>> {
    trace(
        "number",
        move |i: &mut Input<'s>| -> Result<TokenKind<'s>> {
            let cp = i.checkpoint();
            let (part_i, range_i) = number_part.with_span().parse_next(i)?;

            let part_i_b = part_i.as_bytes();
            // parse integer literals with prefixes
            // 0x, 0o, 0b
            if part_i_b[0] == b'0' && part_i.len() > 1 {
                let integer = match &part_i_b[0..2] {
                    b"0x" | b"0X" => {
                        Some(parse_part(&part_i[2..], 16, |c: u8| c.is_ascii_hexdigit()))
                    }
                    b"0o" | b"0O" => {
                        Some(parse_part(&part_i[2..], 8, |c| (b'0'..=b'7').contains(&c)))
                    }
                    b"0b" | b"0B" => Some(parse_part(&part_i[2..], 2, |c| c == b'0' || c == b'1')),
                    _ => None,
                };
                if let Some(p) = integer {
                    let result = TokenKind::Number(
                        p.number as f64,
                        Box::new(match p.radix {
                            16 => NumberInfo::Hexadecimal(p.number),
                            8 => NumberInfo::Octal(p.number),
                            2 => NumberInfo::Binary(p.number),
                            _ => unreachable!(),
                        }),
                    );
                    if p.has_invalid_char {
                        return Ok(TokenKind::unknown(
                            result,
                            range_i,
                            DiagnosticCode::InvalidNumberLiteral,
                        ));
                    }
                    if p.integer_overflow {
                        return Ok(TokenKind::unknown(
                            result,
                            range_i,
                            DiagnosticCode::OverflowIntegerLiteral,
                        ));
                    }
                    if p.has_trailing_underscore {
                        return Ok(TokenKind::unknown(
                            result,
                            range_i,
                            DiagnosticCode::InvalidNumberLiteralUnderscore,
                        ));
                    }
                    return Ok(result);
                }
            }

            let (part, range) = if part_i_b.last() == Some(&b'e') || part_i_b.last() == Some(&b'E')
            {
                // need ('+' | '-')? number_part
                i.reset(&cp);
                (
                    number_part, // this ends with 'e' or 'E'
                    opt((opt(one_of(['+', '-'])), number_part)),
                )
                    .take()
                    .with_span()
                    .parse_next(i)?
            } else if part_i_b.iter().any(|&b| b == b'e' || b == b'E') {
                // float finished
                (part_i, range_i.clone())
            } else {
                // need ('.' float_part)? (('e' | 'E')? ('+' | '-')? number_part)?
                i.reset(&cp);
                (
                    number_part, // this doesn't contain 'e' or 'E'
                    opt(('.', float_part)),
                    opt((one_of(['e', 'E']), opt(one_of(['+', '-'])), number_part)),
                )
                    .take()
                    .with_span()
                    .parse_next(i)?
            };

            if range == range_i {
                // maybe ordinary number
                let result = handle_ordinal(part_i, range_i);
                if !result.is_unknown() {
                    return Ok(result);
                }
            }

            let bytes = part.as_bytes();

            let has_invalid_char = bytes.iter().any(|&b| b != b'_' && !is_valid_float_char(b));
            let has_underscore = bytes.contains(&b'_');
            let has_leading_underscore = bytes.first() == Some(&b'_');
            let has_trailing_underscore = bytes.last() == Some(&b'_');
            let num = if has_invalid_char || has_underscore {
                let s: Vec<_> = bytes
                    .iter()
                    .filter(|&&b| is_valid_float_char(b))
                    .cloned()
                    .collect();
                String::from_utf8(s).unwrap().parse::<f64>()
            } else {
                part.parse::<f64>()
            };
            let parse_failed = num.is_err();
            let parsed_num = num.unwrap_or_default();
            if has_invalid_char || parse_failed {
                return Ok(TokenKind::unknown(
                    TokenKind::Number(parsed_num, NumberInfo::Invalid.into()),
                    range,
                    DiagnosticCode::InvalidNumberLiteral,
                ));
            }
            if has_leading_underscore || has_trailing_underscore {
                return Ok(TokenKind::unknown(
                    TokenKind::Number(parsed_num, NumberInfo::Invalid.into()),
                    range,
                    DiagnosticCode::InvalidNumberLiteralUnderscore,
                ));
            }
            if parsed_num.is_infinite() || parsed_num.is_nan() {
                return Ok(TokenKind::unknown(
                    TokenKind::Number(parsed_num, NumberInfo::Invalid.into()),
                    range,
                    DiagnosticCode::OverflowNumberLiteral,
                ));
            }
            Ok(TokenKind::Number(
                parsed_num,
                Box::new(NumberInfo::Decimal(part)),
            ))
        },
    )
    .parse_next(i)
}

fn handle_ordinal(str: &str, range: SourceRange) -> TokenKind<'_> {
    let p = parse_part(str, 10, |c| c.is_ascii_digit());

    let result = if p.is_valid_ordinal {
        TokenKind::Ordinal(p.ordinal)
    } else {
        TokenKind::unknown(
            TokenKind::Ordinal(p.ordinal),
            range.clone(),
            DiagnosticCode::InvalidOrdinalLiteral,
        )
    };
    if p.has_invalid_char {
        return TokenKind::unknown(result, range, DiagnosticCode::InvalidNumberLiteral);
    }
    if p.has_leading_underscore || p.has_trailing_underscore {
        return TokenKind::unknown(
            result,
            range,
            DiagnosticCode::InvalidNumberLiteralUnderscore,
        );
    }
    result
}

pub(super) fn ordinal<'s>(i: &mut Input<'s>) -> Result<TokenKind<'s>> {
    trace(
        "ordinal",
        number_part.with_span().map(|(s, r)| handle_ordinal(s, r)),
    )
    .parse_next(i)
}
