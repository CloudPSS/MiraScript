use std::{fmt::Display, ops::Deref};

use crate::lexer::Token;

use super::prelude::*;

pub(super) struct Formatter<'s> {
    result: String,
    options: &'s FormatOptions,
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

impl<T> Formattable for Box<T>
where
    T: Formattable,
{
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        (**self).measure(formatter, columns)
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        (**self).format(formatter, measurement)
    }
}

const MIN_WIDTH: usize = 40;

impl<'s> Formatter<'s> {
    pub fn new(options: &'s FormatOptions, indent: usize) -> Self {
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
        if self.use_spaces {
            self.result
                .extend(std::iter::repeat(' ').take(self.indent * self.tab_size));
        } else {
            self.result
                .extend(std::iter::repeat('\t').take(self.indent));
        }
    }
    pub fn write(&mut self, s: &str) {
        self.result.push_str(s);
    }
    pub fn write_token(&mut self, s: &Token<'_>) {
        self.result.push_str(&s.to_string());
    }
    pub fn current_columns(&self) -> usize {
        let indent_width = self.indent * self.tab_size;
        std::cmp::max(MIN_WIDTH, self.line_width.saturating_sub(indent_width))
    }
}

impl<'s> Deref for Formatter<'s> {
    type Target = FormatOptions;

    fn deref(&self) -> &Self::Target {
        &self.options
    }
}
