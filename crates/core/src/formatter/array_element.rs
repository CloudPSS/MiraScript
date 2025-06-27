use crate::parser::ArrayElementBase;

use super::prelude::*;

impl<E> Formattable for ArrayElementBase<'_, E>
where
    E: Formattable,
{
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        use ArrayElementBase::*;
        match self {
            Element(e) => e.measure(formatter, indent),
            Range(range) => range.measure(formatter, indent),
            Spread(_, e) => e.measure(formatter, indent),
        }
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        use ArrayElementBase::*;
        match self {
            Element(e) => e.format(formatter, measurement),
            Range(range) => range.format(formatter, measurement),
            Spread(op, e) => {
                formatter.write_token(op);
                e.format(formatter, measurement);
            }
        }
    }
}
