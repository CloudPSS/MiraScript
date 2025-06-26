use crate::parser::{ParameterList, Range};

use super::prelude::*;

impl Formattable for Range<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        0
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        todo!()
    }
}
