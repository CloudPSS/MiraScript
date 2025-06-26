use crate::parser::{ArrayElementBase, RecordElementBase};

use super::prelude::*;

impl<E> Formattable for ArrayElementBase<'_, E>
where
    E: Formattable,
{
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        (0, 0).into()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
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
