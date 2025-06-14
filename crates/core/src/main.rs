use std::fs;

use compile::compile_script;

use crate::config::Config;

mod ansi;
mod compile;
mod config;
mod diagnostic;
mod emitter;
mod lexer;
mod parser;

fn main() {
    let text = fs::read_to_string("../../examples/fib.mira").unwrap();

    for _ in 0..1_000_000 {
        let (code, errors) = compile_script(
            &text,
            &Config {
                #[cfg(feature = "track_references")]
                track_references: false,
            },
        );
    }

    // for error in errors {
    //     eprintln!("{error}");
    // }
    // let Some(code) = code else {
    //     eprintln!("Failed to compile script");
    //     return;
    // };
    // println!("{:?}", code);
}
