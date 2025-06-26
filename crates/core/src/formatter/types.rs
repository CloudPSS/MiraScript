use std::ops::Deref;

use crate::{
    Expression,
    lexer::{StringFragment, StringInfo, Token, TokenKind},
};

use super::prelude::*;

pub(super) struct Formatter<'o> {
    result: String,
    options: &'o FormatOptions,
    indent: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Measurement {
    pub columns: usize,
    pub lines: usize,
}

impl From<(usize, usize)> for Measurement {
    fn from((columns, lines): (usize, usize)) -> Self {
        Self { columns, lines }
    }
}

pub(super) trait Formattable {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement;
    fn format(&self, formatter: &mut Formatter, measurement: Measurement);
}

const MIN_WIDTH: usize = 40;

impl<'o> Formatter<'o> {
    pub fn new(options: &'o FormatOptions, indent: usize) -> Self {
        Self {
            result: String::new(),
            options,
            indent,
        }
    }
    pub fn done(self) -> String {
        self.result
    }

    pub fn indent(&mut self) {
        self.indent += 1;
    }
    pub fn unindent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }
    pub fn new_line(&mut self) {
        self.result.push('\n');
        let indent = if self.use_spaces {
            std::iter::repeat_n(' ', self.indent * self.tab_size)
        } else {
            std::iter::repeat_n('\t', self.indent)
        };
        self.result.extend(indent);
    }
    pub fn write(&mut self, s: &str) {
        self.result.push_str(s);
    }
    pub fn write_str_token<'s>(
        &mut self,
        info: &StringInfo<'s>,
        expressions: &[Expression<'s>],
        measurement: Measurement,
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
    pub fn write_token(&mut self, s: &Token<'_>) {
        if let TokenKind::String(_, info) = &s.kind {
            self.write_str_token(info, &[], (0, 0).into());
        } else {
            self.result.push_str(&s.to_string());
        }
    }
    pub fn current_columns(&self) -> usize {
        let indent_width = self.indent * self.tab_size;
        std::cmp::max(MIN_WIDTH, self.line_width.saturating_sub(indent_width))
    }
}

impl<'o> Deref for Formatter<'o> {
    type Target = FormatOptions;

    fn deref(&self) -> &Self::Target {
        self.options
    }
}
