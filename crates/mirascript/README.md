# MiraScript for Rust

`mirascript` is the batteries-included Rust interface to the MiraScript compiler
and virtual machine. It re-exports the complete `mirascript-vm` API, including
the derive macros for exposing live Rust values to scripts.

```rust
use mirascript::{MiraAny, MiraContext, compile};

let script = compile("answer + 1")?;
let mut context = MiraContext::new();
context.insert("answer", 41);

assert_eq!(script.run(&context)?, MiraAny::Number(42.0));
# Ok::<(), mirascript::MiraError>(())
```

Compiler configuration and bytecode definitions are available through the
`core` module, while the complete runtime crate is also available as `vm`.

```rust
let config = mirascript::core::Config::new();
let script = mirascript::compile_with("40 + 2", &config)?;

assert_eq!(
    script.run(&mirascript::MiraContext::new())?,
    mirascript::MiraAny::Number(42.0),
);
# Ok::<(), mirascript::MiraError>(())
```
