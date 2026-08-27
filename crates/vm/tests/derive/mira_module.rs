use mirascript_vm::{MiraError, MiraFunction, MiraValue, Result, Runtime, RuntimeErrorKind, mira};

#[mira]
mod outer {
    use super::*;

    #[mira]
    const ANSWER: usize = 42;

    #[mira(use = "renamed")]
    const OTHER: usize = 3;

    #[mira]
    pub(super) fn add(runtime: &mut Runtime, a: f64, b: f64) -> Result<f64> {
        let _ = runtime.options();
        Ok(a + b)
    }

    #[mira]
    fn sum(start: f64, rest: &[MiraValue]) -> Result<f64> {
        rest.iter()
            .try_fold(start, |sum, value| Ok(sum + f64::try_from(*value)?))
    }

    #[mira]
    fn passthrough(value: MiraValue) -> MiraValue {
        value
    }

    #[mira(const = INCREMENT, rename = "outer.increment", use = "inc")]
    pub(super) fn one() -> usize {
        1
    }

    #[mira]
    mod inner {
        #[mira]
        fn f() -> usize {
            7
        }
    }

    pub(super) mod mid {
        use super::*;

        #[mira]
        pub(crate) mod inner {
            #[mira]
            pub(crate) fn f(value: usize) -> usize {
                value
            }
        }
    }
}

#[mira(const = CUSTOM_ROOT, rename = "custom.root")]
mod renamed_root {
    #[mira(use = "alias")]
    pub(super) mod child {
        #[mira]
        pub(crate) fn f(value: usize) -> usize {
            value
        }
    }
}

#[test]
fn modules_derive_names_exports_and_nested_context() {
    let mut runtime = Runtime::new();
    runtime.insert_global("outer", OUTER).unwrap();
    runtime.insert_global("reset", outer::mid::INNER).unwrap();
    assert_eq!(outer::one(), 1);

    assert_eq!(
        runtime.eval_unchecked(
            "outer.ANSWER + outer.renamed + outer.add(1, 2) + outer.sum(4, 5, 6) + outer.passthrough(2) + outer.inc() + outer.inner.f()"
        ),
        73.into(),
    );
    assert_eq!(runtime.eval_unchecked("reset.f(9)"), 9.into());

    let error = runtime.eval("outer.add()").unwrap_err();
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
    let add = runtime.eval_unchecked("outer.add");
    let add = unsafe {
        add.as_function()
            .unwrap()
            .upcast::<outer::__MiraFunction_add>()
    };
    assert_eq!(runtime.get_function(add).unwrap().name(), "outer.add");
    let increment = runtime.eval_unchecked("outer.inc");
    let increment = unsafe {
        increment
            .as_function()
            .unwrap()
            .upcast::<outer::__MiraFunction_one>()
    };
    assert_eq!(
        runtime.get_function(increment).unwrap().name(),
        "outer.increment"
    );
    let error = runtime.eval("reset.f()").unwrap_err();
    assert!(
        matches!(
            error.as_ref(),
            MiraError::Runtime {
                kind: RuntimeErrorKind::MissingArgument { name: "value" },
                ..
            }
        ),
        "{error:?}"
    );
    let reset = runtime.eval_unchecked("reset.f");
    let reset = unsafe {
        reset
            .as_function()
            .unwrap()
            .upcast::<outer::mid::inner::__MiraFunction_f>()
    };
    assert_eq!(runtime.get_function(reset).unwrap().name(), "inner.f");
}

#[test]
fn explicit_module_names_prefix_children_without_changing_export_keys() {
    let mut runtime = Runtime::new();
    runtime.insert_global("root", CUSTOM_ROOT).unwrap();
    assert_eq!(runtime.eval_unchecked("root.alias.f(11)"), 11.into());

    let function = runtime.eval_unchecked("root.alias.f");
    let function = unsafe {
        function
            .as_function()
            .unwrap()
            .upcast::<renamed_root::child::__MiraFunction_f>()
    };
    assert_eq!(
        runtime.get_function(function).unwrap().name(),
        "custom.root.child.f"
    );
}
