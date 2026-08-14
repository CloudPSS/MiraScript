use mira_vm::MiraRecord;

#[derive(MiraRecord)]
struct Duplicate {
    value: u8,
    #[mira(rename = "value")]
    another: u8,
}

fn main() {}
