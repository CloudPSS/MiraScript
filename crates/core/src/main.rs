use std::fs;

use compile::compile_script;

mod ansi;
mod compile;
mod diagnostic;
mod emitter;
mod lexer;
mod parser;

fn main() {
    let text = fs::read_to_string("../../test/main.ms").unwrap();

    let (code, errors) = compile_script(&text);

    for error in errors {
        eprintln!("{error}");
    }
    let Some(code) = code else {
        eprintln!("Failed to compile script");
        return;
    };
    println!("{:?}", code);
}
