use crate::parser::{Callable, Iterable, ListItem, RecordElementBase};

use super::prelude::*;

impl<E, I> Formattable for RecordElementBase<'_, E, I>
where
    E: Formattable,
    I: Formattable,
{
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        0
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        use RecordElementBase::*;
        match self {
            Named(name, colon, e) => {
                formatter.write_token(name);
                formatter.write_token(colon);
                formatter.write(" ");
                e.format(formatter, measurement);
            }
            InterpolateNamed(i, colon, e) => {
                i.format(formatter, measurement);
                formatter.write_token(colon);
                formatter.write(" ");
                e.format(formatter, measurement);
            }
            OmitNamed(colon, e) => {
                formatter.write_token(colon);
                e.format(formatter, measurement);
            }
            Unnamed(e) => {
                e.format(formatter, measurement);
            }
            Spread(_, e) => {
                formatter.write("..");
                e.format(formatter, measurement);
            }
        }
    }
}
