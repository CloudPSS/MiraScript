use mirascript_core::{
    CompileConfig,
    formatter::{FormatOptions, format_document},
};

fn format_file([data]: [&str; 1]) {
    let result =
        format_document(data, &CompileConfig::default(), &FormatOptions::default()).unwrap();
    assert_eq!(result.value.unwrap(), data);
}

test_each_file::test_each_file! {
    for ["mira"] in "./tests" as format_e2e => format_file
}
