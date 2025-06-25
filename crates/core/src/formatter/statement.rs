use crate::{Expression, Statement};

use super::{Formattable, Formatter, Measurement};

impl Formattable for &Statement<'_> {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        todo!()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        todo!()
    }
}
