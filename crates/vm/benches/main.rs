use divan::{Bencher, black_box};
use mira_vm::{MiraContext, RunOptions, compile};

// Keep this setup aligned with packages/mirascript/bench/index.ts so the
// TypeScript and Rust run-only results measure the same script and globals.
const SIMPLE: &str = "sin(x) + cos(y + PI / 2) + 0";
const SCALAR: &str = "let mut total = 0; for i in 1..100 { total += i * i; } total";
const CONTAINER: &str = "[1..100]::map(fn { it * 2 })::sum()";
const CLOSURE: &str = "fn make(x) { (fn (y) { x + y }) } let add = make(2); add(40)";
const STANDARD_LIBRARY: &str = "matrix.multiply([[1, 2], [3, 4]], [[5, 6], [7, 8]])::to_json()";

fn simple_context() -> MiraContext {
    let mut context = MiraContext::new();
    context.insert("x", 1);
    context.insert("y", 2);
    context
}

macro_rules! benchmark_case {
    ($compile_run:ident, $run_only:ident, $source:expr, $context:expr) => {
        #[divan::bench]
        fn $compile_run(bencher: Bencher) {
            let context = $context;
            bencher.bench_local(|| {
                let script = compile(black_box($source)).unwrap();
                black_box(script.run(black_box(&context)).unwrap())
            });
        }

        #[divan::bench]
        fn $run_only(bencher: Bencher) {
            let context = $context;
            let script = compile($source).unwrap();
            bencher.bench_local(|| black_box(script.run(black_box(&context)).unwrap()));
        }
    };
}

benchmark_case!(
    compile_run_simple,
    run_only_simple,
    SIMPLE,
    simple_context()
);
benchmark_case!(
    compile_run_scalar,
    run_only_scalar,
    SCALAR,
    MiraContext::new()
);
benchmark_case!(
    compile_run_container,
    run_only_container,
    CONTAINER,
    MiraContext::new()
);
benchmark_case!(
    compile_run_closure,
    run_only_closure,
    CLOSURE,
    MiraContext::new()
);
benchmark_case!(
    compile_run_standard_library,
    run_only_standard_library,
    STANDARD_LIBRARY,
    MiraContext::new()
);

/// Matches the TypeScript benchmark's pre-created execution configuration more
/// closely than `run_only_simple`, which also constructs default providers.
#[divan::bench]
fn run_with_simple(bencher: Bencher) {
    let context = simple_context();
    let options = RunOptions::default();
    let script = compile(SIMPLE).unwrap();
    bencher.bench_local(|| {
        black_box(
            script
                .run_with(black_box(&context), black_box(&options))
                .unwrap(),
        )
    });
}

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
