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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InputMode;

    fn format_source(source: &str, width: usize) -> String {
        format_document(
            source,
            &CompileConfig::default(),
            &FormatOptions {
                line_width: width,
                ..FormatOptions::default()
            },
        )
        .unwrap()
        .value
        .unwrap()
    }

    #[test]
    fn layout_uses_available_width_instead_of_item_count() {
        assert_eq!(
            format_source("let x=[1,2,3,4,5,6,7,8];", 80),
            "let x = [1, 2, 3, 4, 5, 6, 7, 8];\n"
        );
        assert_eq!(
            format_source("let x=[1111,2222,3333,4444];", 20),
            "let x = [\n  1111,\n  2222,\n  3333,\n  4444,\n];\n"
        );
    }

    #[test]
    fn long_infix_chains_break_after_operators() {
        assert_eq!(
            format_source("let x=first+second+third+fourth;", 20),
            "let x = first +\n  second +\n  third +\n  fourth;\n"
        );
    }

    #[test]
    fn formatting_is_idempotent_and_preserves_crlf() {
        let source = "let x=[1,2,3,4];\r\n";
        let once = format_source(source, 16);
        assert!(once.contains("\r\n"));
        assert_eq!(format_source(&once, 16), once);
    }

    #[test]
    fn comments_blank_lines_and_literals_are_normalized_idempotently() {
        let source = "//heading\n\n\nlet x=0xff_ff;//tail\n";
        let expected = "// heading\n\nlet x = 0xFFFF; // tail\n";
        assert_eq!(format_source(source, 80), expected);
        assert_eq!(format_source(expected, 80), expected);
    }

    #[test]
    fn preserves_one_blank_line_between_statements() {
        let source = "let first=1;\n\n\nlet second=2;\n";
        let expected = "let first = 1;\n\nlet second = 2;\n";
        assert_eq!(format_source(source, 80), expected);
        assert_eq!(format_source(expected, 80), expected);
    }

    #[test]
    fn line_comments_do_not_add_indented_blank_lines() {
        let source = "let value=if true {\n  1; // kept\n};\n";
        let output = format_source(source, 80);
        assert_eq!(output, "let value = if true {\n  1; // kept\n};\n");
        assert!(output.lines().all(|line| line.trim_end() == line));
    }

    #[test]
    fn comments_before_a_closing_brace_stay_inside_the_block() {
        let source = "for item in items {\nif item { item; }\n// kept inside\n}\n";
        let output = format_source(source, 80);
        assert_eq!(
            output,
            "for item in items {\n  if item {\n    item;\n  }\n  // kept inside\n}\n"
        );
        assert_eq!(format_source(&output, 80), output);
    }

    #[test]
    fn block_comment_line_breaks_are_not_rendered_as_spaces() {
        let source = "/* first */\n// second\nlet value=1;\n";
        let expected = "/* first */\n// second\nlet value = 1;\n";
        assert_eq!(format_source(source, 80), expected);
        assert_eq!(format_source(expected, 80), expected);
    }

    #[test]
    fn multiline_strings_do_not_accumulate_renderer_indentation() {
        let source = "let value=`first\nsecond\nthird`;\n";
        let once = format_source(source, 80);
        assert_eq!(once, "let value = `first\nsecond\nthird`;\n");
        assert_eq!(format_source(&once, 80), once);
    }

    #[test]
    fn comments_before_eof_remain_on_their_own_line() {
        let source = "let value=1;\n// final note\n";
        let expected = "let value = 1;\n// final note\n";
        assert_eq!(format_source(source, 80), expected);
        assert_eq!(format_source(expected, 80), expected);
    }

    #[test]
    fn ordinary_block_comments_do_not_turn_into_doc_comments() {
        let source = "/*\nfirst\nsecond\n*/\nlet value=1;\n";
        let once = format_source(source, 80);
        assert!(once.starts_with("/*\n"));
        assert!(!once.starts_with("/**\n"));
        assert_eq!(format_source(&once, 80), once);
    }

    #[test]
    fn template_lines_keep_their_column_context_without_trailing_spaces() {
        let source = "<p>value: ${\n    let value=1;\n    value\n}</p>  \n";
        let config = CompileConfig {
            input_mode: InputMode::Template,
            ..CompileConfig::default()
        };
        let output = format_document(source, &config, &FormatOptions::default())
            .unwrap()
            .value
            .unwrap();
        assert_eq!(output, "<p>value: ${\n  let value = 1;\n  value\n}</p>  \n");
        assert_eq!(
            format_document(&output, &config, &FormatOptions::default())
                .unwrap()
                .value
                .unwrap(),
            output
        );
    }

    #[test]
    fn template_preserves_the_exact_end_of_file() {
        let config = CompileConfig {
            input_mode: InputMode::Template,
            ..CompileConfig::default()
        };
        for source in [
            "<p>Hello</p>",
            "<p>Hello</p>\n",
            "<p>Hello</p>\n\n",
            "<p>Hello</p>  ",
        ] {
            let output = format_document(source, &config, &FormatOptions::default())
                .unwrap()
                .value
                .unwrap();
            assert_eq!(output, source);
        }
    }

    #[test]
    fn tab_indentation_uses_the_configured_tab_size() {
        let outcome = format_document(
            "let x=[1111,2222,3333];",
            &CompileConfig::default(),
            &FormatOptions {
                tab_size: 4,
                use_spaces: false,
                line_width: 12,
            },
        )
        .unwrap();
        assert_eq!(
            outcome.value.unwrap(),
            "let x = [\n\t1111,\n\t2222,\n\t3333,\n];\n"
        );
    }

    #[test]
    fn assignments_keep_the_expression_head_after_the_operator() {
        let source = "let record=(first:1111,second:2222);\nlet callback=fn (x) { x };\nlet result=for x in xs { x; };\n";
        let output = format_source(source, 30);
        assert!(output.contains("let record = (\n"));
        assert!(output.contains("let callback = fn (x)"), "{output}");
        assert!(output.contains("let result = for x in xs {"), "{output}");
        assert!(!output.contains("=\n"));
    }

    #[test]
    fn extension_chains_break_before_colon_colon() {
        let source = "let processed=data::filter(fn { it%2==0 })::map(fn { it*2 })::filter(fn { it>10 })::sum();\n";
        let output = format_source(source, 48);
        assert!(output.contains(
            "let processed = data\n  ::filter(fn { it % 2 == 0 })\n  ::map(fn { it * 2 })"
        ));
        assert!(!output.contains("::filter(\n"));
    }

    #[test]
    fn if_else_branches_choose_the_same_block_layout() {
        let source = "if valid { (valid:true,user:user) } else { (valid:false,errors:errors) }\n";
        let output = format_source(source, 54);
        assert_eq!(
            output,
            "if valid {\n  (valid: true, user: user)\n} else {\n  (valid: false, errors: errors)\n}\n"
        );
    }

    #[test]
    fn multiline_callback_does_not_force_call_parentheses_to_break() {
        let source = "t_throws(fn {\nlet value=nil;\nvalue();\n});\n";
        let output = format_source(source, 80);
        assert_eq!(
            output,
            "t_throws(fn {\n  let value = nil;\n  value();\n});\n"
        );
    }

    #[test]
    fn deeply_indented_short_calls_stay_inline() {
        let source = format!("{}return f(it);{}\n", "fn f { ".repeat(35), " }".repeat(35));
        let output = format_source(&source, 80);
        assert!(output.contains("return f(it);"));
        assert!(!output.contains("return f(\n"));
    }

    #[test]
    fn short_interpolations_stay_inline_even_with_a_long_suffix() {
        let source = "let html=`a very long literal prefix ${ score.points } and a long suffix`;\n";
        let output = format_source(source, 40);
        assert!(output.contains("${ score.points }"), "{output}");
    }

    #[test]
    fn single_line_doc_comments_keep_the_doc_marker() {
        let source = "/** The imaginary unit. */\nlet value=1;\n";
        let output = format_source(source, 80);
        assert!(output.starts_with("/** The imaginary unit. */\n"));
        assert_eq!(format_source(&output, 80), output);
    }

    #[test]
    fn zero_sized_options_are_rejected() {
        let error = format_document(
            "let x = 1;",
            &CompileConfig::default(),
            &FormatOptions {
                tab_size: 0,
                ..FormatOptions::default()
            },
        )
        .unwrap_err();
        assert_eq!(error, FormatError::InvalidOptions("tab_size"));
    }

    #[test]
    fn invalid_ranges_are_rejected_without_panicking() {
        let range = 0..usize::MAX;
        let error = format_ranges(
            "let x = 1;",
            &CompileConfig::default(),
            std::slice::from_ref(&range),
            &FormatOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(error, FormatError::InvalidRange(_)));
    }

    #[test]
    fn range_formatting_returns_non_overlapping_edits() {
        let source = "let x=[1,2,3,4];\nlet y=[5,6,7,8];\n";
        let outcome = format_ranges(
            source,
            &CompileConfig::default(),
            &[6..14, 8..16, 28..34],
            &FormatOptions {
                line_width: 12,
                ..FormatOptions::default()
            },
        )
        .unwrap();
        let edits = outcome.value.unwrap();
        assert!(!edits.is_empty());
        assert!(
            edits
                .windows(2)
                .all(|pair| pair[0].range.end <= pair[1].range.start)
        );
    }

    #[test]
    fn range_formatting_isolates_errors_outside_the_selected_unit() {
        let source = "let good=[1,2,3,4];\nlet bad=[1,2;\n";
        let range = 9..18;
        let outcome = format_ranges(
            source,
            &CompileConfig::default(),
            std::slice::from_ref(&range),
            &FormatOptions::default(),
        )
        .unwrap();
        assert!(
            outcome
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.is_error())
        );
        let edits = outcome.value.unwrap();
        assert_eq!(edits.len(), 1);
        assert!(edits[0].range.end <= source.find("let bad").unwrap());
    }

    #[test]
    fn range_formatting_can_select_a_pattern() {
        let source = "let (first,second)=[1,2];\n";
        let selection = source.find("second").unwrap();
        let range = selection..selection + "second".len();
        let outcome = format_ranges(
            source,
            &CompileConfig::default(),
            std::slice::from_ref(&range),
            &FormatOptions::default(),
        )
        .unwrap();
        let edits = outcome.value.unwrap();
        assert_eq!(edits.len(), 0, "a leaf bind pattern is already canonical");

        let tuple_end = source.find('=').unwrap();
        let range = 4..tuple_end;
        let outcome = format_ranges(
            source,
            &CompileConfig::default(),
            std::slice::from_ref(&range),
            &FormatOptions::default(),
        )
        .unwrap();
        assert_eq!(outcome.value.unwrap()[0].text, "(first, second)");
    }
}
