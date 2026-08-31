use crate::parser::Pattern;

use super::prelude::*;

impl Formattable for Pattern<'_> {
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        use Pattern::*;
        match self {
            Grouping(open, pattern, close) => formatter
                .token(open)
                .append(pattern.format(formatter))
                .append(formatter.token(close)),
            Literal(operator, literal) => operator
                .as_deref()
                .map_or_else(Formatter::nil, |operator| formatter.token(operator))
                .append(formatter.token(literal)),
            Constant(constant) | Discard(constant) => formatter.token(constant),
            Relation(operator, pattern) => formatter
                .token(operator)
                .append(formatter.space())
                .append(pattern.format(formatter)),
            Range(left, operator, right) => left
                .format(formatter)
                .append(formatter.token(operator))
                .append(right.format(formatter)),
            Bind(keyword, identifier) => keyword
                .as_deref()
                .map_or_else(Formatter::nil, |keyword| {
                    formatter.token(keyword).append(formatter.space())
                })
                .append(formatter.token(identifier)),
            Record(open, items, close) => {
                let force_tail = items.len() == 1 && items[0].is_unnamed();
                formatter
                    .token(open)
                    .append(formatter.list_items(items, formatter.line_(), force_tail))
                    .append(formatter.token(close))
            }
            Array(open, items, close) => formatter
                .token(open)
                .append(formatter.list_items(items, formatter.line_(), false))
                .append(formatter.token(close)),
            SpreadDiscard(_) | Unknown { .. } => Formatter::nil(),
            And(left, keyword, right) | Or(left, keyword, right) => left
                .format(formatter)
                .append(formatter.space())
                .append(formatter.token(keyword))
                .append(formatter.line())
                .append(right.format(formatter))
                .group(),
            Not(keyword, pattern) => formatter
                .token(keyword)
                .append(formatter.space())
                .append(pattern.format(formatter)),
        }
    }
}
