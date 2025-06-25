use crate::{Expression, Statement};

use super::prelude::*;

impl Formattable for &Expression<'_> {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        todo!()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        todo!()
    }
}
