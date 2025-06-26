use crate::parser::ListItem;

use super::prelude::*;

impl<T> Formattable for [ListItem<'_, T>]
where
    T: Formattable,
{
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        (0, 0).into()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        let mut first = true;
        for item in self {
            if !first {
                formatter.write(",");
            }
            first = false;
            item.format(formatter, measurement);
        }
    }
}
