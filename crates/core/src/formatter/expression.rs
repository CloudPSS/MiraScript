use crate::{Expression, Operator, parser::MatchCase};

use super::prelude::*;

impl Formattable for Expression<'_> {
    fn measure(&self, formatter: &Formatter, indent: usize) -> usize {
        use Expression::*;
        match self {
            Grouping(_, expression, _) => expression.measure(formatter, indent),
            NonNil(expression, _) | Prefix(_, expression) | Access(expression, _, _) => {
                expression.measure(formatter, indent)
            }
            Index(expression, _, field, _) => usize::max(
                expression.measure(formatter, indent),
                field.measure(formatter, indent),
            ),
            Slice(expression, _, begin, _, end, _) => usize::max(
                expression.measure(formatter, indent),
                usize::max(
                    begin.as_ref().map_or(0, |b| b.measure(formatter, indent)),
                    end.as_ref().map_or(0, |e| e.measure(formatter, indent)),
                ),
            ),
            Infix(left, _, right) => usize::max(
                left.measure(formatter, indent),
                right.measure(formatter, indent),
            ),
            Is(expression, _, pattern) => usize::max(
                expression.measure(formatter, indent),
                pattern.measure(formatter, indent),
            ),
            Loop(_, expression) => expression.measure(formatter, indent),
            While(_, cond, body, else_block) => usize::max(
                cond.measure(formatter, indent),
                usize::max(
                    body.measure(formatter, indent),
                    else_block.measure(formatter, indent),
                ),
            ),
            ForIn(_, pattern, _, iterable, expression, else_block) => usize::max(
                pattern.measure(formatter, indent),
                usize::max(
                    iterable.measure(formatter, indent),
                    usize::max(
                        expression.measure(formatter, indent),
                        else_block.measure(formatter, indent),
                    ),
                ),
            ),
            Cond(cond, _, then_exp, _, else_exp) => usize::max(
                cond.measure(formatter, indent),
                usize::max(
                    then_exp.measure(formatter, indent),
                    else_exp.measure(formatter, indent),
                ),
            ),
            If(_, cond, body, else_block) => usize::max(
                cond.measure(formatter, indent),
                usize::max(
                    body.measure(formatter, indent),
                    else_block.measure(formatter, indent),
                ),
            ),
            Match(_, expr, _, items, _) => usize::max(
                expr.measure(formatter, indent),
                items
                    .iter()
                    .map(|MatchCase(_, pattern, guard, expression)| {
                        usize::max(
                            pattern.measure(formatter, indent),
                            usize::max(
                                guard
                                    .as_ref()
                                    .map_or(0, |(_, expr)| expr.measure(formatter, indent)),
                                expression.measure(formatter, indent),
                            ),
                        )
                    })
                    .max()
                    .unwrap_or(0),
            ),
            Function(_, _, expression) => expression.measure(formatter, indent),
            Block(_, statements, expression, _) => {
                if statements.is_empty() {
                    return expression
                        .as_deref()
                        .map_or(0, |expr| expr.measure(formatter, indent));
                }
                1 + statements
                    .iter()
                    .map(|stmt| stmt.measure(formatter, indent))
                    .max()
                    .unwrap_or(0)
            }
            Literal(literal) => literal.measure(formatter, indent),
            InterpolatedString(token, expressions) => {
                let l = token.measure(formatter, indent);
                let e = expressions
                    .iter()
                    .map(|expr| expr.measure(formatter, indent))
                    .max()
                    .unwrap_or(0);
                usize::max(l, e)
            }
            Variable(v) => v.measure(formatter, indent),
            Record(_, list_items, _) => list_items.measure(formatter, indent),
            Array(_, list_items, _) => list_items.measure(formatter, indent),
            TaggedString(callable, expression) => usize::max(
                callable.measure(formatter, indent),
                expression.measure(formatter, indent),
            ),
            Call(callable, _, list_items, _) => usize::max(
                callable.measure(formatter, indent),
                list_items.measure(formatter, indent),
            ),
            Extension(expression, _, callable, _, list_items, _) => usize::max(
                expression.measure(formatter, indent),
                usize::max(
                    callable.measure(formatter, indent),
                    list_items.measure(formatter, indent),
                ),
            ),
            Unknown { .. } => 0,
        }
    }

    fn format(&self, formatter: &mut Formatter, complexity: usize) {
        use Expression::*;
        match self {
            Literal(token_ref) => formatter.write_token(token_ref),
            InterpolatedString(token, expressions) => {
                formatter.write_str_token(token, expressions, complexity);
            }
            Variable(token_ref) => formatter.write_token(token_ref),
            Grouping(op, expression, cp) => {
                formatter.write_token(op);
                expression.format(formatter, complexity);
                formatter.write_token(cp);
            }
            Record(op, list_items, cp) => {
                formatter.write_token(op);
                if **op == Operator::OpenBrace {
                    formatter.write_space();
                }
                formatter.format(list_items);
                if list_items.len() == 1 && list_items[0].is_unnamed() {
                    formatter.write_token_or(list_items[0].tail_comma(), Operator::Comma);
                }
                if **cp == Operator::CloseBrace {
                    formatter.write_space();
                }
                formatter.write_token(cp);
            }
            Array(op, list_items, cp) => {
                formatter.write_token(op);
                formatter.format(list_items);
                formatter.write_token(cp);
            }
            TaggedString(callable, expression) => {
                callable.format(formatter, complexity);
                if formatter.ends_with("@") {
                    formatter.write_space();
                }
                expression.format(formatter, complexity);
            }
            Call(callable, op, list_items, cp) => {
                formatter.format(callable);
                formatter.write_token(op);
                formatter.format(list_items);
                formatter.write_token(cp);
            }
            Extension(expression, cc, callable, op, list_items, cp) => {
                formatter.format(expression);
                formatter.write_token(cc);
                formatter.format(callable);
                formatter.write_token(op);
                formatter.format(list_items);
                formatter.write_token(cp);
            }
            Access(expression, dot, field) => {
                formatter.format(expression);
                formatter.write_token(dot);
                formatter.write_token(field);
            }
            Index(expression, op, field_expr, cp) => {
                formatter.format(expression);
                formatter.write_token(op);
                formatter.format(field_expr);
                formatter.write_token(cp);
            }
            Slice(expression, op, left, range, right, cp) => {
                formatter.format(expression);
                formatter.write_token(op);
                if let Some(left) = left {
                    formatter.format(left);
                }
                formatter.write_token(range);
                if let Some(right) = right {
                    formatter.format(right);
                }
                formatter.write_token(cp);
            }
            NonNil(expression, bang) => {
                formatter.format(expression);
                formatter.write_token(bang);
            }
            Prefix(op, expression) => {
                formatter.write_token(op);
                if op.is_keyword() {
                    formatter.write_space();
                }
                formatter.format(expression);
            }
            Infix(l, op, r) => {
                formatter.format(l);
                if **op == Operator::Caret {
                    formatter.write_token(op);
                } else {
                    formatter.write_space();
                    formatter.write_token(op);
                    formatter.write_space();
                }
                formatter.format(r);
            }
            Is(expression, kw, pattern) => {
                formatter.format(expression);
                formatter.write_space();
                formatter.write_token(kw);
                formatter.write_space();
                formatter.format(pattern);
            }
            Block(op, statements, expression, cp) => {
                let expression = expression.as_deref();
                if statements.is_empty() && expression.is_none() {
                    formatter.write_token(op);
                    formatter.write_space();
                    formatter.write_token(cp);
                    return;
                }
                if statements.is_empty()
                    && !expression.unwrap().is_block_like()
                    && expression.unwrap().measure(formatter, 0) == 0
                {
                    let expression = expression.unwrap();
                    formatter.write_token(op);
                    formatter.write_space();
                    expression.format(formatter, complexity);
                    formatter.write_space();
                    formatter.write_token(cp);
                    return;
                }
                formatter.write_token(op);
                formatter.indent();
                for statement in statements {
                    formatter.new_line();
                    statement.format(formatter, complexity);
                }
                if let Some(expression) = expression {
                    formatter.new_line();
                    expression.format(formatter, complexity);
                }
                formatter.dedent();
                formatter.new_line();
                formatter.write_token(cp);
            }
            Loop(kw, body) => {
                formatter.write_token(kw);
                formatter.write_space();
                body.format(formatter, complexity);
            }
            While(kw, cond, body, else_block) => {
                formatter.write_token(kw);
                formatter.write_space();
                formatter.format(cond);
                formatter.write_space();
                body.format(formatter, complexity);
                else_block.format(formatter, complexity);
            }
            ForIn(kw_for, pattern, kw_in, iterable, body, else_block) => {
                formatter.write_token(kw_for);
                formatter.write_space();
                formatter.format(pattern);
                formatter.write_space();
                formatter.write_token(kw_in);
                formatter.write_space();
                formatter.format(iterable);
                formatter.write_space();
                body.format(formatter, complexity);
                else_block.format(formatter, complexity);
            }
            Cond(cond, op_question, then_exp, op_colon, else_exp) => {
                formatter.format(cond);
                formatter.write_space();
                formatter.write_token(op_question);
                formatter.write_space();
                then_exp.format(formatter, complexity);
                formatter.write_space();
                formatter.write_token(op_colon);
                formatter.write_space();
                else_exp.format(formatter, complexity);
            }
            If(kw, cond, body, else_block) => {
                formatter.write_token(kw);
                formatter.write_space();
                formatter.format(cond);
                formatter.write_space();
                body.format(formatter, complexity);
                else_block.format(formatter, complexity);
            }
            Match(kw, matcher, op, items, cp) => {
                formatter.write_token(kw);
                formatter.write_space();
                formatter.format(matcher);
                formatter.write_space();
                formatter.write_token(op);
                if items.is_empty() {
                    formatter.write_space();
                    formatter.write_token(cp);
                    return;
                }
                formatter.indent();
                formatter.new_line();
                for (i, MatchCase(kw, pattern, guard, expression)) in items.iter().enumerate() {
                    formatter.write_token(kw);
                    formatter.write_space();
                    pattern.format(formatter, complexity);
                    formatter.write_space();
                    if let Some((kw, expr)) = guard {
                        formatter.write_token(kw);
                        formatter.write_space();
                        expr.format(formatter, complexity);
                        formatter.write_space();
                    }
                    expression.format(formatter, complexity);
                    if i != items.len() - 1 {
                        formatter.new_line();
                    }
                }
                formatter.dedent();
                formatter.new_line();
                formatter.write_token(cp);
            }
            Function(kw, parameter_list, expression) => {
                formatter.write_token(kw);
                formatter.write_space();
                if let Some(parameter_list) = parameter_list {
                    formatter.format(parameter_list);
                    formatter.write_space();
                }
                expression.format(formatter, complexity);
            }
            Unknown { .. } => (),
        }
    }
}
