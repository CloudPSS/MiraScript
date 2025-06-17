use std::fs;

use divan::prelude::*;
use mira_core::{Config, compile as c};

#[divan::bench]
fn compile(bencher: Bencher) {
    let text = fs::read_to_string("../../examples/fib.mira").unwrap();

    bencher.bench_local(|| {
        let (code, errors) = c(&text, &Config::new());

        black_box(code);
        black_box(errors);
    });
}
