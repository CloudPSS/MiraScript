# MiraScript for Rust

`mirascript` is the batteries-included Rust interface to the MiraScript compiler
and virtual machine. It re-exports the complete `mirascript-vm` API, including
the `MiraRecord` and `MiraArray` derive macros.

Compilation and execution are independent. A compiled script is reusable, while
the runtime owns globals, standard-library values, execution state, and its arena:

```rust
use mirascript::{MiraValue, Runtime, compile};

let script = compile("answer + 1")?;
let mut runtime = Runtime::new();
runtime.insert_global("answer", 41)?;

assert_eq!(runtime.run(&script)?, MiraValue::Number(42.0));
# Ok::<(), Box<mirascript::MiraError>>(())
```

Compiler configuration and bytecode definitions are available through `core`,
and the complete runtime crate is also available as `vm`.

```rust
let config = mirascript::core::CompileConfig::new();
let script = mirascript::compile_with("40 + 2", &config)?;
let mut runtime = mirascript::Runtime::new();

assert_eq!(
    runtime.run(&script)?,
    mirascript::MiraValue::Number(42.0),
);
# Ok::<(), Box<mirascript::MiraError>>(())
```
