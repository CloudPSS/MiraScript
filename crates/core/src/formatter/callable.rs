use crate::parser::Callable;

use super::prelude::*;

impl Formattable for Callable<'_> {
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        use Callable::*;
        match self {
            Type(kw) => formatter.token(kw),
            Expression(expression) => expression.format(formatter),
        }
    }
}
