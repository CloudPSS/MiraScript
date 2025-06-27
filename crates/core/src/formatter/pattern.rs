use crate::{Operator, Pattern};

use super::prelude::*;

impl Formattable for Pattern<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        use Pattern::*;
        match self {
            Grouping(_, pattern, _) => pattern.measure(formatter, indent),
            Constant(_, literal) => literal.measure(formatter, indent),

            _ => 0,
        }
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        use Pattern::*;
        match self {
            Grouping(op, pattern, cp) => {
                formatter.write_token(op);
                pattern.format(formatter, measurement);
                formatter.write_token(cp);
            }
            Constant(op, literal) => {
                if let Some(op) = op {
                    formatter.write_token(op);
                }
                formatter.write_token(literal);
            }
            Relation(op, constant) => {
                formatter.write_token(op);
                formatter.write_space();
                constant.format(formatter, measurement);
            }
            Range(left, op, right) => {
                left.format(formatter, measurement);
                formatter.write_token(op);
                right.format(formatter, measurement);
            }
            Discard(kw) => {
                formatter.write_token(kw);
            }
            Bind(kw_mut, id) => {
                if let Some(kw_mut) = kw_mut {
                    formatter.write_token(kw_mut);
                    formatter.write_space();
                }
                formatter.write_token(id);
            }
            Record(op, list_items, cp) => {
                formatter.write_token(op);
                list_items[..].format(formatter, measurement);
                if list_items.len() == 1 && list_items[0].is_unnamed() {
                    formatter.write_token_or(list_items[0].tail_comma(), Operator::Comma);
                }
                formatter.write_token(cp);
            }
            Array(op, list_items, cp) => {
                formatter.write_token(op);
                list_items[..].format(formatter, measurement);
                formatter.write_token(cp);
            }
            SpreadDiscard(_) => (),
            And(left, kw, right) | Or(left, kw, right) => {
                left.format(formatter, measurement);
                formatter.write_space();
                formatter.write_token(kw);
                formatter.write_space();
                right.format(formatter, measurement);
            }
            Not(kw, pattern) => {
                formatter.write_token(kw);
                formatter.write_space();
                pattern.format(formatter, measurement);
            }
            Unknown { .. } => (),
        }
    }
}
