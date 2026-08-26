use mirascript_vm::mira;

#[mira(use = "sum")]
fn add(a: f64, b: f64) -> f64 {
    a + b
}

fn main() {}
