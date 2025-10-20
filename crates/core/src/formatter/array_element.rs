use crate::parser::ArrayElementBase;

use super::prelude::*;

impl<E: Formattable, S: Formattable> Formattable for ArrayElementBase<'_, E, S> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        use ArrayElementBase::*;
        match self {
            Element(e) => e.measure(formatter, indent),
            Spread(_, e) => e.measure(formatter, indent),
        }
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        use ArrayElementBase::*;
        match self {
            Element(e) => e.format(formatter, measurement),
            Spread(op, e) => {
                formatter.write_token(op);
                e.format(formatter, measurement);
            }
        }
    }
}
