use crate::parser::ParameterList;

use super::prelude::*;

impl Formattable for ParameterList<'_> {
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        formatter
            .token(&self.0)
            .append(self.1.format(formatter))
            .append(formatter.token(&self.2))
    }
}
