use crate::parser::ParameterList;

use super::prelude::*;

impl Formattable for ParameterList<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        self.1.measure(formatter, indent)
    }

    fn format(&self, formatter: &mut Formatter, complexity: usize) {
        formatter.write_token(&self.0);
        if !self.1.is_empty() {
            self.1.format(formatter, complexity);
        }
        formatter.write_token(&self.2);
    }
}
