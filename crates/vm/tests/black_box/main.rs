use std::fs;
use std::path::Path;
use std::time::Duration;

use indexmap::IndexMap;
use mirascript_vm::{MiraManageable, MiraNativeFn, MiraValue, RunOptions, Runtime, compile};

mod harness;

fn runtime(options: RunOptions) -> Runtime {
    let mut runtime = Runtime::with_options(options);

    runtime.insert_global("t_eq", harness::T_EQ).unwrap();
    runtime.insert_global("t_ne", harness::T_NE).unwrap();
    runtime.insert_global("t_true", harness::T_TRUE).unwrap();
    runtime.insert_global("t_false", harness::T_FALSE).unwrap();
    runtime
        .insert_global("t_throws", harness::T_THROWS)
        .unwrap();
    runtime
        .insert_global("t_timeout", harness::T_TIMEOUT)
        .unwrap();
    runtime.insert_global("t_never", harness::T_NEVER).unwrap();

    runtime
        .insert_global("v_array", Vec::<MiraValue>::new())
        .unwrap();
    runtime
        .insert_global("v_record", IndexMap::<String, MiraValue>::new())
        .unwrap();
    runtime.insert_global("v_nil", MiraValue::NIL).unwrap();
    runtime.insert_global("v_true", true).unwrap();
    runtime.insert_global("v_false", false).unwrap();
    runtime.insert_global("v_number", 42).unwrap();
    runtime.insert_global("v_string", "Hello, Mira!").unwrap();
    runtime
        .insert_global("v_fn", MiraNativeFn::ok(|_, _| "I am a function"))
        .unwrap();
    runtime
        .insert_global(
            "v_fn_another",
            MiraNativeFn::ok(|_, _| "I am another function"),
        )
        .unwrap();
    runtime
        .insert_global(
            "v_module",
            MiraManageable::map_module("v_module", IndexMap::new()),
        )
        .unwrap();
    runtime
        .insert_global(
            "v_module_another",
            MiraManageable::map_module("v_module_another", IndexMap::new()),
        )
        .unwrap();
    runtime.insert_global("has_extern", false).unwrap();
    runtime
}

test_each_file::test_each_path! {
    for ["mira"] in "./tests" as black_box => black_box
}

fn black_box(path: [&Path; 1]) {
    let path = path[0].to_owned();
    let is_huge = path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("_huge.");
    let options = RunOptions {
        timeout: if is_huge {
            Duration::from_secs(10)
        } else {
            Duration::from_secs(1)
        },
        ..RunOptions::default()
    };
    let mut runtime = runtime(options);
    let source = fs::read_to_string(&path).unwrap();
    let script = compile(&source).unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
    runtime
        .run(&script)
        .unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
}
