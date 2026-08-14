use mira_vm::MiraRecord;

#[derive(MiraRecord)]
union Unsupported {
    value: u8,
}

fn main() {}
