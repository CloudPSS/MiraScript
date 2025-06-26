use crate::parser::{Callable, Iterable, ListItem};

use super::prelude::*;

impl Formattable for Iterable<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        0
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        use Iterable::*;
        match self {
            Range(range) => {
                range.format(formatter, measurement);
            }
            Value(expression) => {
                expression.format(formatter, measurement);
            }
        }
    }
}
