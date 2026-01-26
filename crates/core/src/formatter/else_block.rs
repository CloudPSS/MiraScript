use crate::parser::ElseBlock;

use super::prelude::*;

impl Formattable for ElseBlock<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        self.1.measure(formatter, indent)
    }

    fn format(&self, formatter: &mut Formatter, complexity: usize) {
        formatter.write_space();
        formatter.write_token(&self.0);
        formatter.write_space();
        self.1.format(formatter, complexity);
    }
}
