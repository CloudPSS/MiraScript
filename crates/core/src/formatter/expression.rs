use crate::{Expression, lexer::TokenKind};

use super::prelude::*;

impl Formattable for Expression<'_> {
    fn measure(&self, formatter: &Formatter, columns: usize) -> Measurement {
        (0, 0).into()
    }

    fn format(&self, formatter: &mut Formatter, measurement: Measurement) {
        use Expression::*;
        match self {
            Literal(token_ref) => formatter.write_token(token_ref),
            InterpolatedString(token, expressions) => {
                let TokenKind::InterpolatedString(strs, info) = &token.kind else {
                    unreachable!();
                };
                let quote = info.quote;
                let dollars =
                    String::from_iter(std::iter::repeat_n('$', std::cmp::min(info.ats, 1)));
                if let Some(quote) = quote {
                    formatter.write(&String::from_iter(std::iter::repeat_n('@', info.ats)));
                    formatter.write(&quote.to_string());
                }
                let mut strs = strs.iter();
                for expr in expressions {
                    if let Some((str, _)) = strs.next() {
                        formatter.write(str);
                    }
                    formatter.write(&dollars);
                    expr.format(formatter, measurement);
                }
                if let Some((str, _)) = strs.next() {
                    formatter.write(str);
                }

                if let Some(quote) = quote {
                    formatter.write(&quote.to_string());
                    formatter.write(&String::from_iter(std::iter::repeat_n('@', info.ats)));
                }
            }
            Variable(token_ref) => formatter.write_token(token_ref),
            Grouping(_, expression, _) => {
                formatter.write("(");
                expression.format(formatter, measurement);
                formatter.write(")");
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
            Call(callable, _, list_items, _) => {
                callable.format(formatter, measurement);
                formatter.write("(");
                list_items[..].format(formatter, measurement);
                formatter.write(")");
            }
            Extension(expression, _, callable, _, list_items, _) => {
                expression.format(formatter, measurement);
                formatter.write("::");
                callable.format(formatter, measurement);
                formatter.write("(");
                list_items[..].format(formatter, measurement);
                formatter.write(")");
            }
            Access(expression, _, field) => {
                expression.format(formatter, measurement);
                formatter.write(".");
                formatter.write_token(field);
            }
            Index(expression, _, field_expr, _) => {
                expression.format(formatter, measurement);
                formatter.write("[");
                field_expr.format(formatter, measurement);
                formatter.write("]");
            }
            Slice(expression, _, left, op, right, _) => {
                expression.format(formatter, measurement);
                formatter.write("[");
                if let Some(left) = left {
                    left.format(formatter, measurement);
                }
                formatter.write_token(op);
                if let Some(right) = right {
                    right.format(formatter, measurement);
                }
                formatter.write("]");
            }
            NonNil(expression, _) => {
                expression.format(formatter, measurement);
                formatter.write("!");
            }
            Prefix(op, expression) => {
                formatter.write_token(op);
                expression.format(formatter, measurement);
            }
            Infix(l, op, r) => {
                l.format(formatter, measurement);
                formatter.write(" ");
                formatter.write_token(op);
                formatter.write(" ");
                r.format(formatter, measurement);
            }
            Is(expression, _, pattern) => {
                expression.format(formatter, measurement);
                formatter.write(" is ");
                pattern.format(formatter, measurement);
            }
            Block(_, statements, expression, _) => {
                formatter.write("{");
                formatter.indent();
                formatter.new_line();
                (&statements[..], expression.as_deref()).format(formatter, measurement);
                formatter.unindent();
                formatter.new_line();
                formatter.write("}");
            }
            Loop(_, body) => {
                formatter.write("loop ");
                body.format(formatter, measurement);
            }
            While(_, cond, body, else_block) => {
                formatter.write("while ");
                cond.format(formatter, measurement);
                formatter.write(" ");
                body.format(formatter, measurement);
                if let Some((_, else_block)) = else_block {
                    formatter.write(" else ");
                    else_block.format(formatter, measurement);
                }
            }
            ForIn(_, pattern, _, iterable, body, else_block) => {
                formatter.write("for ");
                pattern.format(formatter, measurement);
                formatter.write(" in ");
                iterable.format(formatter, measurement);
                formatter.write(" ");
                body.format(formatter, measurement);
                if let Some((_, else_block)) = else_block {
                    formatter.write(" else ");
                    else_block.format(formatter, measurement);
                }
            }
            If(_, cond, body, else_block) => {
                formatter.write("if ");
                cond.format(formatter, measurement);
                formatter.write(" ");
                body.format(formatter, measurement);
                if let Some((_, else_block)) = else_block {
                    formatter.write(" else ");
                    else_block.format(formatter, measurement);
                }
            }
            Match(_, matcher, _, items, _) => {
                formatter.write("match ");
                matcher.format(formatter, measurement);
                if items.is_empty() {
                    formatter.write(" {}");
                    return;
                }
                formatter.write(" {");
                formatter.indent();
                formatter.new_line();
                for (_, pattern, expression) in items {
                    formatter.write("case ");
                    pattern.format(formatter, measurement);
                    formatter.write(" ");
                    expression.format(formatter, measurement);
                    formatter.new_line();
                }
                formatter.unindent();
                formatter.new_line();
                formatter.write("}");
            }
            Function(_, parameter_list, expression) => {
                formatter.write("fn ");
                if let Some(parameter_list) = parameter_list {
                    parameter_list.format(formatter, measurement);
                    formatter.write(" ");
                }
                expression.format(formatter, measurement);
            }
            Unknown { .. } => (),
        }
    }
}
