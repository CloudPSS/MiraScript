use crate::{
    Expression,
    lexer::{NumberInfo, StringFragment, StringInfo, Token, TokenKind},
};

use super::prelude::*;

impl<'o> Formatter<'o> {
    pub fn write_str_token<'s>(
        &mut self,
        info: &StringInfo<'s>,
        expressions: &[Expression<'s>],
        measurement: usize,
    ) {
        let quote = info.quote;
        let dollars = String::from_iter(std::iter::repeat_n('$', std::cmp::max(info.ats, 1)));
        if let Some(quote) = quote {
            self.write(&String::from_iter(std::iter::repeat_n('@', info.ats)));
            self.write(&quote.to_string());
        }
        let mut exprs = expressions.iter();
        for str in info.content.iter() {
            match str {
                StringFragment::Literal(text) => self.write(text),
                StringFragment::EscapedChar(_, text) => {
                    self.write("\\");
                    self.write(&text[0..1]);
                    self.write(&text[1..].to_ascii_uppercase());
                }
                StringFragment::Interpolation(_, _, surround) => {
                    let surround = surround.as_deref();
                    self.write(&dollars);
                    if let Some((start, _)) = surround {
                        self.write(&start.to_string());
                    }
                    if let Some(e) = exprs.next() {
                        e.format(self, measurement);
                    }
                    if let Some((_, end)) = surround {
                        self.write(&end.to_string());
                    }
                }
                StringFragment::InvalidEscapedChar(_, _)
                | StringFragment::EndOfString
                | StringFragment::EndOfFile => (),
            }
        }

        if let Some(quote) = quote {
            self.write(&quote.to_string());
            self.write(&String::from_iter(std::iter::repeat_n('@', info.ats)));
        }
    }

    fn write_num_part(&mut self, s: &str, group_size: usize, min_size: usize, right_to_left: bool) {
        debug_assert!(
            s.is_ascii(),
            "Expected ASCII string for underscore insertion, got: {}",
            s
        );

        if group_size == 0 || s.len() <= group_size {
            return self.write(s);
        }
        if min_size != 0 && s.len() < min_size {
            return self.write(s);
        }

        let bytes = s.as_bytes();

        if right_to_left {
            let len = bytes.len();
            for (i, &b) in bytes.iter().enumerate() {
                if i > 0 && (len - i) % group_size == 0 {
                    self.write_char('_');
                }
                self.write_char(b as char);
            }
        } else {
            for (i, &b) in bytes.iter().enumerate() {
                if i > 0 && i % group_size == 0 {
                    self.write_char('_');
                }
                self.write_char(b as char);
            }
        }
    }

    pub fn write_token(&mut self, s: &Token<'_>) {
        match &s.kind {
            TokenKind::String(_, info) | TokenKind::InterpolatedString(_, info) => {
                self.write_str_token(info, &[], 0);
            }
            TokenKind::Identifier(s) => self.write(s),
            TokenKind::Ordinal(s) => self.write(&s.to_string()),
            TokenKind::Number(s, i) => match i.as_ref() {
                NumberInfo::Invalid => self.write(&format!("{:e}", s)),
                NumberInfo::Decimal(s) => {
                    let s = if s.find('_').is_some() {
                        s.chars().filter(|c| *c != '_').collect::<String>()
                    } else {
                        s.to_string()
                    };

                    let (integer_part, fractional_part, exponent_part) =
                        match (s.find(['e', 'E']), s.find('.')) {
                            (Some(e), Some(d)) => (&s[..d], &s[d + 1..e], &s[e + 1..]),
                            (Some(e), None) => (&s[..e], "", &s[e + 1..]),
                            (None, Some(d)) => (&s[..d], &s[d + 1..], ""),
                            (None, None) => (s.as_ref(), "", ""),
                        };
                    self.write_num_part(integer_part, 3, 5, true);
                    if !fractional_part.is_empty() {
                        self.write(".");
                        self.write_num_part(fractional_part, 3, 5, false);
                    }
                    if !exponent_part.is_empty() {
                        self.write("e");
                        self.write(exponent_part);
                    }
                }
                NumberInfo::Hexadecimal(v) => {
                    self.write("0x");
                    self.write_num_part(&format!("{:X}", v), 4, 0, true);
                }
                NumberInfo::Octal(v) => {
                    self.write("0o");
                    self.write_num_part(&format!("{:o}", v), 6, 0, true);
                }
                NumberInfo::Binary(v) => {
                    self.write("0b");
                    self.write_num_part(&format!("{:b}", v), 8, 0, true);
                }
            },
            TokenKind::Operator(operator) => self.write(&operator.to_string()),
            TokenKind::Keyword(keyword) => self.write(keyword.into()),
            TokenKind::Eof => (),
            TokenKind::Empty | TokenKind::Unknown { .. } => (),
        }
    }
}
