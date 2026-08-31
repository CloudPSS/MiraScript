use crate::{
    Operator,
    parser::{ElseBlock, Expression, MatchCase, Statement, TokenRef},
};

use super::prelude::*;

fn collect_infix<'s, 'a>(
    expression: &'a Expression<'s>,
    operands: &mut Vec<&'a Expression<'s>>,
    operators: &mut Vec<&'a crate::parser::TokenRef<'s>>,
) {
    if let Expression::Infix(left, operator, right) = expression
        && **operator != Operator::Caret
    {
        collect_infix(left, operands, operators);
        operators.push(operator);
        collect_infix(right, operands, operators);
    } else {
        operands.push(expression);
    }
}

fn format_infix(expression: &Expression<'_>, formatter: &Formatter) -> FormatDoc {
    let mut operands = Vec::new();
    let mut operators = Vec::new();
    collect_infix(expression, &mut operands, &mut operators);
    let mut operands = operands.into_iter();
    let Some(first) = operands.next() else {
        return Formatter::nil();
    };
    let continuation = operators
        .into_iter()
        .zip(operands)
        .map(|(operator, operand)| {
            formatter
                .space()
                .append(formatter.token(operator))
                .append(formatter.line())
                .append(operand.format(formatter))
        });
    first
        .format(formatter)
        .append(formatter.indent(formatter.concat(continuation)))
        .group()
}

fn collect_extension_docs(
    expression: &Expression<'_>,
    formatter: &Formatter,
    extensions: &mut Vec<FormatDoc>,
) -> FormatDoc {
    let Expression::Extension(expression, colon_colon, callable, open, items, close) = expression
    else {
        return expression.format(formatter);
    };
    let base = collect_extension_docs(expression, formatter, extensions);
    extensions.push(
        formatter
            .token(colon_colon)
            .append(callable.format(formatter))
            .append(formatter.token(open))
            .append(items.format(formatter))
            .append(formatter.token(close)),
    );
    base
}

fn format_extension_chain(expression: &Expression<'_>, formatter: &Formatter) -> FormatDoc {
    let mut extensions = Vec::new();
    let base = collect_extension_docs(expression, formatter, &mut extensions);
    if extensions.len() == 1 {
        return base.append(extensions.pop().unwrap());
    }
    let continuation = extensions
        .into_iter()
        .map(|extension| formatter.line_().append(extension));
    base.append(formatter.indent(formatter.concat(continuation)))
        .group()
}

fn format_block_parts(
    open: &TokenRef<'_>,
    statements: &[Statement<'_>],
    expression: Option<&Expression<'_>>,
    close: &TokenRef<'_>,
    formatter: &Formatter,
    grouped: bool,
) -> FormatDoc {
    let close_comments = formatter.detached_leading_comments(&close.leading_trivia);
    if statements.is_empty() && expression.is_none() && close_comments.is_none() {
        return formatter
            .token(open)
            .append(formatter.space())
            .append(formatter.token_without_leading(close));
    }

    let mut body = formatter.join(
        statements
            .iter()
            .map(|statement| statement.format(formatter))
            .chain(expression.map(|expression| expression.format(formatter))),
        formatter.hardline(),
    );
    if let Some(close_comments) = close_comments {
        if !statements.is_empty() || expression.is_some() {
            body = body.append(formatter.hardline());
        }
        body = body.append(close_comments);
    }
    let boundary = if statements.is_empty()
        && expression.is_some_and(|expression| !expression.is_block_like())
        && close.leading_trivia.is_empty()
    {
        formatter.line()
    } else {
        formatter.hardline()
    };
    let doc = formatter
        .token(open)
        .append(formatter.indent(boundary.clone().append(body)))
        .append(boundary)
        .append(formatter.token_without_leading(close));
    if !grouped {
        return doc;
    }
    let normal = doc.group();
    if statements.is_empty()
        && close.leading_trivia.is_empty()
        && let Some(expression) = expression
        && !expression.is_block_like()
    {
        let inline = formatter
            .token(open)
            .append(formatter.space())
            .append(expression.format(formatter))
            .append(formatter.space())
            .append(formatter.token_without_leading(close));
        let inline_text = formatter.render(inline.clone(), 0);
        if !inline_text.contains('\n') && inline_text.chars().count() <= 20 {
            inline
        } else {
            normal
        }
    } else {
        normal
    }
}

fn format_if_expression(
    expression: &Expression<'_>,
    formatter: &Formatter,
    grouped: bool,
) -> FormatDoc {
    let Expression::If(keyword, condition, body, else_block) = expression else {
        return expression.format(formatter);
    };
    let body = match body.as_ref() {
        Expression::Block(open, statements, expression, close) => format_block_parts(
            open,
            statements,
            expression.as_deref(),
            close,
            formatter,
            false,
        ),
        body => body.format(formatter),
    };
    let else_block = else_block
        .as_ref()
        .map_or_else(Formatter::nil, |ElseBlock(keyword, body)| {
            let body = match body.as_ref() {
                Expression::Block(open, statements, expression, close) => format_block_parts(
                    open,
                    statements,
                    expression.as_deref(),
                    close,
                    formatter,
                    false,
                ),
                Expression::If(..) => format_if_expression(body, formatter, false),
                body => body.format(formatter),
            };
            formatter
                .space()
                .append(formatter.token(keyword))
                .append(formatter.space())
                .append(body)
        });
    let doc = formatter
        .token(keyword)
        .append(formatter.space())
        .append(condition.format(formatter))
        .append(formatter.space())
        .append(body)
        .append(else_block);
    if grouped { doc.group() } else { doc }
}

impl Formattable for Expression<'_> {
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        use Expression::*;
        match self {
            Literal(token) | Variable(token) => formatter.token(token),
            InterpolatedString(token, expressions) => formatter.string_token(token, expressions),
            Grouping(open, expression, close) => formatter
                .token(open)
                .append(expression.format(formatter))
                .append(formatter.token(close)),
            Record(open, items, close) => {
                let boundary = if **open == Operator::OpenBrace {
                    formatter.line()
                } else {
                    formatter.line_()
                };
                let force_tail = items.len() == 1 && items[0].is_unnamed();
                formatter
                    .token(open)
                    .append(formatter.list_items(items, boundary, force_tail))
                    .append(formatter.token(close))
            }
            Array(open, items, close) => formatter
                .token(open)
                .append(formatter.list_items(items, formatter.line_(), false))
                .append(formatter.token(close)),
            TaggedString(callable, expression) => {
                let callable = callable.format(formatter);
                let separator = if formatter.render(callable.clone(), 0).ends_with('@') {
                    formatter.space()
                } else {
                    Formatter::nil()
                };
                callable
                    .append(separator)
                    .append(expression.format(formatter))
            }
            Call(callable, open, items, close) => callable
                .format(formatter)
                .append(formatter.token(open))
                .append(items.format(formatter))
                .append(formatter.token(close)),
            Extension(..) => format_extension_chain(self, formatter),
            Access(expression, dot, field) => expression
                .format(formatter)
                .append(formatter.token(dot))
                .append(formatter.token(field)),
            Index(expression, open, field, close) => expression
                .format(formatter)
                .append(formatter.token(open))
                .append(field.format(formatter))
                .append(formatter.token(close)),
            Slice(expression, open, left, range, right, close) => expression
                .format(formatter)
                .append(formatter.token(open))
                .append(left.format(formatter))
                .append(formatter.token(range))
                .append(right.format(formatter))
                .append(formatter.token(close)),
            NonNil(expression, bang) => expression.format(formatter).append(formatter.token(bang)),
            Prefix(operator, expression) => formatter
                .token(operator)
                .append(if operator.is_keyword() {
                    formatter.space()
                } else {
                    Formatter::nil()
                })
                .append(expression.format(formatter)),
            Infix(left, operator, right) => {
                if **operator == Operator::Caret {
                    left.format(formatter)
                        .append(formatter.token(operator))
                        .append(right.format(formatter))
                } else {
                    let _ = (left, right);
                    format_infix(self, formatter)
                }
            }
            Is(expression, keyword, pattern) => expression
                .format(formatter)
                .append(formatter.space())
                .append(formatter.token(keyword))
                .append(formatter.indent(formatter.line().append(pattern.format(formatter))))
                .group(),
            Block(open, statements, expression, close) => format_block_parts(
                open,
                statements,
                expression.as_deref(),
                close,
                formatter,
                true,
            ),
            Loop(keyword, body) => formatter
                .token(keyword)
                .append(formatter.space())
                .append(body.format(formatter)),
            While(keyword, condition, body, else_block) => formatter
                .token(keyword)
                .append(formatter.space())
                .append(condition.format(formatter))
                .append(formatter.space())
                .append(body.format(formatter))
                .append(else_block.format(formatter)),
            ForIn(keyword_for, pattern, keyword_in, iterable, body, else_block) => formatter
                .token(keyword_for)
                .append(formatter.space())
                .append(pattern.format(formatter))
                .append(formatter.space())
                .append(formatter.token(keyword_in))
                .append(formatter.space())
                .append(iterable.format(formatter))
                .append(formatter.space())
                .append(body.format(formatter))
                .append(else_block.format(formatter)),
            Cond(condition, question, then_expression, colon, else_expression) => condition
                .format(formatter)
                .append(formatter.space())
                .append(formatter.token(question))
                .append(
                    formatter.indent(formatter.line().append(then_expression.format(formatter))),
                )
                .append(formatter.space())
                .append(formatter.token(colon))
                .append(
                    formatter.indent(formatter.line().append(else_expression.format(formatter))),
                )
                .group(),
            If(..) => format_if_expression(self, formatter, true),
            Match(keyword, matcher, open, cases, close) => {
                let head = formatter
                    .token(keyword)
                    .append(formatter.space())
                    .append(matcher.format(formatter))
                    .append(formatter.space())
                    .append(formatter.token(open));
                if cases.is_empty() {
                    return head
                        .append(formatter.space())
                        .append(formatter.token(close));
                }
                let cases = cases
                    .iter()
                    .map(|MatchCase(keyword, pattern, guard, expression)| {
                        formatter
                            .token(keyword)
                            .append(formatter.space())
                            .append(pattern.format(formatter))
                            .append(formatter.space())
                            .append(guard.as_ref().map_or_else(
                                Formatter::nil,
                                |(keyword, guard)| {
                                    formatter
                                        .token(keyword)
                                        .append(formatter.space())
                                        .append(guard.format(formatter))
                                        .append(formatter.space())
                                },
                            ))
                            .append(expression.format(formatter))
                    });
                head.append(
                    formatter.indent(
                        formatter
                            .hardline()
                            .append(formatter.join(cases, formatter.hardline())),
                    ),
                )
                .append(formatter.hardline())
                .append(formatter.token(close))
            }
            Function(keyword, parameters, body) => formatter
                .token(keyword)
                .append(formatter.space())
                .append(parameters.format(formatter))
                .append(if parameters.is_some() {
                    formatter.space()
                } else {
                    Formatter::nil()
                })
                .append(body.format(formatter)),
            Unknown { .. } => Formatter::nil(),
        }
    }
}
