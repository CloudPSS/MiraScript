use std::fs;

use divan::prelude::*;
use mirascript_core::{CompileConfig, Compiler};

#[divan::bench]
fn compile(bencher: Bencher) {
    let text = black_box(fs::read_to_string("../../examples/41_fib.mira").unwrap());

    bencher.bench_local(|| {
        let (code, errors) = Compiler::compile(&text, &CompileConfig::new());

        black_box(code);
        black_box(errors);
    });
}
