use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use mirascript_vm::{
    MiraAny, MiraContext, MiraError, MiraNativeFn, MiraRecord, RunOptions, RuntimeProviders,
    compile,
};

struct DropProbe(Rc<Cell<usize>>);

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

impl MiraRecord for DropProbe {
    fn keys(&self) -> Vec<String> {
        Vec::new()
    }

    fn get(&self, _key: &str) -> mirascript_vm::Result<Option<MiraAny>> {
        Ok(None)
    }
}

fn probe_context(drops: &Rc<Cell<usize>>) -> MiraContext {
    let drops = Rc::clone(drops);
    let mut context = MiraContext::new();
    context.insert_fn(
        "make_probe",
        MiraNativeFn::ok(move |_, _| MiraAny::from_record(DropProbe(Rc::clone(&drops)))),
    );
    context
}

#[test]
fn execution_arena_drops_values_on_every_exit_path() {
    let cases = [
        ("let probe = make_probe(); nil", RunOptions::default()),
        (
            "let probe = make_probe(); panic('boom')",
            RunOptions::default(),
        ),
        (
            "let probe = make_probe(); loop { }",
            RunOptions {
                timeout: Duration::from_millis(10),
                checkpoint_interval: 1,
                ..RunOptions::default()
            },
        ),
        (
            "let probe = make_probe(); fn recurse { recurse() } recurse()",
            RunOptions {
                max_call_depth: 4,
                ..RunOptions::default()
            },
        ),
        (
            "let probe = make_probe(); for i in 0..10 { let capture = fn { i }; } nil",
            RunOptions::default(),
        ),
    ];

    for (source, options) in cases {
        let drops = Rc::new(Cell::new(0));
        let context = probe_context(&drops);
        let _ = compile(source).unwrap().run_with(&context, &options);
        assert_eq!(drops.get(), 1, "arena value leaked for {source}");
    }
}

#[test]
fn reusable_scripts_do_not_retain_frames_between_runs() {
    let drops = Rc::new(Cell::new(0));
    let context = probe_context(&drops);
    let script = compile("let probe = make_probe(); 42").unwrap();
    assert_eq!(script.run(&context).unwrap(), MiraAny::from(42));
    assert_eq!(script.run(&context).unwrap(), MiraAny::from(42));
    assert_eq!(drops.get(), 2);
}

#[test]
fn limits_and_providers_are_applied_per_run() {
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
    let context = MiraContext::new();
    let script =
        compile("debug_print('provider'); (random(), to_timestamp(nil), to_iso8601(nil))").unwrap();
    assert_eq!(
        script.run_with(&context, &options).unwrap(),
        MiraAny::from(indexmap::IndexMap::from([
            ("0".to_owned(), MiraAny::from(0.25)),
            ("1".to_owned(), MiraAny::from(0)),
            ("2".to_owned(), MiraAny::from("1970-01-01T00:00:00.000Z")),
        ])),
    );
    assert_eq!(messages.borrow().as_slice(), ["provider"]);

    assert!(matches!(
        compile("repeat(1, 4)")
            .unwrap()
            .run_with(&context, &options)
            .unwrap_err()
            .as_ref(),
        MiraError::Runtime { .. }
    ));
    assert!(matches!(
        compile("fn recurse { recurse() } recurse()")
            .unwrap()
            .run_with(&context, &options)
            .unwrap_err()
            .as_ref(),
        MiraError::MaxCallDepth { max: 8 }
    ));
}

#[test]
fn native_values_and_live_rust_values_may_escape() {
    let drops = Rc::new(Cell::new(0));
    let mut context = probe_context(&drops);
    context.insert("native", MiraNativeFn::ok(|_, _| MiraAny::Nil));

    assert!(matches!(
        compile("native").unwrap().run(&context).unwrap(),
        MiraAny::Function(_)
    ));
    let value = compile("make_probe() ").unwrap().run(&context).unwrap();
    assert_eq!(value.type_name(), "record");
    assert_eq!(drops.get(), 0);
    drop(value);
    assert_eq!(drops.get(), 1);
}

#[test]
fn script_function_handles_cached_by_hosts_expire_safely() {
    let cached = Rc::new(RefCell::new(None));
    let callback_cache = Rc::clone(&cached);
    let mut context = MiraContext::new();
    context.insert_fn(
        "cache",
        MiraNativeFn::ok(move |_, args| {
            *callback_cache.borrow_mut() = args.first().cloned();
            MiraAny::Nil
        }),
    );
    compile("cache(fn { 42 }); nil")
        .unwrap()
        .run(&context)
        .unwrap();

    context.insert("cached", cached.borrow_mut().take().unwrap());
    assert!(matches!(
        compile("cached()")
            .unwrap()
            .run(&context)
            .unwrap_err()
            .as_ref(),
        &MiraError::ExecutionEnded,
    ));
}
