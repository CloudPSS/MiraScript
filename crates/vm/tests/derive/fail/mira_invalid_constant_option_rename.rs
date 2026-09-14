use mirascript_vm::mira;

#[mira]
mod values {
    #[mira(rename = "values.answer")]
    const ANSWER: usize = 42;
}

fn main() {}
