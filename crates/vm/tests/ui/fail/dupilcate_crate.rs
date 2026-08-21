use mirascript_vm::MiraRecord;

#[derive(MiraRecord)]
#[mira(crate = "xx", crate = "yy")]
struct Record {
    value: u8,
    another: u8,
}

fn main() {}
