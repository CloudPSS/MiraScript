use mirascript_vm::{MiraError, Runtime, RuntimeErrorKind::InvalidHandle};

#[test]
fn string_api() {
    let mut runtime = Runtime::new();
    let str = "Hello, 世界! こんにちは、世界！".to_string();
    let handle = runtime.insert_string(str.clone()).unwrap();
    runtime.insert_global("str", handle).unwrap();
    assert_eq!(
        runtime
            .eval_unchecked("str::chars()::len()")
            .as_number_unchecked(),
        str.chars().count() as f64
    );
    assert_eq!(
        runtime
            .eval_unchecked("`|$str|`")
            .as_str_unchecked(&runtime),
        format!("|{}|", str)
    );
    let str_ref = runtime.get_string(handle).unwrap();
    assert_eq!(str_ref, str.as_str());

    let appended = " Goodbye, 世界! さようなら、世界！";
    let str_mut = runtime.get_string_mut(handle).unwrap();
    str_mut.push_str(appended);
    assert_eq!(
        runtime
            .eval_unchecked("`|$str|`")
            .as_str_unchecked(&runtime),
        format!("|{str}{appended}|")
    );

    let taken = runtime.take_string(handle).unwrap();
    assert_eq!(taken, format!("{str}{appended}"));
    assert!(matches!(
        runtime.get_string(handle).unwrap_err().as_ref(),
        MiraError::Runtime {
            kind: InvalidHandle { .. },
            ..
        }
    ));
}
