use crate::Statement;

use super::{Formattable, Formatter, Measurement};

impl Formattable for Statement<'_> {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        (1, 1).into()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        use Statement::*;
        match self {
            Empty(_) => formatter.write(";"),
            Expression(expression, _) => {
                expression.format(formatter, measurement);
                formatter.write(";");
            }
            BlockExpression(expression) => expression.format(formatter, measurement),
            Bind(_, pattern, _, expression, _) => {
                formatter.write("let ");
                pattern.format(formatter, measurement);
                formatter.write(" = ");
                expression.format(formatter, measurement);
                formatter.write(";");
            }
            Rebind(pattern, _, expression, _) => {
                pattern.format(formatter, measurement);
                formatter.write(" = ");
                expression.format(formatter, measurement);
                formatter.write(";");
            }
            Assign(assignee, op, expression, _) => {
                assignee.format(formatter, measurement);
                formatter.write(" ");
                formatter.write_token(op);
                formatter.write(" ");
                expression.format(formatter, measurement);
                formatter.write(";");
            }
            Function(_, id, parameter_list, expression) => {
                formatter.write("fn ");
                formatter.write_token(id);
                if let Some(param) = parameter_list {
                    param.format(formatter, measurement);
                }
                formatter.write(" ");
                expression.format(formatter, measurement);
            }
            Return(_, expression, _) => {
                formatter.write("return");
                if let Some(expr) = expression {
                    formatter.write(" ");
                    expr.format(formatter, measurement);
                }
                formatter.write(";");
            }
            Break(_, expression, _) => {
                formatter.write("break");
                if let Some(expr) = expression {
                    formatter.write(" ");
                    expr.format(formatter, measurement);
                }
                formatter.write(";");
            }
            Continue(_, _) => {
                formatter.write("continue;");
            }
            Unknown { .. } => (),
        }
        formatter.new_line();
    }
}
