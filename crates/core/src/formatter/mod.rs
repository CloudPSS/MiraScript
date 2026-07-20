mod array_element;
mod callable;
mod else_block;
mod expression;
mod iterable;
mod list_items;
mod manager;
mod parameter_list;
mod pattern;
mod range;
mod record_element;
mod statement;

mod prelude {
    pub(super) use super::manager::{FormatManager as Formatter, Formattable};
}

use crate::{Script, Statement};

use prelude::*;

#[derive(Debug, Clone)]
pub struct FormatOptions {
    pub tab_size: usize,
    pub use_spaces: bool,
    pub line_width: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            tab_size: 2,
            use_spaces: true,
            line_width: 80,
        }
    }
}

pub fn format(input: &Script<'_, '_>, options: &FormatOptions) -> String {
    let Script(stmts, expr, eof) = input;
    let mut formatter = Formatter::new(options, 0);
    for statement in stmts {
        formatter.format(statement);
        formatter.new_line();
    }
    if let Some(expression) = expr {
        formatter.format(expression);
    }

    let end_empty_lines = eof
        .leading_trivia
        .iter()
        .rev()
        .take_while(|trivia| trivia.is_new_line())
        .count();
    if end_empty_lines != eof.leading_trivia.len() {
        if expr.is_some() {
            formatter.new_line();
        }
        for trivia in &eof.leading_trivia[..eof.leading_trivia.len() - end_empty_lines] {
            formatter.write_leading_trivia(trivia);
        }
    } else {
        formatter.new_line();
    }
    formatter.done()
}

pub fn format_statement(input: &Statement<'_, '_>, options: &FormatOptions, indent: usize) -> String {
    let mut formatter = Formatter::new(options, indent);
    let measurement = input.measure(&formatter, indent);
    input.format(&mut formatter, measurement);
    formatter.done()
}
