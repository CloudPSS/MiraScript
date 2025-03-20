use std::ops::Deref;

mod lexer;
mod parser;

fn main() {
    let text = r##"fn {
       a = 3;
    }"##;

    let mut input = lexer::to_input(text);
    let result = lexer::lex(&mut input, true).unwrap();

    let mut input = parser::to_input(&result);
    let exp = parser::parse(&mut input);

    println!("{:?}", input.input.deref());
    println!("{:#}", exp.unwrap());
}
