#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{fs, hint::black_box};

use mira_core::{Compiler, Config};

fn main() {
    let text = fs::read_to_string("../../examples/fib.mira").unwrap();

    for _ in 0..1_000_000 {
        let (code, errors) = Compiler::compile(&text, &Config::new());
        black_box(code);
        black_box(errors);
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
