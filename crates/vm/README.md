# mirascript-vm

`mirascript-vm` is the native, single-threaded Rust runtime for MiraScript. Source
is compiled and validated independently from execution: a `MiraScript` contains
only the decoded program and owned constant table, while a `Runtime` owns globals,
the standard library, execution state, and every dynamically allocated value.

## Quick start

```rust
use mirascript_vm::{MiraRecord, MiraValue, Runtime, compile};

#[derive(MiraRecord)]
struct Foo {
    bar: u8,
}

let script = compile("foo.bar")?;
let mut runtime = Runtime::new();
let foo = runtime.insert_record(Foo { bar: 42 })?;
runtime.insert_global("foo", MiraValue::Record(foo.erase_record()))?;

assert_eq!(runtime.run(&script)?, MiraValue::Number(42.0));
# Ok::<(), Box<mirascript_vm::MiraError>>(())
```

One script can run in multiple runtimes, and one runtime can run any number of
scripts. Each run resets frames, registers, the call stack, timeout accounting,
and its execution generation. Globals, standard-library values, the arena, and
materialized string constants remain until the runtime is dropped. Re-entering
`Runtime::run` from a native callback returns `RuntimeErrorKind::ReentrantRun`.

## Values and arena ownership

`MiraValue` is a 16-byte, copyable value. Nil, booleans, numbers, and truly static
strings are inline; dynamic strings, arrays, records, functions, and modules use
typed handles into the owning runtime's append-only arena. Checked lookups reject
foreign, out-of-range, and wrongly typed handles.

The compiled constant table never contains runtime handles. A string constant is
allocated only when its bytecode instruction executes and is cached by
`(ScriptId, constant_index)` in that runtime. Unexecuted branches allocate
nothing, clones of one script share the cache identity, and different runtimes
materialize their own copies.

Use `Runtime::insert` for an owned `MiraManageable`, or the category-specific
`insert_string`, `insert_array`, `insert_record`, `insert_function`, and
`insert_module` methods when a typed `MiraHandle<T>` is needed. The matching
checked `get_*` and `get_*_mut` methods let the host inspect or mutate live values.

## Host records, arrays, functions, and modules

All compound behavior is dispatched through `dyn MiraArray`, `dyn MiraRecord`,
`dyn MiraFunction`, and `dyn MiraModule`. Script and host implementations use the
same traits; `MiraManageable` carries an owned value until `Runtime::insert`
places it in the correct arena.

`#[derive(MiraRecord)]` supports all structs and `#[derive(MiraArray)]` supports
tuple and unit structs. Both preserve `#[mira(rename = "...")]`,
`#[mira(skip)]`, generics, dependency aliases, and `#[mira(crate = ...)]`.
Nested derived records and arrays are projection views: they retain a typed parent
handle and read the field from the runtime on every access instead of cloning it.

```rust
use mirascript_vm::{MiraRecord, MiraValue, Runtime, compile};

#[derive(MiraRecord)]
struct Position { x: f64, y: f64 }

#[derive(MiraRecord)]
struct User { name: String, position: Position }

let mut runtime = Runtime::new();
let user = runtime.insert_record(User {
    name: "Ada".into(),
    position: Position { x: 3.0, y: 4.0 },
})?;
runtime.insert_global("user", MiraValue::Record(user.erase_record()))?;

let script = compile("user.position.x")?;
assert_eq!(runtime.run(&script)?, MiraValue::Number(3.0));
runtime.get_record_mut(user)?.position.x = 10.0;
assert_eq!(runtime.run(&script)?, MiraValue::Number(10.0));
# Ok::<(), Box<mirascript_vm::MiraError>>(())
```

`MiraExtern` is currently only a public, sealed marker. It has no constructor or
derive macro, and downstream crates cannot implement it.

## Execution options and providers

`Runtime::with_options` configures the timeout, checkpoint interval, maximum call
depth, maximum bounded array length, and a `RuntimeProviders` implementation for
random numbers, time, and debug output.

```rust
use std::rc::Rc;
use mirascript_vm::{RunOptions, Runtime, RuntimeProviders, compile};

struct Deterministic;
impl RuntimeProviders for Deterministic {
    fn random(&self) -> f64 { 0.25 }
}

let mut runtime = Runtime::with_options(RunOptions {
    providers: Rc::new(Deterministic),
    ..RunOptions::default()
});
assert_eq!(runtime.run(&compile("random()")?)?.as_number(), Some(0.25));
# Ok::<(), Box<mirascript_vm::MiraError>>(())
```

Timeout checks are cooperative. Loop backedges and function calls reach
checkpoints; a blocking native callback cannot be preempted.

## Lifetimes and errors

Strings, arrays, and records returned from a run remain valid in that runtime.
Script closures and modules retain their shared decoded program and execution
generation but cannot escape as results; cached callables from an older run return
`RuntimeErrorKind::ExecutionEnded`. The runtime has no garbage collector: all arena
values are released together when it is dropped.

`MiraError` separates compiler diagnostics, invalid bytecode, structured runtime
failures, conversion failures with field/index paths, and arbitrary host errors.
Runtime errors retain bytecode offsets and call-stack context.

## Development

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo doc --workspace --no-deps
git diff --check
```
