use mirascript_vm::{
    MiraArray, MiraError, MiraRecord, MiraValue, Runtime, RuntimeErrorKind, compile,
};

#[test]
fn values_and_registers_are_16_bytes() {
    assert_eq!(std::mem::size_of::<MiraValue>(), 16);
    assert_eq!(std::mem::size_of::<Option<MiraValue>>(), 16);
}

#[test]
fn executes_core_language_features() {
    let mut runtime = Runtime::new();
    assert_eq!(runtime.eval("1 + 2 * 3").unwrap(), 7.into());
    assert_eq!(
        runtime
            .eval("let value = (a: 1, b: [2, 3]); value.b[-1] + value.a",)
            .unwrap(),
        4.into(),
    );
    assert_eq!(
        runtime
            .eval(
                "fn add(a, b) { a + b } fn twice(f, x) { f(f(x)) } twice(fn (x) { add(x, 2) }, 3)",
            )
            .unwrap(),
        7.into(),
    );
    assert_eq!(
        runtime
            .eval("let mut total = 0; for value in 1..5 { total += value; } total",)
            .unwrap(),
        15.into(),
    );
}

#[test]
fn numeric_operations_preserve_number_conversion() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (r#""2" + true"#, 3),
        (r#""4" - true"#, 3),
        (r#""3" * true"#, 3),
        (r#""6" / true"#, 6),
        (r#""7" % "4""#, 3),
        (r#""2" ^ "3""#, 8),
    ] {
        assert_eq!(runtime.eval(source).unwrap(), expected.into(), "{source}");
    }
}

#[test]
fn script_and_runtime_are_independently_reusable() {
    let script = compile("value + PI").unwrap();

    let mut first = Runtime::new();
    first.insert_global("value", 1).unwrap();
    first.insert_global("PI", 10).unwrap();
    assert_eq!(first.run(&script).unwrap(), 11.into());
    first.insert_global("value", 2).unwrap();
    assert_eq!(first.run(&script).unwrap(), 12.into());

    let mut second = Runtime::new();
    second.insert_global("value", 5).unwrap();
    second.insert_global("PI", 20).unwrap();
    assert_eq!(second.run(&script).unwrap(), 25.into());

    let other = compile("value * 2").unwrap();
    assert_eq!(first.run(&other).unwrap(), 4.into());
}

#[test]
fn closures_capture_each_loop_iteration_but_cannot_escape() {
    let mut runtime = Runtime::new();
    let result = runtime.eval(
        "let mut first = nil; let mut second = nil; for value in 1..2 { if value == 1 { first = fn { value }; } else { second = fn { value }; } } first() * 10 + second()",
    )
    .unwrap();
    assert_eq!(result, 12.into());

    let error = runtime.eval("fn value { 1 } value").unwrap_err();
    assert!(matches!(
        error.as_ref(),
        MiraError::Runtime {
            kind: RuntimeErrorKind::EscapingClosure,
            ..
        }
    ));

    let error = runtime
        .eval("mod value { pub let x = 1; } value")
        .unwrap_err();
    assert!(matches!(
        error.as_ref(),
        MiraError::Runtime {
            kind: RuntimeErrorKind::EscapingClosure,
            ..
        }
    ));

    runtime.insert_fn(
        "wrap",
        mirascript_vm::MiraNativeFn::ok(|_, args| args.to_vec()),
    );
    let value = runtime.eval("wrap(fn { 1 })").unwrap();
    assert!(value.is_array());
    runtime.insert_global("value", value).unwrap();
    let inner = runtime.eval("value[0]").unwrap();
    assert!(inner.is_nil());
}

#[derive(MiraRecord)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(MiraRecord)]
struct User {
    age: u8,
    #[mira(rename = "display_name")]
    name: String,
    position: Position,
    #[mira(skip)]
    _secret: String,
}

#[derive(MiraArray)]
#[allow(dead_code)]
struct Point(f64, f64, #[mira(skip)] String);

#[test]
fn derived_values_are_live_runtime_views() {
    let mut runtime = Runtime::new();
    let user = runtime
        .insert_record(User {
            age: 42,
            name: "Ada".into(),
            position: Position { x: 3.0, y: 4.0 },
            _secret: "token".into(),
        })
        .unwrap();
    let point = runtime
        .insert_array(Point(3.0, 4.0, "metadata".into()))
        .unwrap();
    runtime
        .insert_global("user", MiraValue::Record(user.erase_record()))
        .unwrap();
    runtime
        .insert_global("point", MiraValue::Array(point.erase_array()))
        .unwrap();

    assert_eq!(
        runtime
            .eval("user.age + user.position.x + point[0] + point[-1]",)
            .unwrap(),
        52.into(),
    );
    let name = runtime.eval("user.display_name").unwrap();
    assert_eq!(name.as_str(&runtime).unwrap(), Some("Ada"));

    let user_value = runtime.get_record_mut(user).unwrap();
    user_value.age = 7;
    user_value.position.x = 10.0;
    assert_eq!(
        runtime.eval("user.age + user.position.x").unwrap(),
        17.into(),
    );

    let wrong = unsafe { user.erase_record().upcast::<Position>() };
    let error = runtime.get_record(wrong).err().unwrap();
    assert!(matches!(
        error.as_ref(),
        MiraError::Runtime {
            kind: RuntimeErrorKind::HandleTypeMismatch { category: "record" },
            ..
        }
    ));
}

#[test]
fn foreign_handles_are_rejected() {
    let mut first = Runtime::new();
    let value = first.insert("owned by first").unwrap();
    let mut second = Runtime::new();
    let error = second.insert_global("foreign", value).unwrap_err();
    assert!(matches!(
        error.as_ref(),
        MiraError::Runtime {
            kind: RuntimeErrorKind::ForeignHandle,
            ..
        }
    ));
}

#[test]
fn errors_preserve_diagnostics_and_runtime_context() {
    assert!(matches!(
        compile("let =").unwrap_err().as_ref(),
        MiraError::Compile { diagnostics } if !diagnostics.is_empty()
    ));

    let mut runtime = Runtime::new();
    match runtime
        .eval("fn outer { panic('boom') } outer()")
        .unwrap_err()
        .as_ref()
    {
        MiraError::Runtime { trace, .. } => {
            assert!(trace.function.is_some());
            assert!(trace.offset.is_some());
            assert!(!trace.stack.is_empty());
        }
        error => panic!("expected runtime error with context, got {error:?}"),
    }
}
