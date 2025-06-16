mod compile;
mod lexer;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    // Run registered benchmarks.
    divan::main();
}
