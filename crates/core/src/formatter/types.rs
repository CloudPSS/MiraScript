use std::ops::Deref;

use crate::{
    Expression,
    lexer::{StringFragment, StringInfo, Token, TokenKind},
};

use super::prelude::*;

pub(super) struct Formatter<'o> {
    result: String,
    line_count: usize,
    options: &'o FormatOptions,
    indent: usize,
}

pub(super) trait Formattable {
    /// Measure the number of lines of the formatted output.
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize;
    /// Format the output into the provided formatter.
    /// The `measurement` parameter is the number of lines that currently fit within the available space.
    fn format(&self, formatter: &mut Formatter, measurement: usize);
}

const MIN_WIDTH: usize = 40;

impl<'o> Formatter<'o> {
    pub fn new(options: &'o FormatOptions, indent: usize) -> Self {
        Self {
            result: String::new(),
            line_count: 0,
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
        self.line_count += 1;
    }
    pub fn line_count(&self) -> usize {
        self.line_count
    }
    pub fn write(&mut self, s: &str) {
        self.result.push_str(s);
    }
    pub fn write_char(&mut self, c: char) {
        self.result.push(c);
    }
    pub fn width(&self, indent: usize) -> usize {
        let indent_width = indent * self.tab_size;
        std::cmp::max(MIN_WIDTH, self.line_width.saturating_sub(indent_width))
    }
}

impl<'o> Deref for Formatter<'o> {
    type Target = FormatOptions;

    fn deref(&self) -> &Self::Target {
        self.options
    }
}
