use std::ops::Deref;

use crate::formatter::FormatOptions;

#[derive(Debug)]
pub(crate) struct FormatManager<'o> {
    pub(super) result: String,
    pub(super) current_indent: usize,
    pub(super) current_line: String,
    pub(super) current_leading_trivia: String,
    pub(super) current_tailing_trivia: String,
    pub(super) line_count: usize,
    pub(super) options: &'o FormatOptions,
    pub(super) indent: usize,
}

pub(crate) trait Formattable {
    /// Measure the number of lines of the formatted output.
    fn measure(&self, formatter: &FormatManager, indent: usize) -> usize;
    /// Format the output into the provided formatter.
    /// The `measurement` parameter is the number of lines that currently fit within the available space.
    fn format(&self, formatter: &mut FormatManager, measurement: usize);
}

const MIN_WIDTH: usize = 40;

impl<'o> FormatManager<'o> {
    pub fn new(options: &'o FormatOptions, indent: usize) -> Self {
        Self {
            result: String::new(),
            current_indent: indent,
            current_line: String::new(),
            current_leading_trivia: String::new(),
            current_tailing_trivia: String::new(),
            line_count: 0,
            options,
            indent,
        }
    }
    pub fn done(mut self) -> String {
        if !self.current_leading_trivia.is_empty()
            || !self.current_line.is_empty()
            || !self.current_tailing_trivia.is_empty()
        {
            self.new_line();
        }
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
    pub fn measure(&self, item: &impl Formattable) -> usize {
        item.measure(self, self.indent)
    }

    pub(super) fn make_indent(&self, indent: usize) -> std::iter::RepeatN<char> {
        if self.use_spaces {
            std::iter::repeat_n(' ', indent * self.tab_size)
        } else {
            std::iter::repeat_n('\t', indent)
        }
    }
    pub fn new_line(&mut self) {
        let indent = self.make_indent(self.current_indent);
        if !self.result.is_empty() {
            self.result.push('\n');
        }

        let current_leading_trivia = std::mem::take(&mut self.current_leading_trivia);
        if !current_leading_trivia.is_empty() {
            for line in current_leading_trivia.lines() {
                if !line.is_empty() {
                    self.result.extend(indent.clone());
                    self.result.push_str(line);
                }
                self.result.push('\n');
            }
        }

        let current_line = std::mem::take(&mut self.current_line);
        if !current_line.is_empty() {
            self.result.extend(indent.clone());
            self.result.push_str(&current_line);
        }

        let current_tailing_trivia = std::mem::take(&mut self.current_tailing_trivia);
        if !current_tailing_trivia.is_empty() {
            if !current_line.is_empty() {
                self.result.push(' ');
            }
            self.result.push_str(&current_tailing_trivia);
        }

        self.line_count += 1;
        self.current_indent = self.indent;
    }

    pub fn line_count(&self) -> usize {
        self.line_count
    }

    #[inline]
    pub fn write_space(&mut self) {
        self.write_spaces::<1>();
    }
    #[inline]
    pub fn write_spaces<const COUNT: usize>(&mut self) {
        if !self.current_line.is_empty() {
            self.current_line.extend(std::iter::repeat_n(' ', COUNT));
        }
    }
    pub(super) fn write_str(&mut self, s: &str) {
        self.current_line.push_str(s);
    }
    pub(super) fn write_char(&mut self, c: char) {
        self.current_line.push(c);
    }
    pub fn width(&self, indent: usize) -> usize {
        let indent_width = indent * self.tab_size;
        std::cmp::max(MIN_WIDTH, self.line_width.saturating_sub(indent_width))
    }
}

impl<'o> Deref for FormatManager<'o> {
    type Target = FormatOptions;

    fn deref(&self) -> &Self::Target {
        self.options
    }
}
