use crate::parser::Iterable;

use super::prelude::*;

impl Formattable for Iterable<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        use Iterable::*;
        match self {
            Range(range) => range.measure(formatter, indent),
            Value(expression) => expression.measure(formatter, indent),
        }
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
