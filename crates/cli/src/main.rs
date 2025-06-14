#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::fs;

use mira_core::{Config, compile_script};

fn main() {
    let text = fs::read_to_string("../../examples/fib.mira").unwrap();

    for _ in 0..1_000_000 {
        let (code, errors) = compile_script(&text, &Config {});
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
