use crate::Pattern;

use super::prelude::*;

impl Formattable for Pattern<'_> {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        (0, 0).into()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        use Pattern::*;
        match self {
            Grouping(_, pattern, _) => {
                formatter.write("(");
                pattern.format(formatter, measurement);
                formatter.write(")");
            }
            Constant(op, literal) => {
                if let Some(op) = op {
                    formatter.write_token(op);
                }
                formatter.write_token(literal);
            }
            Relation(op, constant) => {
                formatter.write_token(op);
                formatter.write(" ");
                constant.format(formatter, measurement);
            }
            Range(left, op, right) => {
                left.format(formatter, measurement);
                formatter.write_token(op);
                right.format(formatter, measurement);
            }
            Discard(_) => {
                formatter.write("_");
            }
            Bind(kw_mut, id) => {
                if let Some(kw_mut) = kw_mut {
                    formatter.write_token(kw_mut);
                    formatter.write(" ");
                }
                formatter.write_token(id);
            }
            Record(_, list_items, _) => {
                formatter.write("(");
                list_items[..].format(formatter, measurement);
                formatter.write(")");
            }
            Array(_, list_items, _) => {
                formatter.write("[");
                list_items[..].format(formatter, measurement);
                formatter.write("]");
            }
            SpreadDiscard(_) => (),
            And(left, _, right) => {
                left.format(formatter, measurement);
                formatter.write(" and ");
                right.format(formatter, measurement);
            }
            Or(left, _, right) => {
                left.format(formatter, measurement);
                formatter.write(" or ");
                right.format(formatter, measurement);
            }
            Not(_, pattern) => {
                formatter.write("not ");
                pattern.format(formatter, measurement);
            }
            Unknown { .. } => (),
        }
    }
}
