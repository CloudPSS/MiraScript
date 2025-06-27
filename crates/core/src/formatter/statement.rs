use crate::Statement;

use super::prelude::*;

impl Formattable for Statement<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        1
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        use Statement::*;
        match self {
            Empty(semicolon) => formatter.write_token(semicolon),
            Expression(expression, semicolon) => {
                expression.format(formatter, measurement);
                formatter.write_token(semicolon);
            }
            BlockExpression(expression) => expression.format(formatter, measurement),
            Bind(kw, pattern, op, expression, semicolon) => {
                formatter.write_token(kw);
                formatter.write_space();
                pattern.format(formatter, measurement);
                formatter.write_space();
                formatter.write_token(op);
                formatter.write_space();
                expression.format(formatter, measurement);
                formatter.write_token(semicolon);
            }
            Rebind(pattern, op, expression, semicolon) => {
                pattern.format(formatter, measurement);
                formatter.write_space();
                formatter.write_token(op);
                formatter.write_space();
                expression.format(formatter, measurement);
                formatter.write_token(semicolon);
            }
            Assign(assignee, op, expression, semicolon) => {
                assignee.format(formatter, measurement);
                formatter.write_space();
                formatter.write_token(op);
                formatter.write_space();
                expression.format(formatter, measurement);
                formatter.write_token(semicolon);
            }
            Function(kw, id, parameter_list, expression) => {
                formatter.write_token(kw);
                formatter.write_space();
                formatter.write_token(id);
                if let Some(param) = parameter_list {
                    param.format(formatter, measurement);
                }
                formatter.write_space();
                expression.format(formatter, measurement);
            }
            Return(kw, expression, semicolon) => {
                formatter.write_token(kw);
                if let Some(expr) = expression {
                    formatter.write_space();
                    expr.format(formatter, measurement);
                }
                formatter.write_token(semicolon);
            }
            Break(kw, expression, semicolon) => {
                formatter.write_token(kw);
                if let Some(expr) = expression {
                    formatter.write_space();
                    expr.format(formatter, measurement);
                }
                formatter.write_token(semicolon);
            }
            Continue(kw, semicolon) => {
                formatter.write_token(kw);
                formatter.write_token(semicolon);
            }
            Unknown { .. } => (),
        }
    }
}
