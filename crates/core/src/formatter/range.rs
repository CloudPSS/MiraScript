use crate::parser::Range;

use super::prelude::*;

impl Formattable for Range<'_> {
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        self.0
            .format(formatter)
            .append(formatter.line_())
            .append(formatter.token(&self.1))
            .append(formatter.line_())
            .append(self.2.format(formatter))
            .group()
    }
}
