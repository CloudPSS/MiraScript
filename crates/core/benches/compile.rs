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

#[divan::bench(args = [32, 128, 512, 1024])]
fn logical_chain(bencher: Bencher, count: usize) {
    let text = vec!["true"; count].join(" && ");
    let config = CompileConfig::new();
    bencher.bench_local(|| black_box(Compiler::compile(black_box(&text), &config)));
}

#[divan::bench(args = [32, 128, 512, 1024])]
fn emit_logical_chain(bencher: Bencher, count: usize) {
    let text = vec!["true"; count].join(" && ");
    let config = CompileConfig::new();
    let mut parser = Compiler::new(&text, &config);
    let tokens = parser.lex().unwrap();
    let script = parser.parse(&tokens).unwrap();
    bencher.bench_local(|| {
        let mut compiler = Compiler::new(&text, &config);
        black_box(compiler.emit(black_box(&script)));
        black_box(compiler.diagnostics_collector.len());
    });
}
