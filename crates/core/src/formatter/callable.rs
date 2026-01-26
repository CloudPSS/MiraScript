use crate::parser::Callable;

use super::prelude::*;

impl Formattable for Callable<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        use Callable::*;
        match self {
            Type(_) => 0,
            Expression(expression) => expression.measure(formatter, indent),
        }
    }

    fn format(&self, formatter: &mut Formatter, complexity: usize) {
        use Callable::*;
        match self {
            Type(kw) => formatter.write_token(kw),
            Expression(expression) => expression.format(formatter, complexity),
        }
    }
}
