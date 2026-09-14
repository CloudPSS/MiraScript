use mirascript_vm::mira;

#[mira]
mod values {
    #[mira(const = M_ANSWER)]
    const ANSWER: usize = 42;
}

fn main() {}
