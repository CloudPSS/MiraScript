use crate::parser::ElseBlock;

use super::prelude::*;

impl Formattable for ElseBlock<'_> {
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        formatter
            .space()
            .append(formatter.token(&self.0))
            .append(formatter.space())
            .append(self.1.format(formatter))
    }
}
