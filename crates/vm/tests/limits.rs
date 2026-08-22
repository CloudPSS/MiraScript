use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use mirascript_vm::{
    MiraError, MiraManageable, MiraNativeFn, MiraValue, RunOptions, Runtime, RuntimeErrorKind,
    RuntimeProviders, compile,
};

struct DropProbe(Rc<Cell<usize>>);

#[derive(Default)]
struct TestRuntimeProviders {
    messages: Rc<RefCell<Vec<String>>>,
}

impl RuntimeProviders for TestRuntimeProviders {
    fn random(&self) -> f64 {
        0.25
    }

    fn now_millis(&self) -> i64 {
        0
    }

    fn debug(&self, message: &str) {
        self.messages.borrow_mut().push(message.to_owned());
    }
}

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

impl mirascript_vm::MiraRecord for DropProbe {
    fn len(&self) -> usize {
        0
    }

    fn index_of(&self, _key: &str) -> Option<usize> {
        None
    }

    fn key(&self, _index: usize) -> mirascript_vm::Result<&str> {
        Err(MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }

    fn get(
        &self,
        _self_handle: mirascript_vm::MiraHandle<dyn mirascript_vm::MiraRecord>,
        _runtime: &Runtime,
        _index: usize,
    ) -> mirascript_vm::Result<MiraManageable> {
        Err(MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
    }
}

fn probe_runtime(drops: &Rc<Cell<usize>>) -> Runtime {
    let drops = Rc::clone(drops);
    let mut runtime = Runtime::new();
    runtime.insert_fn(
        "make_probe",
        MiraNativeFn::builtin("make_probe", move |_, _| {
            Ok(MiraManageable::from_record(DropProbe(Rc::clone(&drops))))
        }),
    );
    runtime
}

#[test]
fn runtime_arena_persists_across_runs_and_drops_once() {
    let drops = Rc::new(Cell::new(0));
    {
        let mut runtime = probe_runtime(&drops);
        let script = compile("make_probe()").unwrap();
        let value = runtime.run(&script).unwrap();
        assert!(value.is_record());
        let string = runtime.run(&compile("'persistent'").unwrap()).unwrap();
        let array = runtime.run(&compile("[1, 2, 3]").unwrap()).unwrap();
        assert_eq!(drops.get(), 0);
        assert_eq!(runtime.run(&compile("42").unwrap()).unwrap(), 42.into());
        assert_eq!(string.as_str(&runtime).unwrap(), Some("persistent"));
        runtime.insert_global("old_array", array).unwrap();
        assert_eq!(
            runtime
                .run(&compile("old_array[0] + old_array[2]").unwrap())
                .unwrap(),
            4.into()
        );
        assert_eq!(drops.get(), 0);
    }
    assert_eq!(drops.get(), 1);
}

#[test]
fn limits_and_providers_are_runtime_configuration() {
    let messages = Rc::new(RefCell::new(Vec::new()));
    let options = RunOptions {
        timeout: Duration::from_secs(1),
        checkpoint_interval: 1,
        max_call_depth: 8,
        max_array_len: 3,
        providers: Rc::new(TestRuntimeProviders {
            messages: Rc::clone(&messages),
        }),
    };
    let mut runtime = Runtime::with_options(options);
    let script = compile(
        "debug_print('provider'); (random(), to_timestamp(nil), to_iso8601(nil))::to_json()",
    )
    .unwrap();
    let value = runtime.run(&script).unwrap();
    assert_eq!(
        value.as_str(&runtime).unwrap(),
        Some("{\"0\":0.25,\"1\":0,\"2\":\"1970-01-01T00:00:00.000Z\"}")
    );
    assert_eq!(messages.borrow().as_slice(), ["provider"]);

    assert!(matches!(
        runtime
            .run(&compile("repeat(1, 4)").unwrap())
            .unwrap_err()
            .as_ref(),
        MiraError::Runtime {
            kind: RuntimeErrorKind::ArrayLimit { max: 3, .. },
            ..
        }
    ));
    assert!(matches!(
        runtime
            .run(&compile("fn recurse { recurse() } recurse()").unwrap())
            .unwrap_err()
            .as_ref(),
        MiraError::Runtime {
            kind: RuntimeErrorKind::MaxCallDepth { max: 8 },
            ..
        }
    ));
}

#[test]
fn runtime_rejects_reentrant_run() {
    let nested = compile("1").unwrap();
    let mut runtime = Runtime::new();
    runtime.insert_fn(
        "reenter",
        MiraNativeFn::builtin("reenter", move |runtime, _| {
            runtime.run(&nested).map(Into::into)
        }),
    );
    let error = runtime.run(&compile("reenter()").unwrap()).unwrap_err();
    assert!(matches!(
        error.as_ref(),
        MiraError::Runtime {
            kind: RuntimeErrorKind::ReentrantRun,
            ..
        }
    ));
}

#[test]
fn script_function_handles_expire_after_their_run() {
    let cached = Rc::new(RefCell::new(None));
    let callback_cache = Rc::clone(&cached);
    let mut runtime = Runtime::new();
    runtime.insert_fn(
        "cache",
        MiraNativeFn::ok(move |_, args| {
            *callback_cache.borrow_mut() = args.first().cloned();
            MiraValue::nil()
        }),
    );
    runtime
        .run(&compile("cache(fn { 42 }); nil").unwrap())
        .unwrap();

    runtime
        .insert_global("cached", cached.borrow_mut().take().unwrap())
        .unwrap();
    let error = runtime.run(&compile("cached()").unwrap()).unwrap_err();
    assert!(matches!(
        error.as_ref(),
        MiraError::Runtime {
            kind: RuntimeErrorKind::ExecutionEnded,
            ..
        }
    ));
}

#[test]
fn script_module_handles_expire_after_their_run() {
    let cached = Rc::new(RefCell::new(None));
    let callback_cache = Rc::clone(&cached);
    let mut runtime = Runtime::new();
    runtime.insert_fn(
        "cache",
        MiraNativeFn::ok(move |_, args| {
            *callback_cache.borrow_mut() = args.first().cloned();
            MiraValue::nil()
        }),
    );
    runtime
        .run(&compile("mod value { pub let answer = 42; } cache(value); nil").unwrap())
        .unwrap();

    runtime
        .insert_global("cached", cached.borrow_mut().take().unwrap())
        .unwrap();
    let error = runtime.run(&compile("cached.answer").unwrap()).unwrap_err();
    assert!(matches!(
        error.as_ref(),
        MiraError::Runtime {
            kind: RuntimeErrorKind::ExecutionEnded,
            ..
        }
    ));
}
