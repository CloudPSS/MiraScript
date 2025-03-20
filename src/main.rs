use std::ops::Deref;

mod analyzer;
mod ansi;
mod emitter;
mod lexer;
mod parser;

fn main() {
    let text = r##"fn main {
       var a = "${x} + ${y} = ${x + "${z}"}"; // comment
    }"##;

    let mut input = lexer::to_input(text);
    let result = lexer::lex(&mut input, true).unwrap();

    let mut input = parser::to_input(&result);
    let exp = parser::parse(&mut input);

    println!("{:?}", input.input.deref());
    println!("{:#}", exp.unwrap());
}
