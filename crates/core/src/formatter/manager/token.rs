use std::ops::Deref;

use crate::{
    Expression,
    lexer::{NumberInfo, StringFragment, Token, TokenKind},
};

use super::types::{FormatManager, Formattable};

impl Formattable for Token<'_> {
    fn measure(&self, formatter: &FormatManager, indent: usize) -> usize {
        let len = match &self.kind {
            TokenKind::InterpolatedString(parts, _) => {
                parts.iter().map(|part| part.0.len()).sum::<usize>()
            }
            _ => self.range.len(),
        };
        if len > formatter.width(indent) { 1 } else { 0 }
    }

    fn format(&self, formatter: &mut FormatManager, _: usize) {
        formatter.write_token(self);
    }
}

impl<'o> FormatManager<'o> {
    pub fn write_str_token<'s>(
        &mut self,
        s: &Token<'_>,
        expressions: &[Expression<'s, '_>],
        measurement: usize,
    ) {
        for trivia in &s.leading_trivia {
            self.write_leading_trivia(trivia);
        }
        let (TokenKind::String(_, info) | TokenKind::InterpolatedString(_, info)) = &s.kind else {
            unreachable!();
        };
        let quote = info.quote;
        let dollars = String::from_iter(std::iter::repeat_n('$', std::cmp::max(info.ats, 1)));
        if let Some(quote) = quote {
            self.write_str(&String::from_iter(std::iter::repeat_n('@', info.ats)));
            self.write_str(&quote.to_string());
        }
        let mut exprs = expressions.iter();
        for str in info.content.iter() {
            match str {
                StringFragment::Literal(text) => self.write_str(text),
                StringFragment::EscapedChar(_, text) => {
                    self.write_str("\\");
                    self.write_str(&text[0..1]);
                    self.write_str(&text[1..].to_ascii_uppercase());
                }
                StringFragment::Interpolation(_, _, fmt, surround) => {
                    let surround = surround.as_deref();
                    self.write_str(&dollars);
                    if let Some((start, _)) = surround {
                        self.write_str(&start.to_string());
                    }
                    if let Some(e) = exprs.next() {
                        e.format(self, measurement);
                    }
                    if !fmt.is_empty() {
                        self.write_str(":");
                        self.write_str(fmt);
                    }
                    if let Some((_, end)) = surround {
                        self.write_str(&end.to_string());
                    }
                }
                StringFragment::InvalidEscapedChar(_, _)
                | StringFragment::EndOfString
                | StringFragment::EndOfFile => (),
            }
        }

        if let Some(quote) = quote {
            self.write_str(&quote.to_string());
            self.write_str(&String::from_iter(std::iter::repeat_n('@', info.ats)));
        }

        for trivia in &s.tailing_trivia {
            self.write_tailing_trivia(trivia);
        }
    }

    fn write_num_part(&mut self, s: &str, group_size: usize, min_size: usize, right_to_left: bool) {
        debug_assert!(
            s.is_ascii(),
            "Expected ASCII string for underscore insertion, got: {}",
            s
        );

        if group_size == 0 || s.len() <= group_size {
            return self.write_str(s);
        }
        if min_size != 0 && s.len() < min_size {
            return self.write_str(s);
        }

        let bytes = s.as_bytes();

        if right_to_left {
            let len = bytes.len();
            for (i, &b) in bytes.iter().enumerate() {
                if i > 0 && (len - i).is_multiple_of(group_size) {
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

    fn write_token_body(&mut self, s: &TokenKind<'_>) {
        match s {
            TokenKind::String(_, _) | TokenKind::InterpolatedString(_, _) => {
                unreachable!();
            }
            TokenKind::Identifier(s) => self.write_str(s),
            TokenKind::Ordinal(s) => self.write_str(&s.to_string()),
            TokenKind::Number(s, i) => match i.as_ref() {
                NumberInfo::Invalid => self.write_str(&format!("{:e}", s)),
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
                        self.write_str(".");
                        self.write_num_part(fractional_part, 3, 5, false);
                    }
                    if !exponent_part.is_empty() {
                        self.write_str("e");
                        self.write_str(exponent_part);
                    }
                }
                NumberInfo::Hexadecimal(v) => {
                    self.write_str("0x");
                    self.write_num_part(&format!("{:X}", v), 4, 0, true);
                }
                NumberInfo::Octal(v) => {
                    self.write_str("0o");
                    self.write_num_part(&format!("{:o}", v), 6, 0, true);
                }
                NumberInfo::Binary(v) => {
                    self.write_str("0b");
                    self.write_num_part(&format!("{:b}", v), 8, 0, true);
                }
            },
            TokenKind::Operator(operator) => self.write_str(&operator.to_string()),
            TokenKind::Keyword(keyword) => self.write_str(keyword.into()),
            TokenKind::Eof => (),
            TokenKind::Empty | TokenKind::Unknown { .. } => (),
        }
    }

    pub fn write_token(&mut self, s: &Token<'_>) {
        if matches!(
            s.kind,
            TokenKind::String(_, _) | TokenKind::InterpolatedString(_, _)
        ) {
            return self.write_str_token(s, &[], 0);
        }
        for trivia in &s.leading_trivia {
            self.write_leading_trivia(trivia);
        }
        self.write_token_body(&s.kind);
        for trivia in &s.tailing_trivia {
            self.write_tailing_trivia(trivia);
        }
    }

    pub fn write_token_or<'s, T, F>(&mut self, s: Option<T>, fallback: F)
    where
        T: Deref<Target = Token<'s>>,
        F: Into<TokenKind<'s>>,
    {
        if let Some(s) = s.as_deref() {
            self.write_token(s);
        } else {
            self.write_token_body(&fallback.into());
        }
    }
}
