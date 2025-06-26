use crate::parser::ParameterList;

use super::prelude::*;

impl Formattable for ParameterList<'_> {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        (0, 0).into()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        formatter.write("(");
        if !self.1.is_empty() {
            formatter.indent();
            for (i, item) in self.1.iter().enumerate() {
                item.format(formatter, measurement);
                if i < self.1.len() - 1 {
                    formatter.write(", ");
                }
            }
            formatter.unindent();
        }
        formatter.write(")");
    }
}
