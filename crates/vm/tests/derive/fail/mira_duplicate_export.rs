use mirascript_vm::mira;

#[mira]
mod duplicate {
    #[mira(use = "value")]
    fn first() {}

    #[mira(use = "value")]
    const SECOND: usize = 2;
}

fn main() {}
