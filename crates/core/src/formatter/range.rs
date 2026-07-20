use crate::parser::Range;

use super::prelude::*;

impl Formattable for Range<'_, '_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        let inner = usize::max(
            self.0.measure(formatter, indent),
            self.2.measure(formatter, indent),
        );
        if inner > 0 { inner + 1 } else { 0 }
    }

    fn format(&self, formatter: &mut Formatter, complexity: usize) {
        let inner_complexity = if complexity > 0 { complexity - 1 } else { 0 };
        self.0.format(formatter, inner_complexity);
        if complexity > 0 {
            formatter.new_line();
        }
        formatter.write_token(&self.1);
        if complexity > 0 {
            formatter.new_line();
        }
        self.2.format(formatter, inner_complexity);
    }
}
