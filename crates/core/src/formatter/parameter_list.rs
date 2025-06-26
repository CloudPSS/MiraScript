use crate::parser::ParameterList;

use super::prelude::*;

impl Formattable for ParameterList<'_> {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        (0, 0).into()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        todo!()
    }
}
