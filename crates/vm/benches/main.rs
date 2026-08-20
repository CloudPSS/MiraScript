use divan::{Bencher, black_box};
use mirascript_vm::{RunOptions, Runtime, compile};

// Keep this setup aligned with packages/mirascript/bench/index.ts so the
// TypeScript and Rust run-only results measure the same script and globals.
const SIMPLE: &str = "sin(x) + cos(y + PI / 2) + 0";
const NIL: &str = "nil";
const CONSTANT: &str = "1";
const GLOBAL: &str = "x";
const GLOBAL_ARITHMETIC: &str = "x + y";
const REPEATED_GLOBAL: &str = "x + x + x + x + x + x + x + x";
const NATIVE_CALL: &str = "sin(x)";
const SCALAR: &str = "let mut total = 0; for i in 1..100 { total += i * i; } total";
const CONTAINER: &str = "[1..100]::map(fn { it * 2 })::sum()";
const CLOSURE: &str = "fn make(x) { (fn (y) { x + y }) } let add = make(2); add(40)";
const STANDARD_LIBRARY: &str = "matrix.multiply([[1, 2], [3, 4]], [[5, 6], [7, 8]])::to_json()";

fn simple_runtime() -> Runtime {
    let mut runtime = Runtime::new();
    runtime.insert_global("x", 1).unwrap();
    runtime.insert_global("y", 2).unwrap();
    runtime
}

macro_rules! benchmark_case {
    ($compile_run:ident, $run_only:ident, $source:expr, $runtime:expr) => {
        #[divan::bench]
        fn $compile_run(bencher: Bencher) {
            let mut runtime = $runtime;
            bencher.bench_local(|| {
                let script = compile(black_box($source)).unwrap();
                black_box(runtime.run(black_box(&script)).unwrap())
            });
        }

        #[divan::bench]
        fn $run_only(bencher: Bencher) {
            let mut runtime = $runtime;
            let script = compile($source).unwrap();
            bencher.bench_local(|| black_box(runtime.run(black_box(&script)).unwrap()));
        }
    };
}

macro_rules! run_with_case {
    ($name:ident, $source:expr, $runtime:expr) => {
        #[divan::bench]
        fn $name(bencher: Bencher) {
            let mut runtime = $runtime;
            let script = compile($source).unwrap();
            bencher.bench_local(|| black_box(runtime.run(black_box(&script)).unwrap()));
        }
    };
}

benchmark_case!(
    compile_run_simple,
    run_only_simple,
    SIMPLE,
    simple_runtime()
);
benchmark_case!(compile_run_scalar, run_only_scalar, SCALAR, Runtime::new());
benchmark_case!(
    compile_run_container,
    run_only_container,
    CONTAINER,
    Runtime::new()
);
benchmark_case!(
    compile_run_closure,
    run_only_closure,
    CLOSURE,
    Runtime::new()
);
benchmark_case!(
    compile_run_standard_library,
    run_only_standard_library,
    STANDARD_LIBRARY,
    Runtime::new()
);

// The breakdown cases keep script, context, and options outside the timed loop.
// Together they isolate fixed run cost, globals, native calls, and dispatch.
run_with_case!(
    run_with_nil,
    NIL,
    Runtime::with_options(RunOptions::default())
);
run_with_case!(
    run_with_constant,
    CONSTANT,
    Runtime::with_options(RunOptions::default())
);
run_with_case!(run_with_global, GLOBAL, simple_runtime());
run_with_case!(
    run_with_global_arithmetic,
    GLOBAL_ARITHMETIC,
    simple_runtime()
);
run_with_case!(run_with_repeated_global, REPEATED_GLOBAL, simple_runtime());
run_with_case!(run_with_native_call, NATIVE_CALL, simple_runtime());
run_with_case!(run_with_simple, SIMPLE, simple_runtime());
run_with_case!(run_with_scalar_loop, SCALAR, Runtime::new());

#[divan::bench]
fn native_run_simple(bencher: Bencher) {
    bencher.bench_local(|| {
        let x = black_box(1.0_f64);
        let y = black_box(2.0_f64);
        black_box(x.sin() + (y + std::f64::consts::PI / 2.0).cos())
    });
}

fn main() {
    divan::main();
}
