use std::fs;

use compile::compile_script;

mod analyzer;
mod ansi;
mod compile;
mod emitter;
mod error;
mod lexer;
mod parser;

fn main() {
    let text = fs::read_to_string("../../test/main.ms").unwrap();

    let result = compile_script(&text);

    let script = match result {
        Ok(script) => script,
        Err((script, errors)) => {
            for error in errors {
                eprintln!("{error}");
            }
            let Some(script) = script else {
                eprintln!("Failed to compile script");
                return;
            };
            script
        }
    };

    println!("{}", script);
}
