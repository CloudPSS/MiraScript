use crate::parser::{Callable, ListItem};

use super::prelude::*;

impl Formattable for Callable<'_> {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        (0, 0).into()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        todo!()
    }
}
