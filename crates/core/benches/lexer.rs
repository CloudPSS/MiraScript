use std::fs;

use divan::prelude::*;
use mira_core::lexer::*;

#[divan::bench]
fn lexing() {
    let text = black_box(fs::read_to_string("../../examples/bad.mira").unwrap());

    let mut input = to_input(&text);
    let tokens = lex(&mut input).unwrap();
    black_box(tokens);
}
