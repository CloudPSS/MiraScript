use crate::parser::RecordElementBase;

use super::prelude::*;

impl<E, I> Formattable for RecordElementBase<'_, '_, E, I>
where
    E: Formattable,
    I: Formattable,
{
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        use RecordElementBase::*;
        match self {
            InterpolateNamed(_, _, e)
            | Named(_, _, e)
            | OmitNamed(_, e)
            | Unnamed(e)
            | Spread(_, e) => e.measure(formatter, indent),
        }
    }

    fn format(&self, formatter: &mut Formatter, complexity: usize) {
        use RecordElementBase::*;
        match self {
            Named(name, colon, e) => {
                formatter.write_token(name);
                formatter.write_token(colon);
                formatter.write_space();
                e.format(formatter, complexity);
            }
            InterpolateNamed(i, colon, e) => {
                i.format(formatter, complexity);
                formatter.write_token(colon);
                formatter.write_space();
                e.format(formatter, complexity);
            }
            OmitNamed(colon, e) => {
                formatter.write_token(colon);
                e.format(formatter, complexity);
            }
            Unnamed(e) => {
                e.format(formatter, complexity);
            }
            Spread(op, e) => {
                formatter.write_token(op);
                e.format(formatter, complexity);
            }
        }
    }
}
