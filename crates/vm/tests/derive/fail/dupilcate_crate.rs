use mirascript_vm::{MiraRecord, mira};

#[derive(MiraRecord)]
#[mira(crate = xx, crate = yy)]
struct Record {
    value: u8,
    another: u8,
}

#[mira(crate = xx, crate = yy)]
fn test() {}

fn main() {}
