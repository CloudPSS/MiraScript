use crate::{Expression, Pattern, Statement, parser::TokenRef};

use super::prelude::*;

fn format_bind(
    pattern: &Pattern,
    op: &TokenRef,
    expression: &Expression,
    semicolon: &TokenRef,
    formatter: &mut Formatter,
) {
    formatter.format(pattern);
    formatter.write_space();
    formatter.write_token(op);
    formatter.write_space();
    formatter.format(expression);
    formatter.write_token(semicolon);
}

impl Formattable for Statement<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        use Statement::*;
        match self {
            BlockExpression(expression) => usize::max(1, expression.measure(formatter, indent)),
            _ => 1,
        }
    }

    fn format(&self, formatter: &mut Formatter, complexity: usize) {
        use Statement::*;
        match self {
            Empty(semicolon) => formatter.write_token(semicolon),
            Expression(expression, semicolon) => {
                formatter.format(expression.as_ref());
                formatter.write_token(semicolon);
            }
            BlockExpression(expression) => expression.format(formatter, complexity),
            Module(kw_pub, kw_mod, id, body) => {
                if let Some(kw_pub) = kw_pub {
                    formatter.write_token(kw_pub);
                    formatter.write_space();
                }
                formatter.write_token(kw_mod);
                formatter.write_space();
                formatter.write_token(id);
                formatter.write_space();
                body.format(formatter, complexity);
            }
            Bind(kw_pub, kw_let, pattern, op, expression, semicolon) => {
                if let Some(kw_pub) = kw_pub {
                    formatter.write_token(kw_pub);
                    formatter.write_space();
                }
                formatter.write_token(kw_let);
                formatter.write_space();
                format_bind(pattern, op, expression, semicolon, formatter);
            }
            Rebind(pattern, op, expression, semicolon) => {
                format_bind(pattern, op, expression, semicolon, formatter);
            }
            Const(kw_pub, kw_const, id, op, expression, semicolon) => {
                if let Some(kw_pub) = kw_pub {
                    formatter.write_token(kw_pub);
                    formatter.write_space();
                }
                formatter.write_token(kw_const);
                formatter.write_space();
                formatter.write_token(id);
                formatter.write_space();
                formatter.write_token(op);
                formatter.write_space();
                formatter.format(expression);
                formatter.write_token(semicolon);
            }
            Assign(assignee, op, expression, semicolon) => {
                formatter.format(assignee);
                formatter.write_space();
                formatter.write_token(op);
                formatter.write_space();
                formatter.format(expression);
                formatter.write_token(semicolon);
            }
            Function(kw_pub, kw_fn, id, parameter_list, expression) => {
                if let Some(kw_pub) = kw_pub {
                    formatter.write_token(kw_pub);
                    formatter.write_space();
                }
                formatter.write_token(kw_fn);
                formatter.write_space();
                formatter.write_token(id);
                let mut p_complexity = 0;
                if let Some(parameter_list) = parameter_list {
                    p_complexity = formatter.measure(parameter_list);
                    parameter_list.format(formatter, p_complexity);
                }
                formatter.write_space();
                let e_complexity = formatter.measure(expression);
                expression.format(
                    formatter,
                    if p_complexity > 0 {
                        e_complexity.max(1)
                    } else {
                        e_complexity
                    },
                );
            }
            Return(kw, expression, semicolon) => {
                formatter.write_token(kw);
                if let Some(expr) = expression {
                    formatter.write_space();
                    formatter.format(expr);
                }
                formatter.write_token(semicolon);
            }
            Break(kw, expression, semicolon) => {
                formatter.write_token(kw);
                if let Some(expr) = expression {
                    formatter.write_space();
                    formatter.format(expr);
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
