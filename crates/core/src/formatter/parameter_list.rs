use crate::{Operator, parser::ParameterList};

use super::prelude::*;

impl Formattable for ParameterList<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        0
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        formatter.write_token(&self.0);
        if !self.1.is_empty() {
            formatter.indent();
            for (i, item) in self.1.iter().enumerate() {
                item.format(formatter, measurement);
                if i < self.1.len() - 1 {
                    formatter.write_token_or(item.tail_comma(), Operator::Comma);
                    formatter.write_space();
                }
            }
            formatter.unindent();
        }
        formatter.write_token(&self.2);
    }
}
