use crate::parser::{ParameterList, Range};

use super::prelude::*;

impl Formattable for Range<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        0
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        self.0.format(formatter, measurement);
        formatter.write_token(&self.1);
        self.2.format(formatter, measurement);
    }
}
