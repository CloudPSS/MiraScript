use mirascript_vm::{
    MiraError, MiraFunction, MiraManageable, MiraNativeFn, MiraValue, Runtime,
    RuntimeErrorKind::InvalidHandle,
};

#[test]
fn anonymous_fn() {
    let mut runtime = Runtime::new();
    let handle = runtime
        .insert_function(MiraNativeFn::anonymous(|_, _| {
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
