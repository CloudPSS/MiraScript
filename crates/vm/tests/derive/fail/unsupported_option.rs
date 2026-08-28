use mirascript_vm::{MiraArray, MiraRecord, mira};

#[derive(MiraRecord)]
#[mira(unsupported_option)]
struct Record {
    value: u8,
    #[mira(unsupported_option)]
    another: u8,
}

#[derive(MiraRecord)]
struct Record2(#[mira(unsupported_option)] u8);

#[derive(MiraArray)]
struct Array(#[mira(unsupported_option)] u8);

#[mira(unsupported_option)]
fn test() {}

fn main() {}
