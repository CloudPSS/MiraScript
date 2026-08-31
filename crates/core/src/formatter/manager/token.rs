use std::ops::Deref;

use crate::{
    lexer::{NumberInfo, StringFragment, Token, TokenKind},
    parser::Expression,
};

use super::{FormatDoc, FormatManager, Formattable};

impl Formattable for Token<'_> {
    fn format(&self, formatter: &FormatManager) -> FormatDoc {
        formatter.token(self)
    }
}

fn grouped_digits(s: &str, group_size: usize, min_size: usize, right_to_left: bool) -> String {
    if group_size == 0 || s.len() <= group_size || (min_size != 0 && s.len() < min_size) {
        return s.to_owned();
    }
    let mut output = String::with_capacity(s.len() + s.len() / group_size);
    for (index, byte) in s.bytes().enumerate() {
        let insert = if right_to_left {
            index > 0 && (s.len() - index).is_multiple_of(group_size)
        } else {
            index > 0 && index.is_multiple_of(group_size)
        };
        if insert {
            output.push('_');
        }
        output.push(byte as char);
    }
    output
}

fn normalized_number(value: f64, info: &NumberInfo<'_>) -> String {
    match info {
        NumberInfo::Invalid => format!("{value:e}"),
        NumberInfo::Decimal(source) => {
            let source = source.replace('_', "");
            let (mantissa, exponent) = source
                .find(['e', 'E'])
                .map_or((source.as_str(), None), |index| {
                    (&source[..index], Some(&source[index + 1..]))
                });
            let (integer, fraction) = mantissa.find('.').map_or((mantissa, None), |index| {
                (&mantissa[..index], Some(&mantissa[index + 1..]))
            });
            let mut output = grouped_digits(integer, 3, 5, true);
            if let Some(fraction) = fraction.filter(|fraction| !fraction.is_empty()) {
                output.push('.');
                output.push_str(&grouped_digits(fraction, 3, 5, false));
            }
            if let Some(exponent) = exponent.filter(|exponent| !exponent.is_empty()) {
                output.push('e');
                output.push_str(exponent);
            }
            output
        }
        NumberInfo::Hexadecimal(value) => {
            format!("0x{}", grouped_digits(&format!("{value:X}"), 4, 0, true))
        }
        NumberInfo::Octal(value) => {
            format!("0o{}", grouped_digits(&format!("{value:o}"), 6, 0, true))
        }
        NumberInfo::Binary(value) => {
            format!("0b{}", grouped_digits(&format!("{value:b}"), 8, 0, true))
        }
    }
}

impl FormatManager<'_> {
    pub fn string_token(&self, token: &Token<'_>, expressions: &[Expression<'_>]) -> FormatDoc {
        let (TokenKind::String(_, info) | TokenKind::InterpolatedString(_, info)) = &token.kind
        else {
            unreachable!();
        };
        let mut docs = vec![self.leading_trivia(&token.leading_trivia)];
        let dollars = "$".repeat(info.ats.max(1));
        if let Some(quote) = info.quote {
            docs.push(self.text("@".repeat(info.ats)));
            docs.push(self.text(quote.to_string()));
        }
        let mut expressions = expressions.iter();
        let mut line_indent = 0;
        let mut at_line_start = true;
        for fragment in info.content.iter() {
            match fragment {
                StringFragment::Literal(text) => {
                    docs.push(self.source_text(text));
                    update_line_indent(
                        text,
                        self.options.tab_size,
                        &mut line_indent,
                        &mut at_line_start,
                    );
                }
                StringFragment::EscapedChar(_, text) => {
                    docs.push(self.text(format!(
                        "\\{}{}",
                        &text[..1],
                        text[1..].to_ascii_uppercase()
                    )));
                }
                StringFragment::Interpolation(_, _, format, surround) => {
                    docs.push(self.text(dollars.clone()));
                    if let Some((start, _)) = surround.as_deref() {
                        docs.push(self.text(start.to_string()));
                    }
                    if let Some(expression) = expressions.next() {
                        docs.push(expression.format(self).nest(line_indent as isize));
                    }
                    if !format.is_empty() {
                        docs.push(self.text(format!(":{format}")));
                    }
                    if let Some((_, end)) = surround.as_deref() {
                        docs.push(self.text(end.to_string()));
                    }
                }
                StringFragment::InvalidEscapedChar(_, _)
                | StringFragment::EndOfString
                | StringFragment::EndOfFile => {}
            }
        }
        if let Some(quote) = info.quote {
            docs.push(self.text(quote.to_string()));
            docs.push(self.text("@".repeat(info.ats)));
        }
        docs.push(self.tailing_trivia(&token.tailing_trivia));
        self.concat(docs)
    }

    pub fn token_body(&self, kind: &TokenKind<'_>) -> FormatDoc {
        let text = match kind {
            TokenKind::String(_, _) | TokenKind::InterpolatedString(_, _) => unreachable!(),
            TokenKind::Identifier(value) => (*value).to_owned(),
            TokenKind::Ordinal(value) => value.to_string(),
            TokenKind::Number(value, info) => normalized_number(*value, info),
            TokenKind::Operator(operator) => operator.to_string(),
            TokenKind::Keyword(keyword) => {
                let keyword: &str = keyword.into();
                keyword.to_owned()
            }
            TokenKind::Eof | TokenKind::Empty | TokenKind::Unknown { .. } => String::new(),
        };
        self.text(text)
    }

    pub fn token(&self, token: &Token<'_>) -> FormatDoc {
        if matches!(
            token.kind,
            TokenKind::String(_, _) | TokenKind::InterpolatedString(_, _)
        ) {
            return self.string_token(token, &[]);
        }
        self.leading_trivia(&token.leading_trivia)
            .append(self.token_body(&token.kind))
            .append(self.tailing_trivia(&token.tailing_trivia))
    }

    pub fn token_without_leading(&self, token: &Token<'_>) -> FormatDoc {
        self.token_body(&token.kind)
            .append(self.tailing_trivia(&token.tailing_trivia))
    }

    pub fn token_or<'s, T, F>(&self, token: Option<T>, fallback: F) -> FormatDoc
    where
        T: Deref<Target = Token<'s>>,
        F: Into<TokenKind<'s>>,
    {
        token.as_deref().map_or_else(
            || self.token_body(&fallback.into()),
            |token| self.token(token),
        )
    }
}

fn update_line_indent(
    text: &str,
    tab_size: usize,
    line_indent: &mut usize,
    at_line_start: &mut bool,
) {
    for character in text.chars() {
        match character {
            '\r' | '\n' => {
                *line_indent = 0;
                *at_line_start = true;
            }
            ' ' if *at_line_start => *line_indent += 1,
            '\t' if *at_line_start => *line_indent += tab_size,
            _ => *at_line_start = false,
        }
    }
}
