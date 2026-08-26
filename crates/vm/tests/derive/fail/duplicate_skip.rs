use mirascript_vm::MiraRecord;

#[derive(MiraRecord)]
struct Duplicate {
    value: u8,
    #[mira(skip, skip)]
    another: u8,
}

fn main() {}
