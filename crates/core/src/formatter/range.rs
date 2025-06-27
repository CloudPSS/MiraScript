use crate::parser::Range;

use super::prelude::*;

impl Formattable for Range<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        self.0.measure(formatter, indent) + self.2.measure(formatter, indent)
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        self.0.format(formatter, measurement);
        formatter.write_token(&self.1);
        self.2.format(formatter, measurement);
    }
}
