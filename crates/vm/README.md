# mira-vm

`mira-vm` is the native, single-threaded Rust runtime for MiraScript. It uses
`mira-core` to compile source, validates the resulting bytecode, and executes it
without a JavaScript or Python host. `MiraContext::new()` installs the same public
standard-library surface as the TypeScript VM.

The crate currently remains `publish = false` while its public API is reviewed.

## Quick start

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

run()?;
# Ok::<(), mira_vm::MiraError>(())
```

`compile` returns a validated `MiraScript` that can be reused with different
contexts. `eval` is the convenience API for compile-and-run-once. User globals
inserted into a context may replace standard-library globals with the same name.

## Execution options and host providers

`MiraScript::run_with` accepts per-run limits:

- `timeout`: 100 ms by default;
- `checkpoint_interval`: 100 interpreter checkpoints by default;
- `max_call_depth`: 128 by default;
- `max_array_len`: `0x100_0000` by default.

Random numbers, the current time, and debug output are supplied by the object-safe
`RuntimeProviders` trait. Tests can replace the default providers without changing
process-wide state:

```rust
use std::rc::Rc;

use mira_vm::{MiraAny, MiraContext, RunOptions, RuntimeProviders, compile};

struct Deterministic;

impl RuntimeProviders for Deterministic {
    fn random(&self) -> f64 {
        0.25
    }
}

let options = RunOptions {
    providers: Rc::new(Deterministic),
    ..RunOptions::default()
};
let script = compile("random()")?;
assert_eq!(
    script.run_with(&MiraContext::new(), &options)?,
    MiraAny::Number(0.25),
);
# Ok::<(), mira_vm::MiraError>(())
```

Timeout checks are cooperative. Loop backedges, function calls, and returns from
native code reach checkpoints; a blocking host callback cannot be preempted and
should call `MiraCallContext::checkpoint` during long-running work.

## Values and live Rust bridges

`MiraAny` represents VM registers and public return values. Script-owned arrays
and records use `Vec<MiraAny>` and `IndexMap<String, MiraAny>`; the ordered map
keeps field iteration and serialization deterministic. Checked `TryFrom<MiraAny>`
implementations reject non-finite, fractional, or out-of-range integer conversions.

The derive macros expose Rust values as live views:

- `MiraRecord` supports named and unit structs and is read-only in scripts;
- `MiraArray` supports tuple and unit structs and is read-only in scripts;
- `MiraExtern` supports named structs, object identity, and field writes.

Fields support `#[mira(rename = "...")]` and `#[mira(skip)]`. Extern fields also
support `#[mira(readonly)]`, and an extern type can set `#[mira(tag = "...")]`.
Use `#[mira(crate = "...")]` when the `mira-vm` dependency is renamed.

A direct conversion transfers ownership to the VM wrapper. Use `MiraShared<T>`
when the host must retain and mutate the same object:

```rust
use mira_vm::{MiraAny, MiraContext, MiraRecord, MiraShared, compile};

#[derive(MiraRecord)]
struct Counter {
    value: i64,
}

let counter = MiraShared::new(Counter { value: 1 });
let mut context = MiraContext::new();
context.insert("counter", MiraAny::from(counter.clone()));
let script = compile("counter.value")?;

assert_eq!(script.run(&context)?, MiraAny::from(1));
counter.borrow_mut().value = 2;
assert_eq!(script.run(&context)?, MiraAny::from(2));
# Ok::<(), mira_vm::MiraError>(())
```

`MiraShared<T>` uses `Rc<RefCell<T>>`, so host values do not need `Send` or
`Sync`. VM-side dynamic borrow conflicts become `MiraError::BorrowConflict`
instead of panics.

Native functions receive a `MiraCallContext`. They can read values using VM
access rules, call script callbacks, inspect run options, and cooperate with
timeout checks. This is also the path used by callback-based standard-library
functions such as `map`, `filter`, and `fold`.

## Execution and lifetime model

Bytecode emitted by `mira-core` is decoded into a structured internal instruction
tree. Loading validates chunk boundaries, constant encodings and UTF-8, narrow and
wide parameters, opcode legality, register and constant references, and nested
function/control-flow terminators. Runtime errors retain bytecode offsets and call
stack context.

Each run owns a frame arena:

- function frames store registers and their captured parent frame;
- loops reuse a frame only when their body cannot create a closure or module;
- script functions store an instruction definition and an integer frame handle;
- upvalue operations walk parent handles by lexical level.

The arena intentionally has no runtime garbage collector. Script closures and
script modules cannot escape `run`; returning one produces
`MiraError::EscapingClosure`. If a host callback improperly caches one, later use
returns `MiraError::ExecutionEnded` rather than dereferencing stale memory.
Native functions, native modules, and live Rust values may escape safely.

## Runtime structure and TypeScript parity

The Rust source layout follows the TypeScript VM where the implementations share
a concept:

- `src/operations/` mirrors conversion, type checks, operators, record access,
  iterables, spread, slices, and ranges from
  `packages/mirascript/src/vm/operations/`;
- `src/standard_library/global/` mirrors global math, bit, sequence, string,
  JSON, conversion, debug, and time modules;
- `src/standard_library/module/matrix/` mirrors the TypeScript matrix module;
- bytecode decoding, interpreter state/control/calls, and values are split by
  responsibility under their own directories.

`MiraContext::new()` installs math and bit functions, string operations, sequence
operations, JSON and primitive conversion, debug and time providers, and the
`matrix` module. Parameter validation, fallback behavior, and errors are kept
compatible with the TypeScript runtime; host-specific JavaScript behavior is not
copied when it is outside MiraScript semantics.

## Error model

`MiraError` separates compilation diagnostics, invalid bytecode, runtime errors,
Rust conversion failures, extern failures, live-value borrow conflicts, timeouts,
call-depth exhaustion, escaping script references, and expired executions.
Conversion errors can add a nested field or index using `MiraError::at_path`.

## Performance

The retained optimizations target behavior shared by real workloads:

- static global names resolve to borrowed per-run slots;
- calls with zero to four arguments avoid heap argument vectors;
- the call stack keeps common depths inline;
- checkpoints use a countdown;
- frame/register state is exclusively borrowed by the runtime;
- decoded instructions use typed fields instead of generic parameter vectors;
- loops without captured environments reuse their frame;
- static small-arity calls and adjacent global arguments are quickened;
- numeric arithmetic has a typed fast path with semantic fallback.

On the original Windows benchmark host, the simple run-only script improved from
672 ns to roughly 286–289 ns, while the scalar loop improved from 15.38 us to
about 5.73 us. These are historical, harness-specific measurements; compare only
runs made on the same host with the same Divan/Tinybench duration.

JIT is not the default next step. It is worth evaluating only when representative,
long-running numeric workloads remain dispatch-bound and need a material further
gain. A viable JIT must also cover platform support, executable-memory policy,
compile latency, host/extern callbacks, timeout checkpoints, error stacks, and an
interpreter fallback. Single-run startup time alone is not a sufficient reason.

## Development

Run the host-value example with:

```text
cargo run -p mira-vm --example host_values
```

Validate the Rust implementation with:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo bench -p mira-vm --bench main -- --min-time 2 --sample-count 100
```

The benchmark suite covers compile-and-run and run-only paths for fixed overhead,
global access, native calls, scalar loops, containers, closures, and standard
library calls. Performance changes should pass the full compatibility and
lifetime test suite and improve more than a single microbenchmark.
