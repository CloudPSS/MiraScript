use crate::parser::{Expression, Pattern, Statement, TokenRef};

use super::prelude::*;

fn assignment(
    left: FormatDoc,
    operator: &TokenRef<'_>,
    expression: &Expression<'_>,
    semicolon: &TokenRef<'_>,
    formatter: &Formatter,
) -> FormatDoc {
    left.append(formatter.space())
        .append(formatter.token(operator))
        .append(formatter.space())
        .append(expression.format(formatter))
        .append(formatter.token(semicolon))
}

fn bind(
    pattern: &Pattern<'_>,
    operator: &TokenRef<'_>,
    expression: &Expression<'_>,
    semicolon: &TokenRef<'_>,
    formatter: &Formatter,
) -> FormatDoc {
    assignment(
        pattern.format(formatter),
        operator,
        expression,
        semicolon,
        formatter,
    )
}

impl Formattable for Statement<'_> {
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        use Statement::*;
        match self {
            Empty(semicolon) => formatter.token(semicolon),
            Expression(expression, semicolon) => expression
                .format(formatter)
                .append(formatter.token(semicolon)),
            BlockExpression(expression) => expression.format(formatter),
            Module(keyword_pub, keyword_mod, identifier, body) => keyword_pub
                .as_deref()
                .map_or_else(Formatter::nil, |keyword| {
                    formatter.token(keyword).append(formatter.space())
                })
                .append(formatter.token(keyword_mod))
                .append(formatter.space())
                .append(formatter.token(identifier))
                .append(formatter.space())
                .append(body.format(formatter)),
            Bind(keyword_pub, keyword_let, pattern, operator, expression, semicolon) => keyword_pub
                .as_deref()
                .map_or_else(Formatter::nil, |keyword| {
                    formatter.token(keyword).append(formatter.space())
                })
                .append(formatter.token(keyword_let))
                .append(formatter.space())
                .append(bind(pattern, operator, expression, semicolon, formatter)),
            Rebind(pattern, operator, expression, semicolon) => {
                bind(pattern, operator, expression, semicolon, formatter)
            }
            Const(keyword_pub, keyword_const, identifier, operator, expression, semicolon) => {
                let left = keyword_pub
                    .as_deref()
                    .map_or_else(Formatter::nil, |keyword| {
                        formatter.token(keyword).append(formatter.space())
                    })
                    .append(formatter.token(keyword_const))
                    .append(formatter.space())
                    .append(formatter.token(identifier));
                assignment(left, operator, expression, semicolon, formatter)
            }
            Assign(assignee, operator, expression, semicolon) => assignment(
                assignee.format(formatter),
                operator,
                expression,
                semicolon,
                formatter,
            ),
            Function(keyword_pub, keyword_fn, identifier, parameters, body) => keyword_pub
                .as_deref()
                .map_or_else(Formatter::nil, |keyword| {
                    formatter.token(keyword).append(formatter.space())
                })
                .append(formatter.token(keyword_fn))
                .append(formatter.space())
                .append(formatter.token(identifier))
                .append(parameters.format(formatter))
                .append(formatter.space())
                .append(body.format(formatter)),
            Return(keyword, expression, semicolon) | Break(keyword, expression, semicolon) => {
                formatter
                    .token(keyword)
                    .append(
                        expression
                            .as_deref()
                            .map_or_else(Formatter::nil, |expression| {
                                formatter.space().append(expression.format(formatter))
                            }),
                    )
                    .append(formatter.token(semicolon))
            }
            Continue(keyword, semicolon) => {
                formatter.token(keyword).append(formatter.token(semicolon))
            }
            Unknown { .. } => Formatter::nil(),
        }
    }
}
