use mirascript_vm::{MiraRecord, mira};

#[derive(MiraRecord)]
struct Duplicate {
    value: u8,
    #[mira(rename = "value_1", rename = "value_2")]
    another: u8,
}

#[mira(rename = "value_1", rename = "value_2")]
fn test() {}

fn main() {}
