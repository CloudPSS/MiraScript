use mirascript_vm::mira;

#[mira]
#[mira]
mod duplicate {
    #[mira]
    mod inner {}
}

#[mira]
mod outer {
    #[mira]
    #[mira]
    mod duplicate {}
}

fn main() {}
