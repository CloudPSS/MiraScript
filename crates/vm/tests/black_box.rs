use std::fs;
use std::path::Path;
use std::time::Duration;

use indexmap::IndexMap;
use mirascript_vm::{MiraAny, MiraContext, MiraModule, MiraNativeFn, RunOptions, compile};

fn context() -> MiraContext {
    let mut context = MiraContext::new();
    context.insert(
        "t_eq",
        MiraNativeFn::new("t_eq", |_, args| {
            let left = args.first().cloned().unwrap_or(MiraAny::Nil);
            let right = args.get(1).cloned().unwrap_or(MiraAny::Nil);
            if host_equal(&left, &right) {
                Ok(MiraAny::Nil)
            } else {
                anyhow::bail!(
                    "assertion failed: {left:?} != {right:?}; message={:?}",
                    args.get(2)
                )
            }
        }),
    );
    context.insert(
        "t_ne",
        MiraNativeFn::new("t_ne", |_, args| {
            let left = args.first().cloned().unwrap_or(MiraAny::Nil);
            let right = args.get(1).cloned().unwrap_or(MiraAny::Nil);
            if !host_equal(&left, &right) {
                Ok(MiraAny::Nil)
            } else {
                anyhow::bail!(
                    "assertion failed: {left:?} == {right:?}; message={:?}",
                    args.get(2)
                )
            }
        }),
    );
    context.insert(
        "t_true",
        MiraNativeFn::new("t_true", |_, args| match args.first() {
            Some(MiraAny::Boolean(true)) => Ok(MiraAny::Nil),
            value => anyhow::bail!("expected true, got {value:?}; message={:?}", args.get(1)),
        }),
    );
    context.insert(
        "t_false",
        MiraNativeFn::new("t_false", |_, args| match args.first() {
            Some(MiraAny::Boolean(false)) => Ok(MiraAny::Nil),
            value => anyhow::bail!("expected false, got {value:?}; message={:?}", args.get(1)),
        }),
    );
    context.insert(
        "t_throws",
        MiraNativeFn::new("t_throws", |call, args| {
            let function = args
                .first()
                .ok_or_else(|| anyhow::anyhow!("t_throws requires a function"))?;
            match call.call(function, &[]) {
                Ok(value) => anyhow::bail!("expected function to throw, returned {value:?}"),
                Err(_) => Ok(MiraAny::Nil),
            }
        }),
    );
    // Infinite-loop checks are covered by focused RunOptions tests. The shared black-box suite
    // defers these calls in the TypeScript runner as well.
    context.insert(
        "t_timeout",
        MiraNativeFn::ok(|_, _| MiraAny::Nil).with_name("t_timeout"),
    );
    context.insert(
        "t_never",
        MiraNativeFn::err(|_, args| anyhow::anyhow!("unexpected execution: {:?}", args.first()))
            .with_name("t_never"),
    );
    context.insert("v_array", MiraAny::Array(Vec::new().into()));
    context.insert("v_record", MiraAny::Record(IndexMap::new().into()));
    context.insert("v_nil", MiraAny::Nil);
    context.insert("v_true", true);
    context.insert("v_false", false);
    context.insert("v_number", 42);
    context.insert("v_string", "Hello, Mira!");
    context.insert("v_fn", MiraNativeFn::ok(|_, _| "I am a function"));
    context.insert(
        "v_fn_another",
        MiraNativeFn::ok(|_, _| "I am another function"),
    );
    context.insert("v_module", MiraModule::new("v_module", IndexMap::new()));
    context.insert(
        "v_module_another",
        MiraModule::new("v_module_another", IndexMap::new()),
    );
    context.insert("has_extern", false);
    context
}

fn host_equal(left: &MiraAny, right: &MiraAny) -> bool {
    match (left, right) {
        (MiraAny::Number(left), MiraAny::Number(right)) => {
            left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan())
        }
        (MiraAny::Array(left), MiraAny::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| host_equal(left, right))
        }
        (MiraAny::Record(left), MiraAny::Record(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .all(|(key, left)| right.get(key).is_some_and(|right| host_equal(left, right)))
        }
        _ => left == right,
    }
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
    let context = context();
    let options = RunOptions {
        timeout: if is_huge {
            Duration::from_secs(10)
        } else {
            Duration::from_secs(1)
        },
        ..RunOptions::default()
    };
    let source = fs::read_to_string(&path).unwrap();
    let script = compile(&source).unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
    script
        .run_with(&context, &options)
        .unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
}
