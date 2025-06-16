use std::fs;

use divan::prelude::*;
use mira_core::{Config, compile_script};

#[divan::bench]
fn compile() {
    let text = fs::read_to_string("../../examples/fib.mira").unwrap();

    let (code, errors) = compile_script(&text, &Config {});

    black_box(code);
    black_box(errors);
}
