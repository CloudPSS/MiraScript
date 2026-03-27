use std::fs;

use divan::prelude::*;
use mira_core::{Compiler, Config};

#[divan::bench]
fn compile(bencher: Bencher) {
    let text = black_box(fs::read_to_string("../../examples/41_fib.mira").unwrap());

    bencher.bench_local(|| {
        let (code, errors) = Compiler::compile(&text, &Config::new());

        black_box(code);
        black_box(errors);
    });
}
