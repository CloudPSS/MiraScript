mod lexer;
mod tokenizer;

fn main() {
    let text = r##"12 + (1.5:12,3,4) + 5"##;

    let mut input = tokenizer::to_input(text);
    let result = tokenizer::tokenize(&mut input, true).unwrap();
    let mut input = lexer::to_input(&result);
    let exp = lexer::lex(&mut input).unwrap();
    println!("{:?}", input.state);
    println!("{:#}", exp);
}
