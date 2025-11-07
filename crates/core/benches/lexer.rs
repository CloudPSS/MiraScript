use std::fs;

use divan::prelude::*;
use mira_core::{Config, lexer::*};

#[divan::bench]
fn lexing() {
    let text = black_box(fs::read_to_string("../../examples/41_fib.mira").unwrap());

    let config = Config::default();
    let mut input = to_input(&text, &config);
    let tokens = lex(&mut input).unwrap();
    black_box(tokens);
}
