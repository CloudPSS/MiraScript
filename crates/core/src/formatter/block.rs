use crate::{Expression, Statement};

use super::prelude::*;

impl Formattable for (&[Statement<'_>], Option<&Expression<'_>>) {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        let mut max_columns = 0;
        let mut total_lines = 0;
        for stmt in self.0 {
            let Measurement { columns, lines } = stmt.measure(formatter, columns);
            max_columns = max_columns.max(columns);
            total_lines += lines;
        }
        if let Some(expr) = self.1 {
            let Measurement { columns, lines } = expr.measure(formatter, columns);
            max_columns = max_columns.max(columns);
            total_lines += lines;
        }
        (max_columns, total_lines).into()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        for stmt in self.0 {
            stmt.format(formatter, measurement);
        }
        if let Some(expr) = self.1 {
            expr.format(formatter, measurement);
        }
    }
}
