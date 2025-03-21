#![allow(dead_code)]

use std::ops::Deref;

mod analyzer;
mod ansi;
mod emitter;
mod lexer;
mod parser;
mod utils;

fn main() {
    let text = r##"{
     a=1 let b =2 var c=3 d=5 val e=7 x
    }
    y"##;

    let mut input = lexer::to_input(text);
    let result = lexer::lex(&mut input, true).unwrap();
    println!("{:?}", result);

    let mut input = parser::to_input(&result);
    let exp = parser::parse(&mut input);

    println!("{:?}", input.deref());
    println!("{:#}", exp.as_ref().unwrap());
}
