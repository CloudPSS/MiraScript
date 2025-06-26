use crate::parser::ListItem;

use super::prelude::*;

impl<T> Formattable for [ListItem<'_, T>]
where
    T: Formattable,
{
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        0
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        let mut first = true;
        for item in self {
            if !first {
                formatter.write(", ");
            }
            first = false;
            item.format(formatter, measurement);
        }
    }
}
