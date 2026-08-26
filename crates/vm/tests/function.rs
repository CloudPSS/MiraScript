use mirascript_vm::{
    MiraError, MiraFunction, MiraManageable, MiraNativeFn, MiraValue, Runtime,
    RuntimeErrorKind::InvalidHandle,
};

#[test]
fn anonymous_fn() {
    let mut runtime = Runtime::new();
    let handle = runtime
        .insert_function(MiraNativeFn::new(|_, _| {
            Ok::<_, mirascript_vm::MiraError>(42)
        }))
        .unwrap();
    runtime
        .insert_global("f", MiraValue::function(handle))
        .unwrap();
    assert_eq!(runtime.eval_unchecked("f()").as_number_unchecked(), 42f64);

    let func = runtime.take_function(handle).unwrap();
    assert!(matches!(
        runtime.take_function(handle).unwrap_err().as_ref(),
        MiraError::Runtime {
            kind: InvalidHandle { .. },
            ..
        }
    ));
    assert_eq!(
        <MiraManageable as TryInto<MiraValue>>::try_into(func.call(&mut runtime, &[]).unwrap())
            .unwrap(),
        MiraValue::from(42)
    );
    assert_eq!(func.name(), "<anonymous>");
    let renamed = <MiraNativeFn as Clone>::clone(&func).with_name("new_name");
    assert_eq!(func.name(), "<anonymous>");
    assert_eq!(renamed.name(), "new_name");
    assert_eq!(
        <MiraManageable as TryInto<MiraValue>>::try_into(func.call(&mut runtime, &[]).unwrap())
            .unwrap(),
        MiraValue::from(42)
    );
}

#[test]
fn ok() {
    let mut runtime = Runtime::new();
    runtime
        .insert_fn("ok", MiraNativeFn::ok(|_, _| 42))
        .unwrap();
    assert_eq!(runtime.eval_unchecked("ok()").as_number_unchecked(), 42f64);
    assert_eq!(runtime.eval_unchecked("ok(1)").as_number_unchecked(), 42f64);
}

#[test]
fn err() {
    let mut runtime = Runtime::new();
    runtime
        .insert_fn(
            "err",
            MiraNativeFn::err(|_, _| MiraError::Compile {
                diagnostics: vec![],
            }),
        )
        .unwrap();
    assert!(matches!(
        runtime.eval("err()").unwrap_err().as_ref(),
        MiraError::Compile { diagnostics: _ }
    ));
    assert!(matches!(
        runtime.eval("err(12)").unwrap_err().as_ref(),
        MiraError::Compile { diagnostics: _ }
    ));
}
