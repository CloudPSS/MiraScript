mod lexer;
mod tokenizer;

fn main() {
    let text = r##"fn { it == true or it > 0 or it == "
    " }"##;

    let mut input = tokenizer::to_input(text);
    let result = tokenizer::tokenize(&mut input, true).unwrap();

    let mut input = lexer::to_input(&result);
    let exp = lexer::lex(&mut input);

    println!("{:?}", input);
    println!("{:#}", exp.unwrap());
}
