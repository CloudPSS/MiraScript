use winnow::{
    combinator::{alt, peek, repeat},
    token::one_of,
};

use super::{
    basic_expressions::interpolation,
    expressions::expression_or_insert,
    helper::{token, token_or_insert},
    prelude::*,
};

/// Check for the start of a JSON object-like expression
/// Always wraps in `peek` or `not` to avoid consuming input
pub(super) fn json_start<'s>(i: &mut Input<'s>) -> Result<()> {
    (
        // { "xxx" :
        token(Operator::OpenBrace),
        one_of(|t: &Token<'s>| {
            matches!(
                &t.kind,
                &TokenKind::InterpolatedString(..) | TokenKind::String(..)
            )
        }),
        token(Operator::Colon),
    )
        .value(())
        .parse_next(i)
}

enum JsonFieldName<'s, 'a> {
    Literal(TokenRef<'s>),
    Interpolated(ABox<'a, Expression<'s, 'a>>),
}
struct JsonElement<'s, 'a> {
    key: JsonFieldName<'s, 'a>,
    colon: TokenRef<'s>,
    value: ABox<'a, Expression<'s, 'a>>,
    comma: TokenRef<'s>,
}
fn json_field_name<'s: 'a, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<JsonFieldName<'s, 'a>> {
    alt((
        // "xxx"
        one_of(|t: &Token<'s>| matches!(&t.kind, &TokenKind::String(..)))
            .map(|t: &Token<'s>| JsonFieldName::Literal(t.into())),
        // `$xxx`
        (|i: &mut Input<'s>| interpolation(arena, i))
            .map(|e| JsonFieldName::Interpolated(arena.alloc(e))),
    ))
    .parse_next(i)
}

pub(super) fn json_expression<'s: 'a, 'a>(
    arena: &'a AstArena,
    i: &mut Input<'s>,
) -> Result<Expression<'s, 'a>> {
    // Peek to see if it's a JSON object-like expression
    peek(json_start).parse_next(i)?;
    let open = token(Operator::OpenBrace).parse_next(i)?;
    let elements: Vec<_> = repeat(
        1..,
        (
            |i: &mut Input<'s>| json_field_name(arena, i),
            token_or_insert(Operator::Colon, DiagnosticCode::MissingColon),
            expression_or_insert(arena, |t| {
                *t == Operator::Comma || *t == Operator::CloseBrace
            }),
            token_or_insert(Operator::Comma, DiagnosticCode::MissingComma),
        )
            .map(|(key, colon, value, comma)| JsonElement {
                key,
                colon,
                value: arena.alloc(value),
                comma,
            }),
    )
    .parse_next(i)?;
    let close =
        token_or_insert(Operator::CloseBrace, DiagnosticCode::MissingCloseBrace).parse_next(i)?;

    let el_count = elements.len();
    let elements = elements
        .into_iter()
        .enumerate()
        .map(|(idx, e)| {
            let el = match e.key {
                JsonFieldName::Literal(k) => RecordElementBase::Named(k, e.colon, e.value),
                JsonFieldName::Interpolated(k) => {
                    RecordElementBase::InterpolateNamed(k, e.colon, e.value)
                }
            };
            if idx == el_count - 1 && e.comma.is_unknown() {
                // Remove the trailing comma diagnostic for the last element
                RecordElement::new(arena, el)
            } else {
                RecordElement::new_with_comma(arena, el, e.comma)
            }
        })
        .collect();
    Ok(Expression::Record(open, elements, close))
}
