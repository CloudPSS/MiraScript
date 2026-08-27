use mirascript_vm::{MiraError, MiraFunction, MiraValue, Result, Runtime, RuntimeErrorKind, mira};

#[test]
fn function_accept_str() {
    #[mira]
    fn hello(name: &str) -> String {
        format!("Hello, {name}!")
    }

    let mut runtime = Runtime::new();
    runtime.insert_global("hello", HELLO).unwrap();
    assert_eq!(
        runtime
            .eval_unchecked("hello('World')")
            .as_str_unchecked(&runtime),
        "Hello, World!"
    );
    assert_eq!(
        runtime.eval("hello(123)").unwrap_err().to_string(),
        "failed to convert value: expected Rust &str, got MiraScript number"
    );
}

#[test]
fn function_accept_optional() {
    #[mira]
    fn inv(value: Option<bool>) -> Option<bool> {
        value.map(|v| !v)
    }

    let mut runtime = Runtime::new();
    runtime.insert_global("inv", INV).unwrap();
    assert_eq!(runtime.eval_unchecked("inv(true)"), false.into());
    assert_eq!(
        runtime.eval("inv(1)").unwrap_err().to_string(),
        "failed to convert value: expected Rust bool, got MiraScript number"
    );
    assert!(runtime.eval_unchecked("inv()").is_nil());
    assert!(runtime.eval_unchecked("inv(nil)").is_nil());
}

#[test]
fn function_accept_bool() {
    #[mira]
    fn inv(value: bool) -> bool {
        !value
    }

    let mut runtime = Runtime::new();
    runtime.insert_global("inv", INV).unwrap();
    assert_eq!(runtime.eval_unchecked("inv(true)"), false.into());
    assert_eq!(
        runtime.eval("inv(1)").unwrap_err().to_string(),
        "failed to convert value: expected Rust bool, got MiraScript number"
    );
    assert_eq!(
        runtime.eval("inv()").unwrap_err().to_string(),
        "runtime failure: argument \"value\" is required"
    );
}

#[test]
fn function_returns_result() {
    #[mira]
    fn foo() -> Result<MiraValue> {
        Ok(MiraValue::NIL)
    }
    #[mira]
    fn err() -> Result<()> {
        Err(MiraError::Compile {
            diagnostics: vec![],
        }
        .into())
    }
    #[mira]
    fn any_err() -> anyhow::Result<()> {
        anyhow::bail!("any error")
    }

    let mut runtime = Runtime::new();
    runtime.insert_global("foo", FOO).unwrap();
    assert!(dbg!(runtime.eval_unchecked("foo()")).is_nil());

    runtime.insert_global("err", ERR).unwrap();
    let error = runtime.eval("err()").unwrap_err();
    assert!(matches!(
        error.as_ref(),
        MiraError::Compile { diagnostics } if diagnostics.is_empty()
    ));

    runtime.insert_global("any_err", ANY_ERR).unwrap();
    let error = runtime.eval("any_err()").unwrap_err();
    assert!(matches!(
        error.as_ref(),
        MiraError::External(source) if source.to_string() == "any error"
    ));
}

#[test]
fn functions_keep_rust_behavior_and_generate_insertable_constants() {
    #[mira]
    fn add(a: f64, b: f64) -> f64 {
        a + b
    }

    #[mira(const = DIFFERENCE, rename = "custom.subtract")]
    fn subtract(a: f64, b: f64) -> f64 {
        a - b
    }
    fn accepts_fn(function: impl Fn(f64, f64) -> f64) -> f64 {
        function(10.0, 4.0)
    }

    let pointer: fn(f64, f64) -> f64 = add;
    assert_eq!(pointer(1.0, 2.0), 3.0);
    assert_eq!(accepts_fn(add), 14.0);
    assert_eq!(subtract(10.0, 4.0), 6.0);

    let mut runtime = Runtime::new();
    runtime.insert_global("add", ADD).unwrap();
    runtime.insert_global("subtract", DIFFERENCE).unwrap();
    assert_eq!(runtime.eval_unchecked("add(1, 2)"), 3.into());
    assert_eq!(runtime.eval_unchecked("subtract(10, 4)"), 6.into());

    let error = runtime.eval("subtract() ").unwrap_err();
    assert!(
        matches!(
            error.as_ref(),
            MiraError::Runtime {
                kind: RuntimeErrorKind::MissingArgument { name: "a" },
                ..
            }
        ),
        "{error:?}"
    );
    let value = runtime.get_global("subtract").unwrap();
    let handle = unsafe {
        value
            .as_function()
            .unwrap()
            .upcast::<__MiraFunction_subtract>()
    };
    assert_eq!(
        runtime.get_function(handle).unwrap().name(),
        "custom.subtract"
    );
}
