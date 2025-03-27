#![allow(dead_code)]

use std::ops::Deref;

mod analyzer;
mod ansi;
mod emitter;
mod lexer;
mod parser;
mod utils;

fn main() {
    let text = r##"
    print <| "Hello ${"alice"}";
    
    [1,2,3] |> filter(fn {it % 2 == 0}) |> String
    
    for i in 1..<3 {
        print <| i;
    }

    for i in [1,2,3] |> map(fn {it * 2}) {
        print <| i;
    }

    fn  { it = 1 ;};

    1
    "##;

    let mut input = lexer::to_input(text);
    let result = lexer::lex(&mut input, true).unwrap();
    println!("{:?}", result);

    let mut input = parser::to_input(&result);
    let exp = parser::parse(&mut input);

    println!("{:?}", input.deref());
    println!("{:#?}", exp.as_ref().unwrap());
    println!("{:#}", exp.as_ref().unwrap());
}
