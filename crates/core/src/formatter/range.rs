use crate::parser::{ParameterList, Range};

use super::prelude::*;

impl Formattable for Range<'_> {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        (0, 0).into()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        todo!()
    }
}
