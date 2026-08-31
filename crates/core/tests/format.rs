use mirascript_core::{
    CompileConfig, InputMode, SourceRange,
    formatter::{FormatEdit, FormatError, FormatOptions, format_document, format_ranges},
};

fn format_source(source: &str, line_width: usize) -> String {
    format_document(
        source,
        &CompileConfig::default(),
        &FormatOptions {
            line_width,
            ..FormatOptions::default()
        },
    )
    .unwrap()
    .value
    .unwrap()
}

fn format_range(source: &str, ranges: &[SourceRange], line_width: usize) -> Vec<FormatEdit> {
    format_ranges(
        source,
        &CompileConfig::default(),
        ranges,
        &FormatOptions {
            line_width,
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
fn template_range_formatting_does_not_replace_literal_content() {
    let source = "<p>状态: ${\n  if active { \"活跃\" } else { \"非活跃\" }\n}</p>";
    let selected =
        source.find("if active").unwrap()..source.find(" }\n}</p>").unwrap() + " }".len();
    let config = CompileConfig {
        input_mode: InputMode::Template,
        ..CompileConfig::default()
    };
    let outcome = format_ranges(source, &config, &[selected], &FormatOptions::default()).unwrap();
    assert!(
        outcome
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error())
    );
    assert_eq!(outcome.value.unwrap(), []);

    let source = "<p>状态: ${\n  if active{\"活跃\"}else{\"非活跃\"}\n}</p>";
    let selected = source.find("if active").unwrap()..source.find("\n}</p>").unwrap();
    let edits = format_ranges(source, &config, &[selected], &FormatOptions::default())
        .unwrap()
        .value
        .unwrap();
    assert_eq!(edits.len(), 1);
    let edit = &edits[0];
    assert_eq!(&source[..edit.range.start], "<p>状态: ${\n  ");
    assert_eq!(&source[edit.range.end..], "}</p>");
    assert_eq!(edit.text, "if active { \"活跃\" } else { \"非活跃\" }\n");
}

#[test]
fn template_range_layout_ignores_a_leading_blank_line() {
    let config = CompileConfig {
        input_mode: InputMode::Template,
        ..CompileConfig::default()
    };
    let expression = "if active { \"活跃\" } else { \"非活跃\" }";
    for source in [
        format!("<p>状态: ${{\n  {expression}\n}}</p>"),
        format!("<p>状态: ${{\n\n  {expression}\n}}</p>"),
    ] {
        let start = source.find(expression).unwrap();
        let selected = start..start + expression.len();
        let edits = format_ranges(&source, &config, &[selected], &FormatOptions::default())
            .unwrap()
            .value
            .unwrap();
        assert_eq!(edits, [], "{source}");
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
    assert!(
        output.contains(
            "let processed = data\n  ::filter(fn { it % 2 == 0 })\n  ::map(fn { it * 2 })"
        )
    );
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
    let edits = format_range(source, &[6..14, 8..16, 28..34], 12);
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
    let edits = format_range(source, std::slice::from_ref(&range), 80);
    assert_eq!(edits.len(), 0, "a leaf bind pattern is already canonical");

    let tuple_end = source.find('=').unwrap();
    let range = 4..tuple_end;
    let edits = format_range(source, std::slice::from_ref(&range), 80);
    assert_eq!(edits[0].text, "(first, second)");
}
