# mirascript-core

`mirascript-core` contains the MiraScript lexer, parser, compiler, bytecode
definitions, diagnostics, and optional formatter. Most embedders should depend
on the higher-level `mirascript` crate instead.

```rust
use mirascript_core::{Compiler, Config};

let (chunk, diagnostics) = Compiler::compile("40 + 2", &Config::new());
assert!(chunk.is_some());
assert!(diagnostics.is_empty());
```

Enable the `formatter` feature to access source formatting, or the `serde`
feature when compiler configuration needs serialization support.
