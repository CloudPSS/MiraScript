#![allow(dead_code)]

use std::ops::Deref;

mod analyzer;
mod ansi;
mod emitter;
mod lexer;
mod parser;
mod utils;

fn main() {
    let text = r##"{var a =1;}
    "__${12 + 1 + xx}""##;

    let mut input = lexer::to_input(text);
    let result = lexer::lex(&mut input, true).unwrap();
    println!("{:?}", result);

    let mut input = parser::to_input(&result);
    let exp = parser::parse(&mut input);

    println!("{:?}", input.deref());
    println!("{:#}", exp.unwrap());
}
