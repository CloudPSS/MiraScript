use crate::{
    Script, SourceDiagnostic, SourceRange, Token, TokenKind,
    formatter::{FormatEdit, FormatOptions, preserve_line_endings, render_node},
    parser::{
        ArrayElementBase, AstWalker, Callable, Expression, Iterable, ParameterList, Pattern,
        RecordElementBase, Statement,
    },
};

#[derive(Clone, Copy)]
enum FormatNode<'s, 'a> {
    Script(&'a Script<'s>),
    Statement(&'a Statement<'s>),
    Expression(&'a Expression<'s>),
    Pattern(&'a Pattern<'s>),
}

impl FormatNode<'_, '_> {
    fn range(self) -> SourceRange {
        match self {
            Self::Script(value) => value.range(),
            Self::Statement(value) => value.range(),
            Self::Expression(value) => value.range(),
            Self::Pattern(value) => value.range(),
        }
    }
}

fn contains(outer: &SourceRange, inner: &SourceRange) -> bool {
    outer.start <= inner.start && outer.end >= inner.end
}

fn collect_statement<'s, 'a>(statement: &'a Statement<'s>, nodes: &mut Vec<FormatNode<'s, 'a>>) {
    nodes.push(FormatNode::Statement(statement));
    use Statement::*;
    match statement {
        Expression(expression, _) | BlockExpression(expression) | Module(_, _, _, expression) => {
            collect_expression(expression, nodes);
        }
        Bind(_, _, pattern, _, expression, _) | Rebind(pattern, _, expression, _) => {
            collect_pattern(pattern, nodes);
            collect_expression(expression, nodes);
        }
        Const(_, _, _, _, expression, _) => collect_expression(expression, nodes),
        Assign(left, _, right, _) => {
            collect_expression(left, nodes);
            collect_expression(right, nodes);
        }
        Function(_, _, _, parameters, body) => {
            if let Some(parameters) = parameters {
                collect_parameters(parameters, nodes);
            }
            collect_expression(body, nodes);
        }
        Return(_, expression, _) | Break(_, expression, _) => {
            if let Some(expression) = expression {
                collect_expression(expression, nodes);
            }
        }
        Empty(_) | Continue(_, _) | Unknown { .. } => {}
    }
}

fn collect_parameters<'s, 'a>(
    parameters: &'a ParameterList<'s>,
    nodes: &mut Vec<FormatNode<'s, 'a>>,
) {
    for parameter in &parameters.1 {
        match &**parameter {
            ArrayElementBase::Element(pattern) | ArrayElementBase::Spread(_, pattern) => {
                collect_pattern(pattern, nodes);
            }
        }
    }
}

fn collect_pattern<'s, 'a>(pattern: &'a Pattern<'s>, nodes: &mut Vec<FormatNode<'s, 'a>>) {
    nodes.push(FormatNode::Pattern(pattern));
    use Pattern::*;
    match pattern {
        Grouping(_, pattern, _) | Relation(_, pattern) | Not(_, pattern) => {
            collect_pattern(pattern, nodes);
        }
        Range(left, _, right) | And(left, _, right) | Or(left, _, right) => {
            collect_pattern(left, nodes);
            collect_pattern(right, nodes);
        }
        Record(_, items, _) => {
            for item in items {
                match &**item {
                    RecordElementBase::Named(_, _, pattern)
                    | RecordElementBase::OmitNamed(_, pattern)
                    | RecordElementBase::Unnamed(pattern)
                    | RecordElementBase::Spread(_, pattern) => collect_pattern(pattern, nodes),
                    RecordElementBase::InterpolateNamed(name, _, pattern) => {
                        collect_pattern(name, nodes);
                        collect_pattern(pattern, nodes);
                    }
                }
            }
        }
        Array(_, items, _) => {
            for item in items {
                match &**item {
                    ArrayElementBase::Element(pattern) | ArrayElementBase::Spread(_, pattern) => {
                        collect_pattern(pattern, nodes);
                    }
                }
            }
        }
        Literal(..) | Constant(_) | Discard(_) | Bind(..) | SpreadDiscard(_) | Unknown { .. } => {}
    }
}

fn collect_callable<'s, 'a>(callable: &'a Callable<'s>, nodes: &mut Vec<FormatNode<'s, 'a>>) {
    if let Callable::Expression(expression) = callable {
        collect_expression(expression, nodes);
    }
}

fn collect_expression<'s, 'a>(expression: &'a Expression<'s>, nodes: &mut Vec<FormatNode<'s, 'a>>) {
    nodes.push(FormatNode::Expression(expression));
    use Expression::*;
    match expression {
        Grouping(_, expression, _)
        | NonNil(expression, _)
        | Prefix(_, expression)
        | Loop(_, expression) => collect_expression(expression, nodes),
        Function(_, parameters, expression) => {
            if let Some(parameters) = parameters {
                collect_parameters(parameters, nodes);
            }
            collect_expression(expression, nodes);
        }
        InterpolatedString(_, expressions) => {
            for expression in expressions {
                collect_expression(expression, nodes);
            }
        }
        Record(_, items, _) => {
            for item in items {
                match &**item {
                    RecordElementBase::Named(_, _, value)
                    | RecordElementBase::OmitNamed(_, value)
                    | RecordElementBase::Unnamed(value)
                    | RecordElementBase::Spread(_, value) => collect_expression(value, nodes),
                    RecordElementBase::InterpolateNamed(name, _, value) => {
                        collect_expression(name, nodes);
                        collect_expression(value, nodes);
                    }
                }
            }
        }
        Array(_, items, _) => {
            for item in items {
                match &**item {
                    ArrayElementBase::Element(iterable) => match iterable.as_ref() {
                        Iterable::Value(value) => collect_expression(value, nodes),
                        Iterable::Range(range) => {
                            collect_expression(&range.0, nodes);
                            collect_expression(&range.2, nodes);
                        }
                    },
                    ArrayElementBase::Spread(_, value) => collect_expression(value, nodes),
                }
            }
        }
        TaggedString(callable, expression) => {
            collect_expression(callable, nodes);
            collect_expression(expression, nodes);
        }
        Call(callable, _, items, _) => {
            collect_callable(callable, nodes);
            for item in items {
                match &**item {
                    ArrayElementBase::Element(value) | ArrayElementBase::Spread(_, value) => {
                        collect_expression(value, nodes);
                    }
                }
            }
        }
        Extension(expression, _, callable, _, items, _) => {
            collect_expression(expression, nodes);
            collect_callable(callable, nodes);
            for item in items {
                match &**item {
                    ArrayElementBase::Element(value) | ArrayElementBase::Spread(_, value) => {
                        collect_expression(value, nodes);
                    }
                }
            }
        }
        Access(expression, _, _) => collect_expression(expression, nodes),
        Index(expression, _, field, _) => {
            collect_expression(expression, nodes);
            collect_expression(field, nodes);
        }
        Slice(expression, _, left, _, right, _) => {
            collect_expression(expression, nodes);
            if let Some(left) = left {
                collect_expression(left, nodes);
            }
            if let Some(right) = right {
                collect_expression(right, nodes);
            }
        }
        Infix(left, _, right) | Cond(left, _, right, _, _) => {
            collect_expression(left, nodes);
            collect_expression(right, nodes);
            if let Cond(_, _, _, _, otherwise) = expression {
                collect_expression(otherwise, nodes);
            }
        }
        Is(expression, _, pattern) => {
            collect_expression(expression, nodes);
            collect_pattern(pattern, nodes);
        }
        While(_, condition, body, else_block) | If(_, condition, body, else_block) => {
            collect_expression(condition, nodes);
            collect_expression(body, nodes);
            if let Some(else_block) = else_block {
                collect_expression(&else_block.1, nodes);
            }
        }
        ForIn(_, pattern, _, iterable, body, else_block) => {
            collect_pattern(pattern, nodes);
            match iterable.as_ref() {
                Iterable::Value(value) => collect_expression(value, nodes),
                Iterable::Range(range) => {
                    collect_expression(&range.0, nodes);
                    collect_expression(&range.2, nodes);
                }
            }
            collect_expression(body, nodes);
            if let Some(else_block) = else_block {
                collect_expression(&else_block.1, nodes);
            }
        }
        Block(_, statements, expression, _) => {
            for statement in statements {
                collect_statement(statement, nodes);
            }
            if let Some(expression) = expression {
                collect_expression(expression, nodes);
            }
        }
        Match(_, matcher, _, cases, _) => {
            collect_expression(matcher, nodes);
            for case in cases {
                collect_pattern(&case.1, nodes);
                if let Some((_, guard)) = &case.2 {
                    collect_expression(guard, nodes);
                }
                collect_expression(&case.3, nodes);
            }
        }
        Literal(_) | Variable(_) | Unknown { .. } => {}
    }
}

fn collect_tokens<'a, 's>(tokens: &'a [Token<'s>], output: &mut Vec<&'a Token<'s>>) {
    for token in tokens {
        output.push(token);
        if let TokenKind::InterpolatedString(parts, _) = &token.kind {
            for (_, tokens, _) in parts {
                collect_tokens(tokens, output);
            }
        }
    }
}

fn expanded_range(tokens: &[Token<'_>], range: SourceRange) -> SourceRange {
    let mut all_tokens = Vec::new();
    collect_tokens(tokens, &mut all_tokens);
    let first = all_tokens
        .iter()
        .copied()
        .filter(|token| {
            token.range.start <= range.start
                && (token.range.end > range.start || token.range == range)
        })
        .min_by_key(|token| token.range.len());
    let last = all_tokens
        .iter()
        .copied()
        .filter(|token| {
            token.range.end >= range.end && (token.range.start < range.end || token.range == range)
        })
        .min_by_key(|token| token.range.len());
    match (first, last) {
        (Some(first), Some(last)) => first.full_range().start..last.full_range().end,
        _ => range,
    }
}

fn source_column(source: &str, offset: usize) -> usize {
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    source[line_start..offset].chars().count()
}

fn render(node: FormatNode<'_, '_>, options: &FormatOptions, column: usize) -> String {
    match node {
        FormatNode::Script(script) => super::format(script, options),
        FormatNode::Statement(statement) => render_node(statement, options, column),
        FormatNode::Expression(expression) => render_node(expression, options, column),
        FormatNode::Pattern(pattern) => render_node(pattern, options, column),
    }
}

pub(super) fn format_ranges(
    source: &str,
    tokens: &[Token<'_>],
    script: &Script<'_>,
    ranges: &[SourceRange],
    diagnostics: &[SourceDiagnostic],
    options: &FormatOptions,
) -> Vec<FormatEdit> {
    let mut nodes = vec![FormatNode::Script(script)];
    for statement in &script.0 {
        collect_statement(statement, &mut nodes);
    }
    if let Some(expression) = &script.1 {
        collect_expression(expression, &mut nodes);
    }

    let mut selected = Vec::new();
    for requested in ranges {
        let Some(node) = nodes
            .iter()
            .copied()
            .filter(|node| contains(&node.range(), requested))
            .min_by_key(|node| node.range().len())
        else {
            continue;
        };
        selected.push(node);
    }

    selected.sort_by_key(|node| (node.range().start, node.range().end));
    selected.dedup_by_key(|node| node.range());
    while let Some(index) = selected
        .windows(2)
        .position(|pair| pair[0].range().end > pair[1].range().start)
    {
        let union = selected[index].range().start..selected[index + 1].range().end;
        let parent = nodes
            .iter()
            .copied()
            .filter(|node| contains(&node.range(), &union))
            .min_by_key(|node| node.range().len())
            .unwrap_or(FormatNode::Script(script));
        selected.splice(index..=index + 1, [parent]);
    }

    selected
        .into_iter()
        .filter_map(|node| {
            let node_range = node.range();
            let range = expanded_range(tokens, node_range.clone());
            if diagnostics.iter().any(|diagnostic| {
                diagnostic.is_error()
                    && diagnostic.range.start < range.end
                    && range.start < diagnostic.range.end
            }) {
                return None;
            }
            let mut text = render(node, options, source_column(source, node_range.start));
            if range.end <= source.len()
                && source[range.clone()].ends_with('\n')
                && !text.ends_with('\n')
            {
                text.push('\n');
            }
            text = preserve_line_endings(source, text);
            (source[range.clone()] != text).then_some(FormatEdit { range, text })
        })
        .collect()
}
