use std::fs;
use std::path::Path;
use std::time::Duration;

use indexmap::IndexMap;
use mirascript_vm::{MiraManageable, MiraNativeFn, MiraValue, RunOptions, Runtime, compile};

fn runtime(options: RunOptions) -> Runtime {
    let mut runtime = Runtime::with_options(options);
    runtime
        .insert_global(
            "t_eq",
            MiraNativeFn::new("t_eq", |runtime, args| {
                let left = args.first().cloned().unwrap_or(MiraValue::nil());
                let right = args.get(1).cloned().unwrap_or(MiraValue::nil());
                if runtime.values_equal(left, right)? {
                    Ok(MiraValue::nil())
                } else {
                    let left_string = left.as_str(runtime)?.map(str::to_owned);
                    let right_string = right.as_str(runtime)?.map(str::to_owned);
                    anyhow::bail!(
                        "assertion failed: {left:?} {left_string:?} != {right:?} {right_string:?}; message={:?}",
                        args.get(2)
                    )
                }
            }),
        )
        .unwrap();
    runtime
        .insert_global(
            "t_ne",
            MiraNativeFn::new("t_ne", |runtime, args| {
                let left = args.first().cloned().unwrap_or(MiraValue::nil());
                let right = args.get(1).cloned().unwrap_or(MiraValue::nil());
                if !runtime.values_equal(left, right)? {
                    Ok(MiraValue::nil())
                } else {
                    anyhow::bail!(
                        "assertion failed: {left:?} == {right:?}; message={:?}",
                        args.get(2)
                    )
                }
            }),
        )
        .unwrap();
    runtime
        .insert_global(
            "t_true",
            MiraNativeFn::new("t_true", |_, args| {
                let value = args.first();
                if value.and_then(MiraValue::as_boolean) == Some(true) {
                    Ok(MiraValue::nil())
                } else {
                    anyhow::bail!("expected true, got {value:?}; message={:?}", args.get(1))
                }
            }),
        )
        .unwrap();
    runtime
        .insert_global(
            "t_false",
            MiraNativeFn::new("t_false", |_, args| {
                let value = args.first();
                if value.and_then(MiraValue::as_boolean) == Some(false) {
                    Ok(MiraValue::nil())
                } else {
                    anyhow::bail!("expected false, got {value:?}; message={:?}", args.get(1))
                }
            }),
        )
        .unwrap();
    runtime
        .insert_global(
            "t_throws",
            MiraNativeFn::new("t_throws", |runtime, args| {
                let function = *args
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("t_throws requires a function"))?;
                match runtime.call(function, &[]) {
                    Ok(value) => anyhow::bail!("expected function to throw, returned {value:?}"),
                    Err(_) => Ok(MiraValue::nil()),
                }
            }),
        )
        .unwrap();
    runtime
        .insert_global("t_timeout", MiraNativeFn::ok(|_, _| MiraValue::nil()))
        .unwrap();
    runtime
        .insert_global(
            "t_never",
            MiraNativeFn::err(|_, args| {
                anyhow::anyhow!("unexpected execution: {:?}", args.first())
            }),
        )
        .unwrap();
    runtime
        .insert_global("v_array", Vec::<MiraValue>::new())
        .unwrap();
    runtime
        .insert_global("v_record", IndexMap::<String, MiraValue>::new())
        .unwrap();
    runtime.insert_global("v_nil", MiraValue::nil()).unwrap();
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
