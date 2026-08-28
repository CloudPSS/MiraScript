use mirascript_vm::mira;

#[mira]
mod valid {
    #[mira = "path"]
    mod invalid {}
}

#[mira = "path"]
mod invalid {}

fn main() {}
