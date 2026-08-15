# mira-vm

`mira-vm` is the native, single-threaded Rust runtime for MiraScript. It compiles with
`mira-core`, validates the emitted bytecode, and executes it without a JavaScript or Python host.
`MiraContext::new()` includes the same standard-library surface as the TypeScript runtime.

```rust
use mira_vm::{MiraAny, MiraContext, MiraRecord, compile};

#[derive(Clone, MiraRecord)]
struct Foo {
    bar: u8,
}

fn run() -> mira_vm::Result<()> {
    let mut context = MiraContext::new();
    context.insert("foo", MiraAny::from(Foo { bar: 42 }));

    let script = compile("foo.bar")?;
    assert_eq!(script.run(&context)?, MiraAny::from(42));
    Ok(())
}
```

Use `MiraShared<T>` when Rust must retain and mutate a value after it is inserted. Derived
`MiraRecord` and `MiraArray` values are live, read-only views; `MiraExtern` additionally permits
writes to fields not marked `#[mira(readonly)]`. Borrow conflicts are reported as
`MiraError::BorrowConflict` rather than panicking inside the VM.

`MiraScript::run_with` accepts per-run timeout, call-depth, array-length, and provider settings.
Implement `RuntimeProviders` and assign an `Rc<dyn RuntimeProviders>` to `RunOptions::providers`
to make random numbers, the current time, and debug output deterministic in tests. Native
functions can call script closures through `MiraCallContext::call` and should use
`MiraCallContext::checkpoint` during long-running host work.

Script-created closures and modules are scoped to one execution and cannot be returned from
`run`; doing so produces `MiraError::EscapingClosure`. Native functions, native modules, and Rust
live values may be returned safely. The crate is intentionally single-threaded and does not
require host values to implement `Send` or `Sync`.

Run the complete example with:

```text
cargo run -p mira-vm --example host_values
```
