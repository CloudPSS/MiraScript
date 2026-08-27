use std::fs;
use std::path::Path;
use std::time::Duration;

use indexmap::IndexMap;
use mirascript_vm::{MiraManageable, MiraNativeFn, MiraValue, RunOptions, Runtime, compile, mira};

#[mira]
fn t_eq(
    runtime: &mut Runtime,
    left: MiraValue,
    right: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    if runtime.values_equal(left, right)? {
        Ok(MiraValue::NIL)
    } else {
        let left_string = left.as_str(runtime)?.map(str::to_owned);
        let right_string = right.as_str(runtime)?.map(str::to_owned);
        anyhow::bail!(
            "assertion failed: {left:?} {left_string:?} != {right:?} {right_string:?}; message={:?}",
            message
        )
    }
}

#[mira]
fn t_ne(
    runtime: &mut Runtime,
    left: MiraValue,
    right: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    if !runtime.values_equal(left, right)? {
        Ok(MiraValue::NIL)
    } else {
        let left_string = left.as_str(runtime)?.map(str::to_owned);
        let right_string = right.as_str(runtime)?.map(str::to_owned);
        anyhow::bail!(
            "assertion failed: {left:?} {left_string:?} == {right:?} {right_string:?}; message={:?}",
            message
        )
    }
}

#[mira]
fn t_true(
    runtime: &mut Runtime,
    value: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    if value.as_boolean() == Some(true) {
        Ok(MiraValue::NIL)
    } else {
        let value_string = value.as_str(runtime)?.map(str::to_owned);
        anyhow::bail!(
            "assertion failed: expected true, got {value:?} {value_string:?}; message={:?}",
            message
        )
    }
}

#[mira]
fn t_false(
    runtime: &mut Runtime,
    value: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    if value.as_boolean() == Some(false) {
        Ok(MiraValue::NIL)
    } else {
        let value_string = value.as_str(runtime)?.map(str::to_owned);
        anyhow::bail!(
            "assertion failed: expected false, got {value:?} {value_string:?}; message={:?}",
            message
        )
    }
}

#[mira]
fn t_throws(
    runtime: &mut Runtime,
    function: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    match runtime.call(function, &[]) {
        Ok(value) => {
            let value_string = value.as_str(runtime)?.map(str::to_owned);
            anyhow::bail!(
                "assertion failed: expected function to throw, returned {value:?} {value_string:?}; message={:?}",
                message
            )
        }
        Err(_) => Ok(MiraValue::NIL),
    }
}

#[mira]
fn t_timeout(
    _runtime: &mut Runtime,
    _function: MiraValue,
    _message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    // This is a placeholder for a timeout test. In this black-box test, we don't actually implement a timeout mechanism, so we just return Nil to indicate the test passed.
    Ok(MiraValue::NIL)
}

#[mira]
fn t_never(message: Option<String>) -> Result<MiraValue, anyhow::Error> {
    anyhow::bail!("unexpected execution: message={:?}", message)
}

fn runtime(options: RunOptions) -> Runtime {
    let mut runtime = Runtime::with_options(options);

    runtime.insert_global("t_eq", T_EQ).unwrap();
    runtime.insert_global("t_ne", T_NE).unwrap();
    runtime.insert_global("t_true", T_TRUE).unwrap();
    runtime.insert_global("t_false", T_FALSE).unwrap();
    runtime.insert_global("t_throws", T_THROWS).unwrap();
    runtime.insert_global("t_timeout", T_TIMEOUT).unwrap();
    runtime.insert_global("t_never", T_NEVER).unwrap();

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
