use mirascript_vm::MiraRecord;

#[derive(MiraRecord)]
#[mira(unsupported_option)]
struct Record {
    value: u8,
    #[mira(unsupported_option)]
    another: u8,
}

fn main() {}
