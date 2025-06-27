use crate::parser::{Callable, ListItem};

use super::prelude::*;

impl Formattable for Callable<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        0
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        use Callable::*;
        match self {
            Type(kw) => formatter.write_token(kw),
            Expression(expression) => expression.format(formatter, measurement),
        }
    }
}
