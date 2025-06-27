use std::ops::Deref;

use crate::{Operator, parser::ListItem};

use super::prelude::*;

impl<T> Formattable for [ListItem<'_, T>]
where
    T: Formattable,
{
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        let mut sum = 0;
        let mut zeros = 0;
        for item in self.iter() {
            let measurement = item.deref().measure(formatter, indent);
            if measurement == 0 {
                zeros += 1;
            } else {
                sum += measurement;
            }
        }
        if sum == 0 {
            return 0;
        }
        sum + zeros
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
