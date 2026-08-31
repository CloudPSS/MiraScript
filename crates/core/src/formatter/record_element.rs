use crate::parser::RecordElementBase;

use super::prelude::*;

impl<E, I> Formattable for RecordElementBase<'_, E, I>
where
    E: Formattable,
    I: Formattable,
{
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        use RecordElementBase::*;
        match self {
            Named(name, colon, e) => formatter
                .token(name)
                .append(formatter.token(colon))
                .append(formatter.space())
                .append(e.format(formatter)),
            InterpolateNamed(i, colon, e) => i
                .format(formatter)
                .append(formatter.token(colon))
                .append(formatter.space())
                .append(e.format(formatter)),
            OmitNamed(colon, e) => formatter.token(colon).append(e.format(formatter)),
            Unnamed(e) => e.format(formatter),
            Spread(op, e) => formatter.token(op).append(e.format(formatter)),
        }
    }
}
