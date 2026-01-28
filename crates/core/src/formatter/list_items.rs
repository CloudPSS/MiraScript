use std::ops::Deref;

use crate::{Operator, parser::ListItem};

use super::prelude::*;

impl<T> Formattable for Vec<ListItem<'_, T>>
where
    T: Formattable,
{
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        if self.is_empty() {
            return 0;
        }
        if let [single_item] = self.as_slice() {
            // Special case: only one item, write in a single line
            return single_item.measure(formatter, indent);
        }
        let inner = self
            .iter()
            .map(|item| item.deref().measure(formatter, indent))
            .max()
            .unwrap_or(0);
        if inner == 0 {
            if self.len() > 8 { 1 } else { 0 }
        } else {
            inner + 1
        }
    }

    fn format(&self, formatter: &mut Formatter, complexity: usize) {
        if self.is_empty() {
            return;
        }
        let inner_complexity = if complexity > 0 { complexity - 1 } else { 0 };
        if let [single_item] = self.as_slice() {
            // Special case: only one item, write in a single line
            single_item.deref().format(formatter, inner_complexity);
            return;
        }
        if complexity > 0 {
            // Write one item per line
            formatter.indent();
            for item in self.iter() {
                formatter.new_line();
                item.deref().format(formatter, inner_complexity);
                formatter.write_token_or(item.tail_comma(), Operator::Comma);
            }
            formatter.dedent();
            formatter.new_line();
        } else {
            // Write items in a single line, skip tailing comma
            for (i, item) in self.iter().enumerate() {
                item.deref().format(formatter, inner_complexity);
                if i < self.len() - 1 {
                    formatter.write_token_or(item.tail_comma(), Operator::Comma);
                    formatter.write_space();
                }
            }
        }
    }
}
