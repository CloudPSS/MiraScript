use mirascript_vm::mira;

#[mira]
mod values {
    #[mira(crate = my_crate)]
    const ANSWER: usize = 42;
}

fn main() {}
