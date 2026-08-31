use crate::parser::Iterable;

use super::prelude::*;

impl Formattable for Iterable<'_> {
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        use Iterable::*;
        match self {
            Range(range) => range.format(formatter),
            Value(expression) => expression.format(formatter),
        }
    }
}
