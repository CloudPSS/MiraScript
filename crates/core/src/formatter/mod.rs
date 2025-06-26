use crate::{Script, Statement};

mod array_element;
mod block;
mod callable;
mod expression;
mod iterable;
mod list_items;
mod parameter_list;
mod pattern;
mod range;
mod record_element;
mod statement;
mod types;

mod prelude {
    pub(super) use super::FormatOptions;
    pub(super) use super::types::{Formattable, Formatter, Measurement};
}
use prelude::*;

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
            line_width: 120,
        }
    }
}

pub fn format(input: &Script<'_>, options: &FormatOptions) -> String {
    let mut formatter = Formatter::new(options, 0);
    let block = (&input.0[..], input.1.as_deref());
    let measurement = block.measure(&formatter, formatter.current_columns());
    block.format(&mut formatter, measurement);
    formatter.done()
}

pub fn format_statement(input: &Statement<'_>, options: &FormatOptions, indent: usize) -> String {
    let mut formatter = Formatter::new(options, indent);
    let measurement = input.measure(&formatter, formatter.current_columns());
    input.format(&mut formatter, measurement);
    formatter.done()
}
