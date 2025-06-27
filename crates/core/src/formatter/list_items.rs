use std::ops::Deref;

use crate::{Operator, parser::ListItem};

use super::prelude::*;

impl<T> Formattable for [ListItem<'_, T>]
where
    T: Formattable,
{
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        0
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        for (i, item) in self.iter().enumerate() {
            item.deref().format(formatter, measurement);
            if i < self.len() - 1 {
                formatter.write_token_or(item.tail_comma(), Operator::Comma);
                formatter.write_space();
            }
        }
    }
}
