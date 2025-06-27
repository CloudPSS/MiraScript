use crate::{Expression, Operator, lexer::TokenKind};

use super::prelude::*;

impl Formattable for Expression<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        use Expression::*;
        match self {
            Grouping(_, expression, _) => expression.measure(formatter, indent),
            NonNil(expression, _) | Prefix(_, expression) | Access(expression, _, _) => {
                expression.measure(formatter, indent)
            }
            Index(expression, _, field, _) => {
                expression.measure(formatter, indent) + field.measure(formatter, indent)
            }
            Slice(expression, _, begin, _, end, _) => {
                expression.measure(formatter, indent)
                    + begin.as_ref().map_or(0, |b| b.measure(formatter, indent))
                    + end.as_ref().map_or(0, |e| e.measure(formatter, indent))
            }
            Infix(left, _, right) => {
                left.measure(formatter, indent) + right.measure(formatter, indent)
            }
            Is(expression, _, pattern) => {
                expression.measure(formatter, indent) + pattern.measure(formatter, indent)
            }
            Loop(_, expression) => expression.measure(formatter, indent),
            While(_, cond, body, else_block) => {
                cond.measure(formatter, indent)
                    + body.measure(formatter, indent)
                    + else_block.measure(formatter, indent)
            }
            ForIn(_, pattern, _, iterable, expression, else_block) => {
                expression.measure(formatter, indent)
                    + pattern.measure(formatter, indent)
                    + iterable.measure(formatter, indent)
                    + else_block.measure(formatter, indent)
            }
            If(_, cond, body, else_block) => {
                cond.measure(formatter, indent)
                    + body.measure(formatter, indent)
                    + else_block.measure(formatter, indent)
            }
            Match(_, expr, _, items, _) => expr.measure(formatter, indent) + items.len(),
            Function(_, _, expression) => expression.measure(formatter, indent),
            Block(_, statements, expression, _) => {
                if statements.is_empty() {
                    return expression
                        .as_deref()
                        .map_or(0, |expr| expr.measure(formatter, indent));
                }
                statements.len()
                    + 2
                    + expression
                        .as_deref()
                        .map_or(0, |expr| expr.measure(formatter, indent))
            }
            Literal(literal) => literal.measure(formatter, indent),
            InterpolatedString(token, expressions) => {
                let l = token.measure(formatter, indent);
                let e = expressions
                    .iter()
                    .map(|expr| expr.measure(formatter, indent))
                    .sum::<usize>();
                l + e
            }
            Variable(v) => v.measure(formatter, indent),
            Record(_, list_items, _) => list_items.measure(formatter, indent),
            Array(_, list_items, _) => list_items.measure(formatter, indent),
            Call(callable, _, list_items, _) => {
                callable.measure(formatter, indent) + list_items.measure(formatter, indent)
            }
            Extension(expression, _, callable, _, list_items, _) => {
                expression.measure(formatter, indent)
                    + callable.measure(formatter, indent)
                    + list_items.measure(formatter, indent)
            }
            Unknown { .. } => 0,
        }
    }

    fn format(&self, formatter: &mut Formatter, measurement: usize) {
        use Expression::*;
        match self {
            Literal(token_ref) => formatter.write_token(token_ref),
            InterpolatedString(token, expressions) => {
                let TokenKind::InterpolatedString(_, info) = &token.kind else {
                    unreachable!();
                };
                formatter.write_str_token(info, expressions, measurement);
            }
            Variable(token_ref) => formatter.write_token(token_ref),
            Grouping(op, expression, cp) => {
                formatter.write_token(op);
                expression.format(formatter, measurement);
                formatter.write_token(cp);
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
            Call(callable, op, list_items, cp) => {
                callable.format(formatter, measurement);
                formatter.write_token(op);
                list_items[..].format(formatter, measurement);
                formatter.write_token(cp);
            }
            Extension(expression, cc, callable, op, list_items, cp) => {
                expression.format(formatter, measurement);
                formatter.write_token(cc);
                callable.format(formatter, measurement);
                formatter.write_token(op);
                list_items[..].format(formatter, measurement);
                formatter.write_token(cp);
            }
            Access(expression, dot, field) => {
                expression.format(formatter, measurement);
                formatter.write_token(dot);
                formatter.write_token(field);
            }
            Index(expression, op, field_expr, cp) => {
                expression.format(formatter, measurement);
                formatter.write_token(op);
                field_expr.format(formatter, measurement);
                formatter.write_token(cp);
            }
            Slice(expression, op, left, range, right, cp) => {
                expression.format(formatter, measurement);
                formatter.write_token(op);
                if let Some(left) = left {
                    left.format(formatter, measurement);
                }
                formatter.write_token(range);
                if let Some(right) = right {
                    right.format(formatter, measurement);
                }
                formatter.write_token(cp);
            }
            NonNil(expression, bang) => {
                expression.format(formatter, measurement);
                formatter.write_token(bang);
            }
            Prefix(op, expression) => {
                formatter.write_token(op);
                expression.format(formatter, measurement);
            }
            Infix(l, op, r) => {
                l.format(formatter, measurement);
                if **op == Operator::Caret {
                    formatter.write_token(op);
                } else {
                    formatter.write_space();
                    formatter.write_token(op);
                    formatter.write_space();
                }
                r.format(formatter, measurement);
            }
            Is(expression, kw, pattern) => {
                expression.format(formatter, measurement);
                formatter.write_space();
                formatter.write_token(kw);
                formatter.write_space();
                pattern.format(formatter, measurement);
            }
            Block(op, statements, expression, cp) => {
                let expression = expression.as_deref();
                if statements.is_empty() && expression.is_none() {
                    formatter.write_token(op);
                    formatter.write_space();
                    formatter.write_token(cp);
                    return;
                }
                if statements.is_empty() && formatter.measure(expression.unwrap()) == 0 {
                    let expression = expression.unwrap();
                    formatter.write_token(op);
                    formatter.write_space();
                    expression.format(formatter, measurement);
                    formatter.write_space();
                    formatter.write_token(cp);
                    return;
                }
                formatter.write_token(op);
                formatter.indent();
                for statement in statements {
                    formatter.new_line();
                    statement.format(formatter, measurement);
                }
                if let Some(expression) = expression {
                    formatter.new_line();
                    expression.format(formatter, measurement);
                }
                formatter.unindent();
                formatter.new_line();
                formatter.write_token(cp);
            }
            Loop(kw, body) => {
                formatter.write_token(kw);
                formatter.write_space();
                body.format(formatter, measurement);
            }
            While(kw, cond, body, else_block) => {
                formatter.write_token(kw);
                formatter.write_space();
                cond.format(formatter, measurement);
                formatter.write_space();
                body.format(formatter, measurement);
                else_block.format(formatter, measurement);
            }
            ForIn(kw_for, pattern, kw_in, iterable, body, else_block) => {
                formatter.write_token(kw_for);
                formatter.write_space();
                pattern.format(formatter, measurement);
                formatter.write_space();
                formatter.write_token(kw_in);
                formatter.write_space();
                iterable.format(formatter, measurement);
                formatter.write_space();
                body.format(formatter, measurement);
                else_block.format(formatter, measurement);
            }
            If(kw, cond, body, else_block) => {
                formatter.write_token(kw);
                formatter.write_space();
                cond.format(formatter, measurement);
                formatter.write_space();
                body.format(formatter, measurement);
                else_block.format(formatter, measurement);
            }
            Match(kw, matcher, op, items, cp) => {
                formatter.write_token(kw);
                formatter.write_space();
                matcher.format(formatter, measurement);
                formatter.write_space();
                formatter.write_token(op);
                if items.is_empty() {
                    formatter.write_space();
                    formatter.write_token(cp);
                    return;
                }
                formatter.indent();
                formatter.new_line();
                for (i, (kw, pattern, expression)) in items.iter().enumerate() {
                    formatter.write_token(kw);
                    formatter.write_space();
                    pattern.format(formatter, measurement);
                    formatter.write_space();
                    expression.format(formatter, measurement);
                    if i != items.len() - 1 {
                        formatter.new_line();
                    }
                }
                formatter.unindent();
                formatter.new_line();
                formatter.write_token(cp);
            }
            Function(kw, parameter_list, expression) => {
                formatter.write_token(kw);
                formatter.write_space();
                if let Some(parameter_list) = parameter_list {
                    parameter_list.format(formatter, measurement);
                    formatter.write_space();
                }
                expression.format(formatter, measurement);
            }
            Unknown { .. } => (),
        }
    }
}
