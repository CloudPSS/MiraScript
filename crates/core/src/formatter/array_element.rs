use crate::parser::ArrayElementBase;

use super::prelude::*;

impl<E: Formattable, S: Formattable> Formattable for ArrayElementBase<'_, E, S> {
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        use ArrayElementBase::*;
        match self {
            Element(e) => e.format(formatter),
            Spread(op, e) => formatter.token(op).append(e.format(formatter)),
        }
    }
}
