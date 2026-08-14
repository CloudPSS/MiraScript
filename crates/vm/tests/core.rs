use mira_vm::{
    MiraAny, MiraArray, MiraContext, MiraError, MiraExtern, MiraRecord, MiraShared, compile, eval,
};

#[test]
fn executes_core_language_features() {
    let context = MiraContext::empty();
    assert_eq!(eval("1 + 2 * 3", &context).unwrap(), MiraAny::from(7));
    assert_eq!(
        eval(
            "let value = (a: 1, b: [2, 3]); value.b[-1] + value.a",
            &context
        )
        .unwrap(),
        MiraAny::from(4),
    );
    assert_eq!(
        eval(
            "fn add(a, b) { a + b } fn twice(f, x) { f(f(x)) } twice(fn (x) { add(x, 2) }, 3)",
            &context,
        )
        .unwrap(),
        MiraAny::from(7),
    );
    assert_eq!(
        eval(
            "let mut total = 0; for value in 1..5 { total += value; } total",
            &context
        )
        .unwrap(),
        MiraAny::from(15),
    );
}

#[test]
fn closures_capture_each_loop_iteration() {
    let result = eval(
        "let mut first = nil; let mut second = nil; for value in 1..2 { if value == 1 { first = fn { value }; } else { second = fn { value }; } } first() * 10 + second()",
        &MiraContext::empty(),
    )
    .unwrap();
    assert_eq!(result, MiraAny::from(12));
}

#[test]
fn native_functions_can_call_script_callbacks() {
    assert_eq!(
        eval("matrix.entrywise(1, 1, fn { 'x' })", &MiraContext::new()).unwrap(),
        MiraAny::from("x"),
    );
}

#[test]
fn rejects_escaping_script_values() {
    let error = eval("fn value { 1 } value", &MiraContext::empty()).unwrap_err();
    assert_eq!(error, MiraError::EscapingClosure);

    let error = eval("mod value { pub let x = 1; } value", &MiraContext::empty()).unwrap_err();
    assert_eq!(error, MiraError::EscapingClosure);
}

#[derive(Clone, MiraRecord)]
struct Foo {
    bar: u8,
    #[mira(rename = "display_name")]
    name: String,
    #[mira(skip)]
    _secret: String,
}

#[derive(Clone, MiraArray)]
#[allow(dead_code)]
struct Point(f64, f64, #[mira(skip)] String);

#[derive(Clone, MiraExtern)]
#[mira(tag = "Counter")]
struct Counter {
    value: i64,
    #[mira(readonly)]
    limit: i64,
}

#[test]
fn derived_live_values_bridge_rust_and_mirascript() {
    let foo = MiraShared::new(Foo {
        bar: 42,
        name: "Ada".into(),
        _secret: "token".into(),
    });
    let point = MiraShared::new(Point(3.0, 4.0, "metadata".into()));
    let counter = MiraShared::new(Counter { value: 1, limit: 9 });
    let mut context = MiraContext::empty();
    context.insert("foo", MiraAny::from(foo.clone()));
    context.insert("point", MiraAny::from(point));
    context.insert("counter", MiraAny::from(counter.clone()));

    assert_eq!(
        eval("foo.bar + point[0] + point[-1]", &context).unwrap(),
        MiraAny::from(49),
    );
    assert_eq!(
        eval("foo.display_name", &context).unwrap(),
        MiraAny::from("Ada"),
    );
    foo.borrow_mut().bar = 7;
    assert_eq!(eval("foo.bar", &context).unwrap(), MiraAny::from(7));

    assert_eq!(
        eval("counter.value = 5; counter.value", &context).unwrap(),
        MiraAny::from(5),
    );
    assert_eq!(counter.borrow().value, 5);
    let failed_conversion = eval("counter.value = 'bad'; nil", &context);
    assert!(matches!(
        failed_conversion,
        Err(MiraError::Conversion { .. })
    ));
    assert_eq!(counter.borrow().value, 5);
    let readonly_write = eval("counter.limit = 12; nil", &context);
    assert!(matches!(readonly_write, Err(MiraError::Runtime { .. })));
    assert_eq!(counter.borrow().limit, 9);
    context.insert("counter_alias", MiraAny::from(counter.clone()));
    assert_eq!(
        eval("counter == counter_alias", &context).unwrap(),
        MiraAny::Boolean(true),
    );

    let _borrow = foo.borrow_mut();
    assert!(matches!(
        eval("foo.bar", &context),
        Err(MiraError::BorrowConflict { .. })
    ));
}

#[test]
fn checked_integer_conversions_reject_finite_out_of_range_values() {
    assert!(u64::try_from(MiraAny::Number(2_f64.powi(64))).is_err());
    assert!(i64::try_from(MiraAny::Number(2_f64.powi(63))).is_err());
    assert_eq!(
        i64::try_from(MiraAny::Number(-2_f64.powi(63))).unwrap(),
        i64::MIN
    );
    assert!(u8::try_from(MiraAny::Number(1.5)).is_err());
}

#[test]
fn errors_preserve_diagnostics_and_runtime_context() {
    assert!(matches!(
        compile("let ="),
        Err(MiraError::Compile { diagnostics }) if !diagnostics.is_empty()
    ));

    let error = eval("fn outer { panic('boom') } outer()", &MiraContext::new()).unwrap_err();
    match error {
        MiraError::Runtime {
            function,
            offset,
            stack,
            ..
        } => {
            assert!(function.is_some());
            assert!(offset.is_some());
            assert!(!stack.is_empty());
        }
        error => panic!("expected runtime error with context, got {error:?}"),
    }
}
