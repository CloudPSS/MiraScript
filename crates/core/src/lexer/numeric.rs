use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use winnow::{
    combinator::{opt, trace},
    prelude::*,
    token::{one_of, take_while},
};

use crate::diagnostic::{DiagnosticCode, SourceRange};

use super::{Input, TokenKind};
use super::{Result, identifier::is_identifier_continue};

struct ParsedPart {
    number: f64,
    ordinal: i32,
    is_valid_ordinal: bool,
    has_invalid_char: bool,
    has_leading_underscore: bool,
    has_trailing_underscore: bool,
}

fn parse_part(bytes: &[u8], radix: u32, is_valid_char: impl Fn(u8) -> bool) -> ParsedPart {
    if bytes == b"0" {
        return ParsedPart {
            number: 0f64,
            ordinal: 0,
            is_valid_ordinal: true,
            has_invalid_char: false,
            has_leading_underscore: false,
            has_trailing_underscore: false,
        };
    }
    let has_invalid_char = bytes.iter().any(|&b| b != b'_' && !is_valid_char(b));
    let has_underscore = bytes.iter().any(|&b| b == b'_');
    let has_leading_underscore = bytes.first() == Some(&b'_');
    let has_leading_zero = bytes.first() == Some(&b'0');
    let has_trailing_underscore = bytes.last() == Some(&b'_');

    let num = if has_invalid_char || has_underscore {
        let s: Vec<_> = bytes
            .iter()
            .filter(|&&b| is_valid_char(b))
            .cloned()
            .collect();
        BigUint::parse_bytes(&s, radix)
    } else {
        BigUint::parse_bytes(bytes, radix)
    };
    let parse_failed = num.is_none();
    let has_invalid_char = has_invalid_char || parse_failed;
    let parsed_num = num.unwrap_or_default();
    let number = parsed_num.to_f64().unwrap_or_default();
    let ordinal = parsed_num.to_i32();
    let is_valid_ordinal = !has_underscore && ordinal.is_some() && !has_leading_zero;
    let ordinal = ordinal.unwrap_or(i32::MAX);
    ParsedPart {
        number,
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
            if part_i_b[0] == b'0' && part_i_b.len() > 1 {
                let integer = match &part_i_b[0..2] {
                    b"0x" | b"0X" => Some(parse_part(&part_i_b[2..], 16, |c: u8| {
                        c.is_ascii_hexdigit()
                    })),
                    b"0o" | b"0O" => Some(parse_part(&part_i_b[2..], 8, |c| {
                        (b'0'..=b'7').contains(&c)
                    })),
                    b"0b" | b"0B" => {
                        Some(parse_part(&part_i_b[2..], 2, |c| c == b'0' || c == b'1'))
                    }
                    _ => None,
                };
                if let Some(p) = integer {
                    let result = TokenKind::Number(p.number);
                    if p.has_invalid_char {
                        return Ok(TokenKind::unknown_range(
                            result,
                            range_i,
                            DiagnosticCode::InvalidNumberLiteral,
                        ));
                    }
                    if p.has_trailing_underscore {
                        return Ok(TokenKind::unknown_range(
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
                let result = handle_ordinal(part_i_b, range_i, false);
                if !result.is_unknown() {
                    return Ok(result);
                }
            }

            let bytes = part.as_bytes();

            let has_invalid_char = bytes.iter().any(|&b| b != b'_' && !is_valid_float_char(b));
            let has_underscore = bytes.iter().any(|&b| b == b'_');
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
                return Ok(TokenKind::unknown_range(
                    TokenKind::Number(parsed_num),
                    range,
                    DiagnosticCode::InvalidNumberLiteral,
                ));
            }
            if has_leading_underscore || has_trailing_underscore {
                return Ok(TokenKind::unknown_range(
                    TokenKind::Number(parsed_num),
                    range,
                    DiagnosticCode::InvalidNumberLiteralUnderscore,
                ));
            }
            Ok(TokenKind::Number(parsed_num))
        },
    )
    .parse_next(i)
}

fn handle_ordinal(bytes: &[u8], range: SourceRange, force_ordinal: bool) -> TokenKind<'_> {
    let p = parse_part(bytes, 10, |c| c.is_ascii_digit());

    let result = if p.is_valid_ordinal {
        TokenKind::Ordinal(p.ordinal)
    } else if !force_ordinal {
        TokenKind::Number(p.number)
    } else {
        TokenKind::unknown_range(
            TokenKind::Ordinal(p.ordinal),
            range.clone(),
            DiagnosticCode::InvalidOrdinalLiteral,
        )
    };
    if p.has_invalid_char {
        return TokenKind::unknown_range(result, range, DiagnosticCode::InvalidNumberLiteral);
    }
    if p.has_leading_underscore || p.has_trailing_underscore {
        return TokenKind::unknown_range(
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
        number_part
            .with_span()
            .map(|(s, r)| handle_ordinal(s.as_bytes(), r, true)),
    )
    .parse_next(i)
}
