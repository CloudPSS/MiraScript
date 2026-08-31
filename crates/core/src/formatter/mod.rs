mod array_element;
mod callable;
mod else_block;
mod expression;
mod iterable;
mod list_items;
mod manager;
mod parameter_list;
mod pattern;
mod range;
mod range_format;
mod record_element;
mod statement;

mod prelude {
    pub(super) use super::manager::{FormatDoc, FormatManager as Formatter, Formattable};
}

use std::{error::Error, fmt::Display};

use crate::{CompileConfig, Compiler, Script, SourceDiagnostic, SourceRange, lexer::Trivia};

use manager::{FormatManager as Formatter, Formattable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatOptions {
    pub tab_size: usize,
    pub use_spaces: bool,
    pub line_width: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            tab_size: 2,
            use_spaces: true,
            line_width: 80,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatEdit {
    pub range: SourceRange,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatOutcome<T> {
    pub value: Option<T>,
    pub diagnostics: Vec<SourceDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    InvalidOptions(&'static str),
    InvalidRange(SourceRange),
}

impl Display for FormatError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidOptions(name) => {
                write!(formatter, "format option {name} must be greater than zero")
            }
            Self::InvalidRange(range) => write!(
                formatter,
                "invalid format range {}..{}",
                range.start, range.end
            ),
        }
    }
}

impl Error for FormatError {}

fn validate_options(options: &FormatOptions) -> Result<(), FormatError> {
    if options.tab_size == 0 {
        return Err(FormatError::InvalidOptions("tab_size"));
    }
    if options.line_width == 0 {
        return Err(FormatError::InvalidOptions("line_width"));
    }
    Ok(())
}

pub(crate) fn render_node(
    value: &impl Formattable,
    options: &FormatOptions,
    column: usize,
) -> String {
    let formatter = Formatter::new(options);
    formatter.render(value.format(&formatter), column)
}

fn format_raw(input: &Script<'_>, options: &FormatOptions) -> String {
    debug_assert!(options.tab_size > 0);
    debug_assert!(options.line_width > 0);
    let Script(statements, expression, eof) = input;
    let formatter = Formatter::new(options);
    let body = formatter.join(
        statements
            .iter()
            .map(|statement| statement.format(&formatter))
            .chain(
                expression
                    .iter()
                    .map(|expression| expression.format(&formatter)),
            ),
        formatter.hardline(),
    );
    let eof_has_comments = eof
        .leading_trivia
        .iter()
        .any(|trivia| !matches!(trivia, Trivia::NewLine(_)));
    let eof = formatter.token(eof);
    let doc = if statements.is_empty() && expression.is_none() {
        eof
    } else if eof_has_comments {
        body.append(formatter.hardline()).append(eof)
    } else {
        body.append(eof)
    };
    formatter.render(doc, 0)
}

pub fn format(input: &Script<'_>, options: &FormatOptions) -> String {
    let mut output = format_raw(input, options);
    while output.ends_with(['\r', '\n']) {
        output.pop();
    }
    output.push('\n');
    output
}

fn parse_and_format<T>(
    source: &str,
    config: &CompileConfig,
    operation: impl for<'s> FnOnce(&'s [crate::Token<'s>], &Script<'s>, &[SourceDiagnostic]) -> T,
) -> FormatOutcome<T> {
    let mut config = config.clone();
    config.trivia = true;
    let mut compiler = Compiler::new(source, &config);
    let Some(tokens) = compiler.lex() else {
        return FormatOutcome {
            value: None,
            diagnostics: compiler.diagnostics_collector.into_iter().collect(),
        };
    };
    let Some(script) = compiler.parse(&tokens) else {
        return FormatOutcome {
            value: None,
            diagnostics: compiler.diagnostics_collector.into_iter().collect(),
        };
    };
    let diagnostics = compiler
        .diagnostics_collector
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let value = operation(&tokens, &script, &diagnostics);
    FormatOutcome {
        value: Some(value),
        diagnostics,
    }
}

fn preserve_line_endings(source: &str, output: String) -> String {
    if source.contains("\r\n") {
        output.replace('\n', "\r\n")
    } else {
        output
    }
}

pub fn format_document(
    source: &str,
    config: &CompileConfig,
    options: &FormatOptions,
) -> Result<FormatOutcome<String>, FormatError> {
    validate_options(options)?;
    let outcome = parse_and_format(source, config, |_, script, diagnostics| {
        if diagnostics.iter().any(|diagnostic| diagnostic.is_error()) {
            None
        } else {
            let output = if config.input_mode == crate::InputMode::Template {
                format_raw(script, options)
            } else {
                format(script, options)
            };
            Some(preserve_line_endings(source, output))
        }
    });
    Ok(FormatOutcome {
        value: outcome.value.flatten(),
        diagnostics: outcome.diagnostics,
    })
}

pub fn format_ranges(
    source: &str,
    config: &CompileConfig,
    ranges: &[SourceRange],
    options: &FormatOptions,
) -> Result<FormatOutcome<Vec<FormatEdit>>, FormatError> {
    validate_options(options)?;
    for range in ranges {
        if range.start > range.end
            || range.end > source.len()
            || !source.is_char_boundary(range.start)
            || !source.is_char_boundary(range.end)
        {
            return Err(FormatError::InvalidRange(range.clone()));
        }
    }
    Ok(parse_and_format(
        source,
        config,
        |tokens, script, diagnostics| {
            range_format::format_ranges(source, tokens, script, ranges, diagnostics, options)
        },
    ))
}

