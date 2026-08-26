use mirascript_vm::mira;

#[mira]
fn identity<T>(value: T) -> T {
    value
}

fn main() {}
