use crate::parser::ArrayElementBase;

use super::prelude::*;

impl<E: Formattable, S: Formattable> Formattable for ArrayElementBase<'_, '_, E, S> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        use ArrayElementBase::*;
        match self {
            Element(e) => e.measure(formatter, indent),
            Spread(_, e) => e.measure(formatter, indent),
        }
    }

    fn format(&self, formatter: &mut Formatter, complexity: usize) {
        use ArrayElementBase::*;
        match self {
            Element(e) => e.format(formatter, complexity),
            Spread(op, e) => {
                formatter.write_token(op);
                e.format(formatter, complexity);
            }
        }
    }
}
