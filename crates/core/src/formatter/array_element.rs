use crate::parser::{ArrayElementBase, RecordElementBase};

use super::prelude::*;

impl<E> Formattable for ArrayElementBase<'_, E>
where
    E: Formattable,
{
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        0
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        use ArrayElementBase::*;
        match self {
            Element(e) => e.format(formatter, measurement),
            Range(range) => range.format(formatter, measurement),
            Spread(_, e) => {
                formatter.write("..");
                e.format(formatter, measurement);
            }
        }
    }
}
